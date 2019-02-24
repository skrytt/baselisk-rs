extern crate dsp;

use defs;
use dsp::sample::slice;
use dsp::Sample;
use dsp::Frame;

pub struct Gain {
    amplitude: defs::Sample,
}

impl Gain
{
    pub fn new(
        amplitude: defs::Sample,
    ) -> Gain {
        Gain {
            amplitude,
        }
    }

    pub fn process_buffer(&mut self,
                          adsr_input_buffer: &defs::FrameBuffer,
                          output_buffer: &mut defs::FrameBuffer,
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
