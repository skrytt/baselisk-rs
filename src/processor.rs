extern crate dsp;

use std::fmt;

pub trait Processor<S> {
    fn name(&self) -> String;

    fn details(&self) -> String;

    fn update_state(&mut self, sample_rate: f64);

    fn process(&mut self, input: S) -> S
    where
        S: dsp::sample::FloatSample + dsp::FromSample<f32> + fmt::Display;
}
