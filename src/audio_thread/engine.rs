
use audio_thread::buffer::Buffer;
use comms;
use defs;
use event;
use processor::{Adsr, Gain, Oscillator, LowPassFilter, MonoNoteSelector, Waveshaper};
use sample::slice;

pub struct Engine
{
    // Misc
    pub comms: comms::AudioThreadComms,
    pub event_buffer: event::Buffer,
    pub note_selector: MonoNoteSelector,
    // Buffers
    pub adsr_buffer: Buffer,
    // DSP Units
    pub oscillator: Oscillator,
    pub adsr: Adsr,
    pub gain: Gain,
    pub low_pass_filter: LowPassFilter,
    pub waveshaper: Waveshaper,
}

impl Engine
{
    pub fn new(comms: comms::AudioThreadComms,
               portmidi: portmidi::PortMidi) -> Engine {
        Engine{
            // Misc
            comms,
            event_buffer: event::Buffer::new(portmidi),
            note_selector: MonoNoteSelector::new(),
            // Buffers
            adsr_buffer: Buffer::new(),
            // DSP Units
            oscillator: Oscillator::new(),
            adsr: Adsr::new(),
            gain: Gain::new(1.0),
            low_pass_filter: LowPassFilter::new(),
            waveshaper: Waveshaper::new(),
        }
    }

    fn apply_patch_events(&mut self) {
        // Handle patch events but don't block if none are available.
        // Where view updates are required, send events back
        // to the main thread to indicate success or failure.
        while let Ok(event) = self.comms.rx.try_recv() {
            if let event::Event::Patch(event) = event {
                let result: Result<(), &'static str> = match event {

                    event::PatchEvent::MidiDeviceSet { device_id } => {
                        self.event_buffer.midi.set_port(device_id)
                    },
                    event::PatchEvent::OscillatorTypeSet { type_name } => {
                        self.oscillator.set_type(&type_name)
                    },
                    event::PatchEvent::OscillatorPitchSet { semitones } => {
                        self.oscillator.set_pitch(semitones)
                    },
                    event::PatchEvent::OscillatorPulseWidthSet { width } => {
                        self.oscillator.set_pulse_width(width)
                    },
                    event::PatchEvent::FilterFrequencySet { hz } => {
                        self.low_pass_filter.set_frequency(hz)
                    },
                    event::PatchEvent::FilterQualitySet { q } => {
                        self.low_pass_filter.set_quality(q)
                    },
                    event::PatchEvent::AdsrAttackSet { duration } => {
                        self.adsr.set_attack(duration)
                    },
                    event::PatchEvent::AdsrDecaySet { duration } => {
                        self.adsr.set_decay(duration)
                    },
                    event::PatchEvent::AdsrSustainSet { level } => {
                        self.adsr.set_sustain(level)
                    },
                    event::PatchEvent::AdsrReleaseSet { duration } => {
                        self.adsr.set_release(duration)
                    },
                    event::PatchEvent::WaveshaperInputGainSet { gain } => {
                        self.waveshaper.set_input_gain(gain)
                    },
                    event::PatchEvent::WaveshaperOutputGainSet { gain } => {
                        self.waveshaper.set_output_gain(gain)
                    },
                };
                self.comms.tx.send(result)
                    .expect("Failed to send response to main thread");
            }
        }
    }

    /// Request audio.
    /// Buffer is a mutable slice of frames,
    /// where each frame is a slice containing a single sample.
    pub fn audio_requested(&mut self,
                           main_buffer: &mut defs::FrameBuffer,
                           sample_rate: defs::Sample)
    {
        self.apply_patch_events();
        self.event_buffer.update_midi();

        let frames_this_buffer = main_buffer.len();

        // Zero the buffer
        slice::equilibrium(main_buffer);

        // Note Selector
        self.note_selector.process_midi_events(
            self.event_buffer.iter_midi());
        let selected_note = self.note_selector.get_note();

        // Oscillator
        self.oscillator.process_buffer(main_buffer,
                                       selected_note,
                                       self.event_buffer.iter_midi(),
                                       sample_rate);

        // ADSR buffer for Gain and Filter (shared for now)
        let adsr_buffer = self.adsr_buffer.get_sized_mut(frames_this_buffer);
        self.adsr.process_buffer(adsr_buffer,
                                 self.event_buffer.iter_midi(),
                                 sample_rate);

        // Gain
        self.gain.process_buffer(adsr_buffer,
                                 main_buffer);

        // Filter
        self.low_pass_filter.process_buffer(adsr_buffer,
                                            main_buffer,
                                            sample_rate);

        // Waveshaper
        self.waveshaper.process_buffer(main_buffer);

    }
}
