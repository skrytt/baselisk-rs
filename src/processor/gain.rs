extern crate dsp;

use defs;
use dsp::sample::frame;
use dsp::Sample;

pub struct Gain {
    amplitude: defs::Output,
}

impl Gain
{
    pub fn new(
        amplitude: defs::Output,
    ) -> Gain {
        Gain {
            amplitude,
        }
    }

    pub fn process_buffer(&mut self,
                          buffer: &mut [frame::Mono<defs::Output>]
    )
    {
        for frame in buffer.iter_mut() {
            for sample in frame.iter_mut() {
                sample.mul_amp(self.amplitude);
            }
        }
    }
}
