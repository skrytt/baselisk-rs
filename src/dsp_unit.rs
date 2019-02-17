extern crate dsp;

use defs;
use processor;

/// DspUnit enumerates types that can feature in our DSP graph.
pub enum DspUnit<S>
where
    S: dsp::Sample + dsp::FromSample<f32>,
{
    Master,
    Processor(Box<dyn processor::Processor<S>>),
}

impl<S> DspUnit<S>
where
    S: dsp::Sample + dsp::FromSample<f32>,
{
    /// Set a parameter to a specified value.
    pub fn set_param(&mut self, param_name: String, param_val: String) -> Result<(), String> {
        match *self {
            DspUnit::Master => Err(String::from("Master node has no parameters")),
            DspUnit::Processor(ref mut processor) => processor.set_param(param_name, param_val),
        }
    }
}

/// A multiple-input, multiple-output audio processor.
trait MimoAudioProcessor {
    /// Use this method to request a processor takes its inputs and uses them to
    /// write values to the outputs.
    ///
    /// Input and output buffers are each references to vectors of buffers,
    /// where a buffer is a Box<[defs::Frame]> to allow for variable size of
    /// buffer, such that we can:
    ///
    /// 1. Have variable numbers of inputs/outputs to each node
    /// 2. Have variably-sized buffers
    /// 3. Mutate the buffers when required
    ///
    /// Implementors should:
    fn audio_requested(&mut self,
                       input_buffers: &Vec<Box<[defs::Frame]>>,
                       output_buffers: &mut Vec<Box<[defs::Frame]>>,
                       sample_rate: f64);
}

impl MimoAudioProcessor for DspUnit<defs::Output> {
    /// Request that a DspUnit type performs its processing on the mutable buffer.
    fn audio_requested(&mut self,
                       input_buffers: &Vec<Box<[defs::Frame]>>,
                       output_buffers: &mut Vec<Box<[defs::Frame]>>,
                       sample_rate: f64
    )
    {
        match *self {
            DspUnit::Master => (),
            DspUnit::Processor(ref mut processor) => {
                processor.update_state(sample_rate);
                // TODO: make this work
                panic!("TODO");
                //dsp::slice::map_in_place(buffer, |input_frame| {
                //    input_frame.map(|s| processor.process(s))
                //});
            }
        }
    }
}
