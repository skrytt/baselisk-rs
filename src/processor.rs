extern crate dsp;

use std::fmt;

/// Any audio processor that only outputs audio must implement the Source trait
pub trait Source<S> {
    fn name(&self) -> &str;

    fn update_state(&mut self, sample_rate: f64);

    fn generate(&mut self) -> S
    where
        S: dsp::Sample + dsp::FromSample<f32> + fmt::Display;
}

/// Any audio processor that both inputs and outputs audio must implement the Processor trait
pub trait Processor<S> {
    fn type_name(&self) -> &'static str;

    fn update_state(&mut self, sample_rate: f64);

    fn process(&mut self, input: S) -> S
    where
        S: dsp::sample::FloatSample + dsp::FromSample<f32> + fmt::Display;
}
