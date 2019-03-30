extern crate sample;

use defs;
use parameter::{Parameter, LinearParameter};
use sample::{Frame, slice};

pub struct Waveshaper {
    input_gain: LinearParameter,
    output_gain: LinearParameter,
}

impl Waveshaper {
    pub fn new() -> Waveshaper {
        Waveshaper{
            input_gain: LinearParameter::new(0.333),
            output_gain: LinearParameter::new(0.667),
        }
    }

    pub fn set_input_gain(&mut self, gain: defs::Sample) -> Result<(), &'static str> {
        if gain <= 0.0 || gain >= 1.0 {
            return Err("Gain must be in the range 0.0 <= gain <= 1.0")
        }
        self.input_gain.set_base(gain);
        Ok(())
    }

    pub fn set_output_gain(&mut self, gain: defs::Sample) -> Result<(), &'static str> {
        if gain <= 0.0 || gain >= 1.0 {
            return Err("Gain must be in the range 0.0 <= gain <= 1.0")
        }
        self.output_gain.set_base(gain);
        Ok(())
    }

    pub fn process_buffer(&mut self, output_buffer: &mut defs::MonoFrameBufferSlice)
    {
        slice::map_in_place(output_buffer, |output_frame| {
            output_frame.map(|output_sample| {
                // Polynomial: -x^3 + x^2 + x
                // With input and output gain scaling
                let x = output_sample.abs() * self.input_gain.get();
                self.output_gain.get() * output_sample.signum() * (
                    -x.powi(3) + x.powi(2) + x)
            })
        })
    }
}
