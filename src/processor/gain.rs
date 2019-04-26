extern crate sample;

use defs;
use sample::{Frame, Sample, slice};

pub struct Gain {
    amplitude: defs::Sample,
}

impl Gain
{
    pub fn new(
        amplitude: defs::Sample,
    ) -> Self {
        Self {
            amplitude,
        }
    }

    pub fn process_buffer(&mut self,
                          adsr_input_buffer: &defs::MonoFrameBufferSlice,
                          output_buffer: &mut defs::MonoFrameBufferSlice,
    )
    {
        // Iterate over two buffers at once using a zip method
        slice::zip_map_in_place(output_buffer, adsr_input_buffer,
                                |output_frame, adsr_input_frame|
        {
            // Iterate over the samples in each frame using a zip method
            output_frame.zip_map(adsr_input_frame,
                                 |output_sample, adsr_input_sample| {
                output_sample.mul_amp(self.amplitude * adsr_input_sample)
            })
        })
    }
}
