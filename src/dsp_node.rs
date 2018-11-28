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
    Synth,
    Source(Box<dyn processor::Source<S>>),
    Processor(Box<dyn processor::Processor<S>>),
}

impl dsp::Node<defs::Frame> for DspNode<defs::Output> {
    fn audio_requested(&mut self, buffer: &mut [defs::Frame], _sample_hz: f64) {
        match *self {
            DspNode::Synth => (),
            DspNode::Source(ref mut source) => {
                source.update_state();
                dsp::slice::map_in_place(buffer, |_| dsp::Frame::from_fn(|_| source.generate()));
            }
            DspNode::Processor(ref mut processor) => {
                processor.update_state();
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
            DspNode::Synth => write!(f, "Synth"),
            DspNode::Source(ref t) => write!(f, "{}", t.type_name()),
            DspNode::Processor(ref t) => write!(f, "{}", t.type_name()),
        }
    }
}
