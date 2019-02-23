
use defs;
use dsp;
use dsp::sample::frame;
use event;
use processor::oscillator::Oscillator;
use processor::gain::Gain;

use std::rc::Rc;
use std::cell::RefCell;

pub struct Engine
{
    pub oscillator: Oscillator,
    pub gain: Gain,
}

impl Engine
{
    pub fn new(event_buffer: &Rc<RefCell<event::Buffer>>) -> Engine {
        Engine{
            oscillator: Oscillator::new(event_buffer),
            gain: Gain::new(0.2),
        }
    }

    pub fn audio_requested(&mut self,
                           buffer: &mut [frame::Mono<defs::Output>],
                           sample_rate: defs::Output)
    {
        // Zero the buffer
        dsp::slice::equilibrium(buffer);

        // Oscillator
        self.oscillator.update_state(sample_rate);
        self.oscillator.process_buffer(buffer, sample_rate);

        // Amplitude ADSR
        // TODO

        // Gain (TODO: make this driven by Amplitude ADSR)
        self.gain.process_buffer(buffer);

        // Filter ADSR

        // Filter (driven by Filter ADSR)

    }
}
