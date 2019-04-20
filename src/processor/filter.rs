extern crate sample;

use buffer::ResizableFrameBuffers;
use defs;
use event::{EngineEvent, ModulatableParameter, ModulatableParameterUpdateData};
use parameter::{Parameter, FrequencyParameter, LinearParameter};

/// Parameters available for filters.
struct FilterParams {
    frequency: FrequencyParameter,
    adsr_sweep_octaves: LinearParameter,
    quality_factor: LinearParameter,
}

impl FilterParams {
    /// Constructor for FilterParams instances
    fn new() -> FilterParams {
        FilterParams {
            frequency: FrequencyParameter::new(1.0, 22000.0, 10.0),
            adsr_sweep_octaves: LinearParameter::new(0.0, 20.0, 6.5),
            quality_factor: LinearParameter::new(0.5, 10.0, 0.707),
        }
    }
}

enum BufferEnum {
    Frequency,
    Quality,
    TwoQInv,
    ThetaC,
    CosThetaC,
    SinThetaC,
    Alpha,
    A0Inv,
    A1,
    A2,
    B0,
    B1,
    B2,
    Output,
}
const NUM_BUFFERS: usize = 15;

/// A low pass filter type that can be used for audio processing.
/// This is to be a constant-peak-gain two-pole resonator with
/// parameterized cutoff frequency and resonance.
pub struct Filter
{
    params: FilterParams,
    buffers: ResizableFrameBuffers<defs::MonoFrame>,
    biquad_coefficient_generator_func: BiquadCoefficientGeneratorFunc,
    x0: defs::Sample,
    x1: defs::Sample,
    x2: defs::Sample,
    y1: defs::Sample,
    y2: defs::Sample,
}

impl Filter
{
    /// Constructor for new Filter instances
    pub fn new() -> Filter {
        Filter {
            params: FilterParams::new(),
            buffers: ResizableFrameBuffers::new(NUM_BUFFERS),
            biquad_coefficient_generator_func: get_lowpass_second_order_biquad_consts,
            x0: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    pub fn resize_buffers(&mut self, size: usize) {
        self.buffers.resize(size);
    }

    pub fn midi_panic(&mut self) {
        panic!("midi panic needs reimplementation for filter module");
    }

    /// Set frequency (Hz) for this filter.
    /// Note: frequency will always be limited to the Nyquist frequency,
    /// a function of the sample rate, even if this parameter is higher.
    pub fn update_frequency(&mut self, data: ModulatableParameterUpdateData)
                            -> Result<(), &'static str>
    {
        self.params.frequency.update_patch(data)
    }

    /// Set sweep range (octaves) for this filter.
    pub fn update_sweep(&mut self, data: ModulatableParameterUpdateData)
                        -> Result<(), &'static str>
    {
        self.params.adsr_sweep_octaves.update_patch(data)
    }

    /// Set quality for this filter.
    pub fn update_quality(&mut self, data: ModulatableParameterUpdateData)
                          -> Result<(), &'static str>
    {
        self.params.quality_factor.update_patch(data)
    }

    pub fn process_buffer(&mut self,
                          adsr_input_buffer: &defs::MonoFrameBufferSlice,
                          output_buffer: &mut defs::MonoFrameBufferSlice,
                          mut engine_event_iter: std::slice::Iter<(usize, EngineEvent)>,
                          sample_rate: defs::Sample) {

        let buffer_size = output_buffer.len();

        // Calculate the frequency and quality values per-frame
        {
            let all_buffers = self.buffers.get_mut();

            let (frequency_buffer, remaining) = all_buffers.split_at_mut(buffer_size);
            let (quality_buffer, _) = remaining.split_at_mut(buffer_size);

            let mut this_keyframe: usize = 0;
            let mut next_keyframe: usize;
            loop {
                // Get next selected note, if there is one.
                let next_event = engine_event_iter.next();

                // This match block continues on events that are unimportant to this processor.
                match next_event {
                    Some((frame_num, engine_event)) => {
                        match engine_event {
                            EngineEvent::ModulateParameter { parameter, .. } => match parameter {
                                ModulatableParameter::FilterFrequency => (),
                                ModulatableParameter::FilterQuality => (),
                                ModulatableParameter::FilterSweepRange => (),
                                _ => continue,
                            },
                            _ => continue,
                        }
                        next_keyframe = *frame_num;
                    },
                    None => {
                        // No more note change events, so we'll process to the end of the buffer.
                        next_keyframe = output_buffer.len();
                    },
                };

                // Apply the old parameters up until next_keyframe.
                {
                    // Frequency
                    let base_frequency_hz = self.params.frequency.get();
                    let adsr_sweep_octaves = self.params.adsr_sweep_octaves.get();

                    let adsr_buffer_slice = adsr_input_buffer.get(
                            this_keyframe..next_keyframe).unwrap();
                    let frequency_buffer_slice = frequency_buffer.get_mut(
                            this_keyframe..next_keyframe).unwrap();

                    for (frame_num, frame) in frequency_buffer_slice.iter_mut().enumerate() {
                        // Use adsr_input (0 <= x <= 1) to determine the influence
                        // of self.params.adsr_sweep_octaves on the filter frequency.
                        // Limit to just under the Nyquist frequency for stability.
                        *frame = [(base_frequency_hz
                            * defs::Sample::exp2(adsr_sweep_octaves * adsr_buffer_slice[frame_num][0]))
                              .min(0.495 * sample_rate)];
                    };

                    // Quality
                    let quality_factor = self.params.quality_factor.get();
                    let quality_buffer_slice = quality_buffer.get_mut(
                            this_keyframe..next_keyframe).unwrap();
                    for frame in quality_buffer_slice.iter_mut() {
                        *frame = [quality_factor];
                    }

                } // output_buffer_slice exits scope

                // We've reached the next_keyframe.
                this_keyframe = next_keyframe;

                // What we do now depends on whether we reached the end of the buffer.
                if this_keyframe == output_buffer.len() {
                    // Loop exit condition: reached the end of the buffer.
                    break
                } else {
                    // Before the next iteration, use the event at this keyframe
                    // to update the current state.
                    let (_, event) = next_event.unwrap();
                    match event {
                        EngineEvent::ModulateParameter { parameter, value } => match parameter {
                            ModulatableParameter::FilterFrequency => {
                                self.params.frequency.update_cc(*value);
                            },
                            ModulatableParameter::FilterQuality => {
                                self.params.quality_factor.update_cc(*value);
                            }
                            ModulatableParameter::FilterSweepRange => {
                                self.params.adsr_sweep_octaves.update_cc(*value);
                            },
                            _ => (),
                        },
                        _ => (),
                    };
                }
            }
        } // all_buffers exits scope

        // Once we're here, we have frequency and quality values for all samples.
        // We need to do a little more work to get all the coefficients for the samples too.
        (self.biquad_coefficient_generator_func)(&mut self.buffers, buffer_size, sample_rate);

        // Finally we can compute the output values.
        let all_buffers = self.buffers.get_mut();
        let (_, remaining) = all_buffers.split_at_mut(
            (BufferEnum::A1 as usize) * buffer_size);
        let (a1_buffer, remaining) = remaining.split_at_mut(buffer_size);
        let (a2_buffer, remaining) = remaining.split_at_mut(buffer_size);
        let (b0_buffer, remaining) = remaining.split_at_mut(buffer_size);
        let (b1_buffer, remaining) = remaining.split_at_mut(buffer_size);
        let (b2_buffer, remaining) = remaining.split_at_mut(buffer_size);
        let (temp_input_buffer, remaining) = remaining.split_at_mut(buffer_size);
        let (temp_output_buffer, _) = remaining.split_at_mut(buffer_size);

        // Copy the input from the real input buffer to temp_input_buffer
        sample::slice::write(temp_input_buffer, output_buffer);

        let mut x0 = self.x0;
        let mut x1 = self.x1;
        let mut x2 = self.x2;
        let mut y1 = self.y1;
        let mut y2 = self.y2;

        for frame_num in 0..buffer_size {
            // Update input ringbuffer with the new x:
            x2 = x1;
            x1 = x0;
            x0 = temp_input_buffer[frame_num][0];

            // Calculate the output
            let temp_output_sample: &mut defs::Sample = &mut temp_output_buffer[frame_num][0];
            *temp_output_sample = b0_buffer[frame_num][0] * x0;
            *temp_output_sample += b1_buffer[frame_num][0] * x1;
            *temp_output_sample += b2_buffer[frame_num][0] * x2;
            *temp_output_sample -= a1_buffer[frame_num][0] * y1;
            *temp_output_sample -= a2_buffer[frame_num][0] * y2;

            // Update output ringbuffer and return the output
            y2 = y1;
            y1 = *temp_output_sample;
        }

        self.x0 = x0;
        self.x1 = x1;
        self.x2 = x2;
        self.y1 = y1;
        self.y2 = y2;


        // Copy the results from temp_output_buffer to the real output buffer
        sample::slice::write(output_buffer, temp_output_buffer);
    }
}

/// Accepts: a ResizableFrameBuffers (must have 12 buffers), a buffer size and a sample rate.
/// Returns a BiquadCoefficients.
type BiquadCoefficientGeneratorFunc = fn (&mut ResizableFrameBuffers<defs::MonoFrame>,
                                          usize,
                                          defs::Sample);

pub fn get_lowpass_second_order_biquad_consts(buffers: &mut ResizableFrameBuffers<defs::MonoFrame>,
                                              buffer_size: usize,
                                              sample_rate: defs::Sample)
{
    let all_buffers = buffers.get_mut();
    let (frequency_buffer, remaining) = all_buffers.split_at_mut(buffer_size);
    let (quality_buffer, remaining) = remaining.split_at_mut(buffer_size);
    let (two_q_inv_buffer, remaining) = remaining.split_at_mut(buffer_size);
    let (theta_c_buffer, remaining) = remaining.split_at_mut(buffer_size);
    let (cos_theta_c_buffer, remaining) = remaining.split_at_mut(buffer_size);
    let (sin_theta_c_buffer, remaining) = remaining.split_at_mut(buffer_size);
    let (alpha_buffer, remaining) = remaining.split_at_mut(buffer_size);
    let (a0_inv_buffer, remaining) = remaining.split_at_mut(buffer_size);
    let (a1_buffer, remaining) = remaining.split_at_mut(buffer_size);
    let (a2_buffer, remaining) = remaining.split_at_mut(buffer_size);
    let (b0_buffer, remaining) = remaining.split_at_mut(buffer_size);
    let (b1_buffer, remaining) = remaining.split_at_mut(buffer_size);
    let (b2_buffer, _) = remaining.split_at_mut(buffer_size);

    // Intermediate variables:
    for (frame_num, frame) in two_q_inv_buffer.iter_mut().enumerate() {
        frame[0] = 0.5 / quality_buffer[frame_num][0];
    }
    for (frame_num, frame) in theta_c_buffer.iter_mut().enumerate() {
        frame[0] = defs::TWOPI * frequency_buffer[frame_num][0] / sample_rate;
    }
    for (frame_num, frame) in cos_theta_c_buffer.iter_mut().enumerate() {
        frame[0] = theta_c_buffer[frame_num][0].cos();
    }
    for (frame_num, frame) in sin_theta_c_buffer.iter_mut().enumerate() {
        frame[0] = theta_c_buffer[frame_num][0].sin();
    }
    for (frame_num, frame) in alpha_buffer.iter_mut().enumerate() {
        frame[0] = sin_theta_c_buffer[frame_num][0] * two_q_inv_buffer[frame_num][0];
    }
    for (frame_num, frame) in a0_inv_buffer.iter_mut().enumerate() {
        frame[0] = 1.0 / (1.0 + alpha_buffer[frame_num][0]);
    }

    // Calculate the coefficients.
    // a0 was divided off from each one to save on computation.
    for (frame_num, frame) in a1_buffer.iter_mut().enumerate() {
        frame[0] = -2.0 * cos_theta_c_buffer[frame_num][0] * a0_inv_buffer[frame_num][0];
    }
    for (frame_num, frame) in a2_buffer.iter_mut().enumerate() {
        frame[0] = (1.0 - alpha_buffer[frame_num][0]) * a0_inv_buffer[frame_num][0];
    }
    for (frame_num, frame) in b1_buffer.iter_mut().enumerate() {
        frame[0] = (1.0 - cos_theta_c_buffer[frame_num][0]) * a0_inv_buffer[frame_num][0];
    }
    for (frame_num, frame) in b0_buffer.iter_mut().enumerate() {
        frame[0] = 0.5 * b1_buffer[frame_num][0];
    }
    for (frame_num, frame) in b2_buffer.iter_mut().enumerate() {
        frame[0] = b0_buffer[frame_num][0];
    }
}

pub fn get_highpass_second_order_biquad_consts(buffers: &mut ResizableFrameBuffers<defs::MonoFrame>,
                                               buffer_size: usize,
                                               sample_rate: defs::Sample)
{
    let all_buffers = buffers.get_mut();
    let (frequency_buffer, remaining) = all_buffers.split_at_mut(buffer_size);
    let (quality_buffer, remaining) = remaining.split_at_mut(buffer_size);
    let (two_q_inv_buffer, remaining) = remaining.split_at_mut(buffer_size);
    let (theta_c_buffer, remaining) = remaining.split_at_mut(buffer_size);
    let (cos_theta_c_buffer, remaining) = remaining.split_at_mut(buffer_size);
    let (sin_theta_c_buffer, remaining) = remaining.split_at_mut(buffer_size);
    let (alpha_buffer, remaining) = remaining.split_at_mut(buffer_size);
    let (a0_inv_buffer, remaining) = remaining.split_at_mut(buffer_size);
    let (a1_buffer, remaining) = remaining.split_at_mut(buffer_size);
    let (a2_buffer, remaining) = remaining.split_at_mut(buffer_size);
    let (b0_buffer, remaining) = remaining.split_at_mut(buffer_size);
    let (b1_buffer, remaining) = remaining.split_at_mut(buffer_size);
    let (b2_buffer, _) = remaining.split_at_mut(buffer_size);

    // Intermediate variables:
    for (frame_num, frame) in two_q_inv_buffer.iter_mut().enumerate() {
        frame[0] = 0.5 / quality_buffer[frame_num][0];
    }
    for (frame_num, frame) in theta_c_buffer.iter_mut().enumerate() {
        frame[0] = defs::TWOPI * frequency_buffer[frame_num][0] / sample_rate;
    }
    for (frame_num, frame) in cos_theta_c_buffer.iter_mut().enumerate() {
        frame[0] = theta_c_buffer[frame_num][0].cos();
    }
    for (frame_num, frame) in sin_theta_c_buffer.iter_mut().enumerate() {
        frame[0] = theta_c_buffer[frame_num][0].sin();
    }
    for (frame_num, frame) in alpha_buffer.iter_mut().enumerate() {
        frame[0] = sin_theta_c_buffer[frame_num][0] * two_q_inv_buffer[frame_num][0];
    }
    for (frame_num, frame) in a0_inv_buffer.iter_mut().enumerate() {
        frame[0] = 1.0 / (1.0 + alpha_buffer[frame_num][0]);
    }

    // Calculate the coefficients.
    // a0 was divided off from each one to save on computation.
    for (frame_num, frame) in a1_buffer.iter_mut().enumerate() {
        frame[0] = -2.0 * cos_theta_c_buffer[frame_num][0] * a0_inv_buffer[frame_num][0];
    }
    for (frame_num, frame) in a2_buffer.iter_mut().enumerate() {
        frame[0] = (1.0 - alpha_buffer[frame_num][0]) * a0_inv_buffer[frame_num][0];
    }
    for (frame_num, frame) in b0_buffer.iter_mut().enumerate() {
        frame[0] = 0.5 * (1.0 + cos_theta_c_buffer[frame_num][0]) * a0_inv_buffer[frame_num][0];
    }
    for (frame_num, frame) in b1_buffer.iter_mut().enumerate() {
        frame[0] = -2.0 * b0_buffer[frame_num][0];
    }
    for (frame_num, frame) in b2_buffer.iter_mut().enumerate() {
        frame[0] = b0_buffer[frame_num][0];
    }
}
