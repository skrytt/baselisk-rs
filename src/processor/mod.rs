
// TODO: fix all these to use MimoAudioProcessor trait
pub mod oscillator;
//pub mod gain;
//pub mod modulator;
//pub mod filter;

extern crate dsp;

use processor;

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

    fn get_view(&self) -> Box<dyn processor::ProcessorView>;
}
