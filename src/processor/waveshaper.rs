
extern crate dsp;

use defs;
use dsp::sample::frame;
use dsp::sample::slice;
use dsp::Frame;

pub struct Waveshaper {
    input_gain: defs::Output,
}

impl Waveshaper {
    pub fn new() -> Waveshaper {
        Waveshaper{
            input_gain: 0.7,
        }
    }

    pub fn process_buffer(&mut self, output_buffer: &mut [frame::Mono<defs::Output>])
    {
        slice::map_in_place(output_buffer, |output_frame| {
            output_frame.map(|output_sample| {
                // Polynomial: -x^3 + x^2 + x
                // With input and output gain scaling
                let x = output_sample.abs() * self.input_gain;
                output_sample.signum() * (
                    -x.powi(3) + x.powi(2) + x)
            })
        })
    }
}
