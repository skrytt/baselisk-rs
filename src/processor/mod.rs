
// TODO: fix all these to use MimoAudioProcessor trait
pub mod oscillator;
//pub mod gain;
//pub mod modulator;
//pub mod filter;

extern crate dsp;

/// Processor: represents functionality that actual Processor types should implement.
pub trait Processor<S> {
    /// Processor::name: should return the name of the DSP unit.
    fn name(&self) -> String;

    /// Processor::set_param: accepts a name-value pair as parameters,
    /// and if these match a parameter, should try to set the parameter value.
    /// The returned Result value should indicate whether this was successful.
    fn set_param(&mut self, _param_name: String, _param_val: String) -> Result<(), String> {
        // Default implementation
        Err(String::from("This processor has no settable parameters"))
    }

    /// Processor::update_state: perform any updates prior to audio processing.
    fn update_state(&mut self, sample_rate: f64);
}
