extern crate sample;

use defs;
use sample::{Frame, slice};

pub struct Waveshaper {
    input_gain: defs::Sample,
    output_gain: defs::Sample,
}

impl Waveshaper {
    pub fn new() -> Waveshaper {
        Waveshaper{
            input_gain: 0.333,
            output_gain: 0.667,
        }
    }

    pub fn set_input_gain(&mut self, gain: defs::Sample) -> Result<(), &'static str> {
        if gain <= 0.0 || gain >= 1.0 {
            return Err("Gain must be in the range 0.0 <= gain <= 1.0")
        }
        self.input_gain = gain;
        Ok(())
    }

    pub fn set_output_gain(&mut self, gain: defs::Sample) -> Result<(), &'static str> {
        if gain <= 0.0 || gain >= 1.0 {
            return Err("Gain must be in the range 0.0 <= gain <= 1.0")
        }
        self.output_gain = gain;
        Ok(())
    }

    pub fn process_buffer(&mut self, output_buffer: &mut defs::FrameBuffer)
    {
        slice::map_in_place(output_buffer, |output_frame| {
            output_frame.map(|output_sample| {
                // Polynomial: -x^3 + x^2 + x
                // With input and output gain scaling
                let x = output_sample.abs() * self.input_gain;
                self.output_gain * output_sample.signum() * (
                    -x.powi(3) + x.powi(2) + x)
            })
        })
    }
}
