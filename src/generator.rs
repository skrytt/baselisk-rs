
extern crate dsp;

use std::fmt;

/// An oscillator must implement the Oscillator trait
pub trait Generator<S> {
    fn type_name(&self) -> &'static str;

    fn update_state(&mut self);

    fn generate(&mut self) -> S
    where S: dsp::Sample + dsp::FromSample<f32> + fmt::Display;
}

