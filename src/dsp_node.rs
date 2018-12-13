extern crate dsp;

use defs;
use dsp::Frame;
use processor;
use std::fmt;

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
    pub fn set_param(&mut self, param_name: String, param_val: String) {
        match *self {
            DspNode::Master => (),
            DspNode::Processor(ref mut processor) => {
                processor.set_param(param_name, param_val).unwrap();
            }
        }

    }
}

impl dsp::Node<defs::Frame> for DspNode<defs::Output> {
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

impl fmt::Display for DspNode<defs::Output> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DspNode::Master => write!(f, "master"),
            DspNode::Processor(ref t) => write!(f, "{} {}", t.name(), t.details()),
        }
    }
}
