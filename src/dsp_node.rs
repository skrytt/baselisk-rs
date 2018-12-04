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
    Source(Box<dyn processor::Source<S>>),
    Processor(Box<dyn processor::Processor<S>>),
}

impl dsp::Node<defs::Frame> for DspNode<defs::Output> {
    fn audio_requested(&mut self, buffer: &mut [defs::Frame], sample_rate: f64) {
        match *self {
            DspNode::Master => (),
            DspNode::Source(ref mut source) => {
                source.update_state(sample_rate);
                dsp::slice::map_in_place(buffer, |_| dsp::Frame::from_fn(|_| source.generate()));
            }
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
            DspNode::Source(ref t) => write!(f, "{}", t.name()),
            DspNode::Processor(ref t) => write!(f, "{}", t.type_name()),
        }
    }
}
