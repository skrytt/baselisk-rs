
use audio_thread::buffer::Buffer;
use defs;
use dsp;
use dsp::sample::frame;
use event;
use processor::oscillator::Oscillator;
use processor::modulator::Adsr;
use processor::gain::Gain;

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
            gain: Gain::new(0.2),
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

        // Amplitude ADSR
        let adsr_buffer = self.adsr_buffer.get_sized_mut(frames_this_buffer);
        self.adsr_gain.process_buffer(adsr_buffer, sample_rate);

        // Gain (TODO: make this driven by Amplitude ADSR)
        self.gain.process_buffer(main_buffer);

        // Filter ADSR

        // Filter (driven by Filter ADSR)

    }
}
