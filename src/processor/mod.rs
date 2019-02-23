
// TODO: fix all these to use MimoAudioProcessor trait
pub mod oscillator;
//pub mod gain;
//pub mod modulator;
//pub mod filter;

extern crate dsp;

/// Processor: represents functionality that actual Processor types should implement.
pub trait Processor<S> {
    fn name(&self) -> String;

    fn set_param(&mut self, _param_name: String, _param_val: String) -> Result<(), String> {
        // Default implementation
        Err(String::from("This processor has no settable parameters"))
    }

    fn update_state(&mut self, sample_rate: f64);
}
