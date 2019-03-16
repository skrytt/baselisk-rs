
use audio_thread::buffer::Buffer;
use defs;
use event;
use processor::{Adsr, Gain, Oscillator, LowPassFilter, MonoNoteSelector, Waveshaper};
use sample::slice;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Engine
{
    // Misc
    pub event_buffer: Rc<RefCell<event::Buffer>>,
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
    pub fn new(event_buffer: &Rc<RefCell<event::Buffer>>) -> Engine {
        Engine{
            // Misc
            event_buffer: Rc::clone(event_buffer),
            note_selector: MonoNoteSelector::new(),
            // Buffers
            adsr_buffer: Buffer::new(),
            // DSP Units
            oscillator: Oscillator::new(event_buffer),
            adsr: Adsr::new(event_buffer),
            gain: Gain::new(1.0),
            low_pass_filter: LowPassFilter::new(),
            waveshaper: Waveshaper::new(),
        }
    }

    /// Request audio.
    /// Buffer is a mutable slice of frames,
    /// where each frame is a slice containing a single sample.
    pub fn audio_requested(&mut self,
                           main_buffer: &mut defs::FrameBuffer,
                           sample_rate: defs::Sample)
    {
        let frames_this_buffer = main_buffer.len();

        // Zero the buffer
        slice::equilibrium(main_buffer);

        // Note Selector
        self.note_selector.process_midi_events(
            self.event_buffer.borrow().iter_midi());
        let selected_note = self.note_selector.get_note();

        // Oscillator
        self.oscillator.process_buffer(main_buffer, selected_note, sample_rate);

        // ADSR buffer for Gain and Filter (shared for now)
        let adsr_buffer = self.adsr_buffer.get_sized_mut(frames_this_buffer);
        self.adsr.process_buffer(adsr_buffer, sample_rate);

        // Gain
        self.gain.process_buffer(adsr_buffer, main_buffer);

        // Filter
        self.low_pass_filter.process_buffer(adsr_buffer, main_buffer, sample_rate);

        // Waveshaper
        self.waveshaper.process_buffer(main_buffer);

    }
}
