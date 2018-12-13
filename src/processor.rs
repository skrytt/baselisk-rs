extern crate dsp;

use std::fmt;

pub trait Processor<S> {
    fn name(&self) -> String;

    fn details(&self) -> String;

    #[allow(unused_variables)]
    fn set_param(&mut self, param_name: String, param_val: String) -> Result<(), String>{
        // Default implementation
        Err(String::from("This processor has no settable parameters"))
    }

    fn update_state(&mut self, sample_rate: f64);

    fn process(&mut self, input: S) -> S
    where
        S: dsp::sample::FloatSample + dsp::FromSample<f32> + fmt::Display;
}

