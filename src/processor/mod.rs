
pub mod gain;
pub mod oscillator;

extern crate dsp;

use std::fmt;
use std::sync::{Arc, RwLock};
use event;

pub trait ProcessorView {
    fn name(&self) -> String;

    fn details(&self) -> String;

    #[allow(unused_variables)]
    fn set_param(&mut self, param_name: String, param_val: String) -> Result<(), String>{
        // Default implementation
        Err(String::from("This processor has no settable parameters"))
    }
}

pub trait Processor<S>: ProcessorView {
    fn update_state(&mut self, sample_rate: f64);

    fn process(&mut self, input: S) -> S
    where
        S: dsp::sample::FloatSample + dsp::FromSample<f32> + fmt::Display;
}

pub fn new_source<S>(
    name: &str,
    event_buffer: Arc<RwLock<event::Buffer>>,
) -> Result<Box<dyn Processor<S>>, &'static str>
where
    S: dsp::Sample + dsp::FromSample<f32> + fmt::Display + 'static,
{
    match name {
        "sine" | "saw" | "square" => oscillator::new(name, event_buffer),
        _ => return Err("Unknown source name"),
    }
}

pub fn new_processor<S>(
    name: &str,
    event_buffer: Arc<RwLock<event::Buffer>>,
) -> Result<Box<dyn Processor<S>>, &'static str> {
    match name {
        "adsrgain" => gain::new(name, event_buffer),
        _ => Err("Unknown processor name"),
    }
}
