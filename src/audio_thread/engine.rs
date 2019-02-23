
use dsp;
use dsp::sample::{frame, FloatSample, FromSample};
use event;
use processor::Processor;
use processor::oscillator::Oscillator;

use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

pub struct Engine<S>
where
    S: FloatSample + FromSample<f32> + fmt::Display + 'static,
{
    pub oscillator: Oscillator<S>,
}

impl<S> Engine<S>
where
    S: FloatSample + FromSample<f32> + fmt::Display,
{
    pub fn new(event_buffer: &Rc<RefCell<event::Buffer>>) -> Engine<S> {
        Engine{
            oscillator: Oscillator::new(event_buffer),
        }
    }

    pub fn audio_requested(&mut self, output_buffer: &mut [frame::Mono<S>], sample_rate: f64) {
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
