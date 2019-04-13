
use buffer::ResizableFrameBuffer;
use defs;
use event::{ControllerBindData, EngineEvent, MidiEvent, ModulatableParameter, PatchEvent};
use processor::{Adsr, Gain, Oscillator, Filter,
                ModulationMatrix, MonoNoteSelector, PitchBend, Waveshaper};
use sample::slice;

pub struct Engine
{
    // Misc
    engine_event_buffer: Vec<(usize, EngineEvent)>,
    note_selector: MonoNoteSelector,
    pitch_bend: PitchBend,
    modulation_matrix: ModulationMatrix,
    timing_data: TimingData,
    dump_timing_info: bool,
    // Buffers
    adsr_buffer: ResizableFrameBuffer<defs::MonoFrame>,
    // DSP Units
    oscillator: Oscillator,
    adsr: Adsr,
    gain: Gain,
    filter: Filter,
    waveshaper: Waveshaper,
    //delay: Delay,
}

impl Engine
{
    pub fn new(dump_timing_info: bool) -> Engine {
        Engine {
            // Engine Event Processing
            engine_event_buffer: Vec::with_capacity(defs::ENGINE_EVENT_BUF_LEN),
            note_selector: MonoNoteSelector::new(),
            pitch_bend: PitchBend::new(),
            modulation_matrix: ModulationMatrix::new(),
            timing_data: Default::default(),
            dump_timing_info,
            // Buffers
            adsr_buffer: ResizableFrameBuffer::new(),
            // DSP Units
            oscillator: Oscillator::new(),
            adsr: Adsr::new(),
            gain: Gain::new(1.0),
            filter: Filter::new(),
            waveshaper: Waveshaper::new(),
            //delay: Delay::new(),
        }
    }

    pub fn apply_patch_events(&mut self,
                              rx: &std::sync::mpsc::Receiver<PatchEvent>,
                              tx: &std::sync::mpsc::SyncSender<Result<(), &'static str>>,
    )
    {
        // Handle patch events but don't block if none are available.
        // Where view updates are required, send events back
        // to the main thread to indicate success or failure.
        while let Ok(patch_event) = rx.try_recv() {
            let result: Result<(), &'static str> = match patch_event {
                PatchEvent::PitchBendRangeSet { semitones } => {
                    self.pitch_bend.set_range(semitones)
                },
                PatchEvent::OscillatorTypeSet { type_name } => {
                    self.oscillator.set_type(&type_name)
                },
                PatchEvent::ControllerBindUpdate { parameter, bind_type } => {
                    match bind_type {
                        ControllerBindData::MidiLearn => {
                            self.modulation_matrix.learn_parameter(parameter)
                        },
                        ControllerBindData::CliInput(cc_number) => {
                            self.modulation_matrix.bind_parameter(cc_number, parameter)
                        },
                    }
                },
                PatchEvent::ModulatableParameterUpdate { parameter, data } => match parameter {
                    ModulatableParameter::AdsrAttack => {
                        self.adsr.update_attack(data)
                    },
                    ModulatableParameter::AdsrDecay => {
                        self.adsr.update_decay(data)
                    },
                    ModulatableParameter::AdsrSustain => {
                        self.adsr.update_sustain(data)
                    },
                    ModulatableParameter::AdsrRelease => {
                        self.adsr.update_release(data)
                    },
                    ModulatableParameter::FilterFrequency => {
                        self.filter.update_frequency(data)
                    },
                    ModulatableParameter::FilterSweepRange => {
                        self.filter.update_sweep(data)
                    },
                    ModulatableParameter::FilterQuality => {
                        self.filter.update_quality(data)
                    },
                    ModulatableParameter::OscillatorPitch => {
                        self.oscillator.update_pitch(data)
                    },
                    ModulatableParameter::OscillatorPulseWidth => {
                        self.oscillator.update_pulse_width(data)
                    },
                    ModulatableParameter::WaveshaperInputGain => {
                        self.waveshaper.update_input_gain(data)
                    },
                    ModulatableParameter::WaveshaperOutputGain => {
                        self.waveshaper.update_output_gain(data)
                    },
                },
            };
            // TODO: either fix this, or refactor it out
            tx.send(result)
                .expect("Failed to send response to main thread");
        }
    }

    /// Request audio.
    /// Buffer is a mutable slice of frames,
    /// where each frame is a slice containing a single sample.
    pub fn audio_requested(&mut self,
                           main_buffer: &mut defs::MonoFrameBufferSlice,
                           raw_midi_iter: jack::MidiIter,
                           sample_rate: defs::Sample)
    {
        // Zero the buffer
        slice::equilibrium(main_buffer);

        self.engine_event_buffer.clear();
        let mut midi_panic = false;
        for raw_midi_event in raw_midi_iter {
            if let Some((frame_num, midi_event)) = MidiEvent::parse(raw_midi_event, None) {
                // Check for MIDI panics.
                match midi_event {
                    MidiEvent::AllNotesOff | MidiEvent::AllSoundOff => {
                        midi_panic = true;
                        break
                    },
                    _ => (),
                }
                if let Some(engine_event) = self.note_selector.process_event(&midi_event) {
                    self.engine_event_buffer.push((frame_num, engine_event));
                }
                if let Some(engine_event) = self.pitch_bend.process_event(&midi_event) {
                    self.engine_event_buffer.push((frame_num, engine_event));
                }
                if let Some(engine_event) = self.modulation_matrix.process_event(&midi_event) {
                    self.engine_event_buffer.push((frame_num, engine_event));
                }
            }
        }

        // If we are panicking, we run this alternate code to reset state
        // and do not process audio this buffer.
        if midi_panic {
            self.handle_midi_panic();
            return
        }

        // ADSR buffer for Gain and Filter (shared for now).
        let adsr_start_time = time::precise_time_ns();
        let frames_this_buffer = main_buffer.len();
        let adsr_buffer = self.adsr_buffer.get_sized_mut(frames_this_buffer);
        let adsr_any_nonzero_output = self.adsr.process_buffer(adsr_buffer,
                                 self.engine_event_buffer.iter(),
                                 sample_rate);
        self.timing_data.adsr = (time::precise_time_ns() - adsr_start_time) / 1000;

        // Optimization: when ADSR is in the off state for a whole buffer,
        // the result of the oscillator and gain stages is silence
        if adsr_any_nonzero_output {

            // Oscillator
            let oscillator_start_time = time::precise_time_ns();
            self.oscillator.process_buffer(main_buffer,
                                           self.engine_event_buffer.iter(),
                                           sample_rate);
            self.timing_data.oscillator = (time::precise_time_ns() - oscillator_start_time) / 1000;

            // Use ADSR to apply gain to oscillator output
            let gain_start_time = time::precise_time_ns();
            self.gain.process_buffer(adsr_buffer,
                                     main_buffer);
            self.timing_data.gain = (time::precise_time_ns() - gain_start_time) / 1000;

        }

        // Filter
        let filter_start_time = time::precise_time_ns();
        self.filter.process_buffer(adsr_buffer,
                                   main_buffer,
                                   self.engine_event_buffer.iter(),
                                   sample_rate);
        self.timing_data.filter = (time::precise_time_ns() - filter_start_time) / 1000;

        // Waveshaper
        let waveshaper_start_time = time::precise_time_ns();
        self.waveshaper.process_buffer(main_buffer,
                                       self.engine_event_buffer.iter());
        self.timing_data.waveshaper = (time::precise_time_ns() - waveshaper_start_time) / 1000;

        // Delay
        //let delay_start_time = time::precise_time_ns();
        //self.delay.process_buffer(main_buffer,
        //                          self.engine_event_buffer.iter(),
        //                          sample_rate);
        //self.timing_data.delay = (time::precise_time_ns() - delay_start_time) / 1000;

        if self.dump_timing_info {
            self.timing_data.dump_to_stderr();
        }
    }

    fn handle_midi_panic(&mut self) {
        self.note_selector.midi_panic();
        self.oscillator.midi_panic();
        self.adsr.midi_panic();
        self.filter.midi_panic();
    }
}

#[derive(Default)]
struct TimingData {
    oscillator: u64,
    adsr: u64,
    gain: u64,
    filter: u64,
    waveshaper: u64,
    delay: u64,
}
impl TimingData {
    fn dump_to_stderr(&self) {
        eprintln!("osc={}us adsr={}us gain={}us filter={}us waveshaper={}us delay={}us",
                self.oscillator,
                self.adsr,
                self.gain,
                self.filter,
                self.waveshaper,
                self.delay
        );
    }
}
