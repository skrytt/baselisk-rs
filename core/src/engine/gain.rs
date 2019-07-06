extern crate sample;

use defs;
use sample::{Frame, Sample, slice};

pub fn process_buffer(adsr_input_buffer: &defs::MonoFrameBufferSlice,
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
            output_sample.mul_amp(adsr_input_sample)
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use defs;

    fn _test(amplitude: defs::Sample,
             expected_output: [defs::MonoFrame; 5])
    {
        let adsr_input_buffer: [defs::MonoFrame; 5] = [[amplitude]; 5];

        let mut output_buffer: [defs::MonoFrame; 5] = [[-1.0], [-0.5], [0.0], [0.5], [1.0]];
        process_buffer(&adsr_input_buffer, &mut output_buffer);

        for i in 0..5 {
            assert_eq!(output_buffer[i], expected_output[i]);
        }
    }

    #[test]
    fn test_unity_gain() {
        _test(1.0, [[-1.0], [-0.5], [0.0], [0.5], [1.0]]);
    }

    #[test]
    fn test_half_gain() {
        _test(0.5, [[-0.5], [-0.25], [0.0], [0.25], [0.5]]);
    }

    #[test]
    fn test_silence() {
        _test(0.0, [[0.0]; 5]);
    }
}
