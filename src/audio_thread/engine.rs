
use defs;
use dsp;
use dsp::sample::frame;
use event;
use processor::oscillator::Oscillator;

use std::rc::Rc;
use std::cell::RefCell;

pub struct Engine
{
    pub oscillator: Oscillator,
}

impl Engine
{
    pub fn new(event_buffer: &Rc<RefCell<event::Buffer>>) -> Engine {
        Engine{
            oscillator: Oscillator::new(event_buffer),
        }
    }

    pub fn audio_requested(&mut self,
                           output_buffer: &mut [frame::Mono<defs::Output>],
                           sample_rate: defs::Output)
    {
        // Zero the buffer
        dsp::slice::equilibrium(output_buffer);

        // Oscillator
        self.oscillator.update_state(sample_rate);
        self.oscillator.process_buffer(output_buffer, sample_rate);

        // Amplitude ADSR
        // TODO
        // Gain (TODO: make this driven by Amplitude ADSR)

        // Filter ADSR

        // Filter (driven by Filter ADSR)

    }
}
