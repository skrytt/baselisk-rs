
use audio_thread::buffer::Buffer;
use defs;
use dsp;
use dsp::sample::frame;
use event;
use processor::{Adsr, Gain, Oscillator, LowPassFilter, Waveshaper};

use std::rc::Rc;
use std::cell::RefCell;

pub struct Engine
{
    // Buffers
    pub adsr_buffer: Buffer,
    // DSP Units
    pub oscillator: Oscillator,
    pub adsr_gain: Adsr,
    pub gain: Gain,
    pub low_pass_filter: LowPassFilter,
    pub waveshaper: Waveshaper,
}

impl Engine
{
    pub fn new(event_buffer: &Rc<RefCell<event::Buffer>>) -> Engine {
        Engine{
            // Buffers
            adsr_buffer: Buffer::new(),
            // DSP Units
            oscillator: Oscillator::new(event_buffer),
            adsr_gain: Adsr::new(event_buffer),
            gain: Gain::new(1.0),
            low_pass_filter: LowPassFilter::new(),
            waveshaper: Waveshaper::new(),
        }
    }

    /// Request audio.
    /// Buffer is a mutable slice of frames,
    /// where each frame is a slice containing a single sample.
    pub fn audio_requested(&mut self,
                           main_buffer: &mut [frame::Mono<defs::Output>],
                           sample_rate: defs::Output)
    {
        let frames_this_buffer = main_buffer.len();

        // Zero the buffer
        dsp::slice::equilibrium(main_buffer);

        // Oscillator
        self.oscillator.update_state(sample_rate);
        self.oscillator.process_buffer(main_buffer, sample_rate);

        // ADSR buffer for Gain and Filter (shared for now)
        let adsr_buffer = self.adsr_buffer.get_sized_mut(frames_this_buffer);
        self.adsr_gain.process_buffer(adsr_buffer, sample_rate);

        // Gain
        self.gain.process_buffer(adsr_buffer, main_buffer);

        // Filter
        self.low_pass_filter.process_buffer(adsr_buffer, main_buffer, sample_rate);

        // Waveshaper
        self.waveshaper.process_buffer(main_buffer);

    }
}
