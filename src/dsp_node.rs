
extern crate dsp;

use defs;
use generator;

/// DspNode enumerates types that can feature in our DSP graph.
pub enum DspNode<S>
where
    S: dsp::Sample + dsp::FromSample<f32>,
{
    Synth,
    Oscillator(Box<dyn generator::Generator<S>>),
}

impl dsp::Node<[defs::Output; defs::CHANNELS]> for DspNode<defs::Output> {
    fn audio_requested(
        &mut self,
        buffer: &mut [[defs::Output; defs::CHANNELS]],
        _sample_hz: f64
)
{
        match *self {
            DspNode::Synth => (),
            DspNode::Oscillator(ref mut oscillator) => {
                dsp::slice::map_in_place(buffer, |_| {
                    oscillator.update_state();
                    dsp::Frame::from_fn(|_| oscillator.generate())
                });
            }
        }
    }
}
