extern crate dsp;

use defs;
use dsp::Frame;
use processor;

/// DspNode enumerates types that can feature in our DSP graph.
pub enum DspNode<S>
where
    S: dsp::Sample + dsp::FromSample<f32>,
{
    Master,
    Processor(Box<dyn processor::Processor<S>>),
}

impl<S> DspNode<S>
where
    S: dsp::Sample + dsp::FromSample<f32>,
{
    /// Set a parameter to a specified value.
    pub fn set_param(&mut self, param_name: String, param_val: String) -> Result<(), String> {
        match *self {
            DspNode::Master => Err(String::from("Master node has no parameters")),
            DspNode::Processor(ref mut processor) => processor.set_param(param_name, param_val),
        }
    }
}

impl dsp::Node<defs::Frame> for DspNode<defs::Output> {
    /// Request that a DspNode type performs its processing on the mutable buffer.
    fn audio_requested(&mut self, buffer: &mut [defs::Frame], sample_rate: f64) {
        match *self {
            DspNode::Master => (),
            DspNode::Processor(ref mut processor) => {
                processor.update_state(sample_rate);
                dsp::slice::map_in_place(buffer, |input_frame| {
                    input_frame.map(|s| processor.process(s))
                });
            }
        }
    }
}
