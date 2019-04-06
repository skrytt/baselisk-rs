extern crate sample;

use defs;
use event::ModulatableParameterUpdateData;
use parameter::{Parameter, LinearParameter};
use sample::{Frame, slice};

pub struct Waveshaper {
    input_gain: LinearParameter,
    output_gain: LinearParameter,
}

impl Waveshaper {
    pub fn new() -> Waveshaper {
        Waveshaper{
            input_gain: LinearParameter::new(0.0, 1.0, 0.333),
            output_gain: LinearParameter::new(0.0, 1.0, 0.667),
        }
    }

    pub fn update_input_gain(&mut self, data: ModulatableParameterUpdateData)
                          -> Result<(), &'static str> {
        self.input_gain.update_patch(data)
    }

    pub fn update_output_gain(&mut self, data: ModulatableParameterUpdateData)
                           -> Result<(), &'static str> {
        self.output_gain.update_patch(data)
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
