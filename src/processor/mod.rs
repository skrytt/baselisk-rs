pub mod gain;
pub mod modulator;
pub mod oscillator;
pub mod filter;

extern crate dsp;

use event;
use processor;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

/// ProcessorView: represents functionality that views representing Processor types
/// should implement.
pub trait ProcessorView {
    fn name(&self) -> String;

    fn details(&self) -> String {
        // Default implementation
        String::from("(no parameters)")
    }

    #[allow(unused_variables)]
    fn set_param(&mut self, param_name: String, param_val: String) -> Result<(), String> {
        // Default implementation
        Err(String::from("This processor has no settable parameters"))
    }
}

/// Processor: represents functionality that actual Processor types should implement.
pub trait Processor<S>: ProcessorView {
    fn update_state(&mut self, sample_rate: f64);

    fn process(&mut self, input: S) -> S
    where
        S: dsp::sample::FloatSample + dsp::FromSample<f32> + fmt::Display;

    fn get_view(&self) -> Box<dyn processor::ProcessorView>;
}

/// Create a new source from a given name.
/// This function can create processors supported by the "add" command.
/// there is no implementation difference between sources and other processors;
/// however, generally they are things like oscillators used for sound generation,
/// that will have no nodes before them in the graph.
pub fn new<S>(
    name: &str,
    event_buffer: Rc<RefCell<event::Buffer>>,
) -> Result<Box<dyn Processor<S>>, &'static str>
where
    S: dsp::sample::FloatSample + dsp::FromSample<f32> + fmt::Display + 'static,
    f32: dsp::FromSample<S>,
{
    match name {
        "sine" | "saw" | "square" => oscillator::new(name, event_buffer),
        "adsrgain"                => gain::new(name, event_buffer),
        "lowpass_simple"          => filter::new(name, event_buffer),
        _ => return Err("Unknown source name"),
    }
}
