
extern crate dsp;
extern crate portmidi;

use std::fmt;

/// An oscillator must implement the Oscillator trait
pub trait Generator<S> {
    fn update_state(&mut self);

    fn generate(&mut self) -> S
    where S: dsp::Sample + dsp::FromSample<f32> + fmt::Display;
}

