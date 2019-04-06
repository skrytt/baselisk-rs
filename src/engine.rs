
use buffer::ResizableFrameBuffer;
use defs;
use event::{ControllerBindData, EngineEvent, MidiEvent, ModulatableParameter, PatchEvent};
use processor::{Adsr, Gain, Oscillator, LowPassFilter,
                ModulationMatrix, MonoNoteSelector, PitchBend, Waveshaper};
use sample::slice;

pub struct Engine
{
    // Misc
    pub engine_event_buffer: Vec<(usize, EngineEvent)>,
    pub note_selector: MonoNoteSelector,
    pub pitch_bend: PitchBend,
    pub modulation_matrix: ModulationMatrix,
    // Buffers
    pub adsr_buffer: ResizableFrameBuffer<defs::MonoFrame>,
    // DSP Units
    pub oscillator: Oscillator,
    pub adsr: Adsr,
    pub gain: Gain,
    pub low_pass_filter: LowPassFilter,
    pub waveshaper: Waveshaper,
}

impl Engine
{
    pub fn new() -> Engine {
        Engine {
            // Engine Event Processing
            engine_event_buffer: Vec::with_capacity(defs::ENGINE_EVENT_BUF_LEN),
            note_selector: MonoNoteSelector::new(),
            pitch_bend: PitchBend::new(),
            modulation_matrix: ModulationMatrix::new(),
            // Buffers
            adsr_buffer: ResizableFrameBuffer::new(),
            // DSP Units
            oscillator: Oscillator::new(),
            adsr: Adsr::new(),
            gain: Gain::new(1.0),
            low_pass_filter: LowPassFilter::new(),
            waveshaper: Waveshaper::new(),
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
                        _ => Err("Unimplemented"),
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
                        self.low_pass_filter.update_frequency(data)
                    },
                    ModulatableParameter::FilterSweepRange => {
                        self.low_pass_filter.update_sweep(data)
                    },
                    ModulatableParameter::FilterQuality => {
                        self.low_pass_filter.update_quality(data)
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

        // Oscillator
        self.oscillator.process_buffer(main_buffer,
                                       self.engine_event_buffer.iter(),
                                       sample_rate);

        // ADSR buffer for Gain and Filter (shared for now)
        let frames_this_buffer = main_buffer.len();
        let adsr_buffer = self.adsr_buffer.get_sized_mut(frames_this_buffer);
        self.adsr.process_buffer(adsr_buffer,
                                 self.engine_event_buffer.iter(),
                                 sample_rate);

        // Gain
        self.gain.process_buffer(adsr_buffer,
                                 main_buffer);

        // Filter
        self.low_pass_filter.process_buffer(adsr_buffer,
                                            main_buffer,
                                            self.engine_event_buffer.iter(),
                                            sample_rate);

        // Waveshaper
        self.waveshaper.process_buffer(main_buffer);

    }

    fn handle_midi_panic(&mut self) {
        self.note_selector.midi_panic();
        self.oscillator.midi_panic();
        self.adsr.midi_panic();
        self.low_pass_filter.midi_panic();
    }
}
