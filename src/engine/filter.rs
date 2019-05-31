extern crate sample;

use defs;
use event::EngineEvent;
use parameter::{
    BaseliskPluginParameters,
    PARAM_FILTER_FREQUENCY,
    PARAM_FILTER_SWEEP_RANGE,
    PARAM_FILTER_RESONANCE,
};
use sample::{Frame, slice};
use std::default::Default;
use vst::plugin::PluginParameters;

/// A low pass filter type that can be used for audio processing.
/// This is to be a constant-peak-gain two-pole resonator with
/// parameterized cutoff frequency and resonance.
pub struct Filter
{
    sample_rate: defs::Sample,
    last_adsr_input_sample_bits: u32,
    biquad_coefficient_func: Option<BiquadCoefficientGeneratorFunc>,
    history: BiquadSampleHistory,
    coeffs: BiquadCoefficients,
}

impl Filter
{
    /// Constructor for new Filter instances
    pub fn new() -> Self {
        Self {
            sample_rate: 0.0,
            last_adsr_input_sample_bits: 0,
            biquad_coefficient_func: Some(get_lowpass_second_order_biquad_consts),
            history: BiquadSampleHistory::new(),
            coeffs: BiquadCoefficients::new(),
        }
    }

    pub fn midi_panic(&mut self) {
        self.history.reset();
    }

    pub fn process_buffer(&mut self,
                          adsr_input_buffer: &defs::MonoFrameBufferSlice,
                          output_buffer: &mut defs::MonoFrameBufferSlice,
                          mut engine_event_iter: std::slice::Iter<(usize, EngineEvent)>,
                          sample_rate: defs::Sample,
                          params: &BaseliskPluginParameters) {
        self.sample_rate = sample_rate;

        // Calculate the output values per-frame
        let mut this_keyframe: usize = 0;
        let mut next_keyframe: usize;
        loop {
            // Get next selected note, if there is one.
            let next_event = engine_event_iter.next();

            // This block continues on events that are unimportant to this processor.
            if let Some((frame_num, engine_event)) = next_event {
                match engine_event {
                    EngineEvent::ModulateParameter { param_id, .. } => match *param_id {
                        // All filter events will trigger keyframes
                        PARAM_FILTER_FREQUENCY |
                        PARAM_FILTER_RESONANCE |
                        PARAM_FILTER_SWEEP_RANGE => (),
                        _ => continue,
                    },
                    _ => continue,
                }
                next_keyframe = *frame_num;
            } else {
                // No more note change events, so we'll process to the end of the buffer.
                next_keyframe = output_buffer.len();
            };

            // Apply the old parameters up until next_keyframe.
            {
                let output_buffer_slice = output_buffer.get_mut(
                        this_keyframe..next_keyframe).unwrap();
                let adsr_input_buffer_slice = adsr_input_buffer.get(
                        this_keyframe..next_keyframe).unwrap();

                let quality_factor = params.get_real_value(PARAM_FILTER_RESONANCE);
                let base_frequency_hz = params.get_real_value(PARAM_FILTER_FREQUENCY);
                let adsr_sweep_octaves = params.get_real_value(PARAM_FILTER_SWEEP_RANGE);

                // This forces the biquad coefficients to be computed at least once this slice:
                self.last_adsr_input_sample_bits = u32::max_value();

                if let Some(biquad_coefficient_func) = self.biquad_coefficient_func {
                    // Iterate over two buffer slices at once using a zip method
                    slice::zip_map_in_place(output_buffer_slice, adsr_input_buffer_slice,
                                            |output_frame, adsr_input_frame|
                    {
                        // Iterate over the samples in each frame using a zip method
                        output_frame.zip_map(adsr_input_frame,
                                             |sample, adsr_input_sample|
                        {
                            // Optimization: don't recompute the coefficients if they haven't changed
                            // since last iteration.
                            let adsr_input_sample_bits = adsr_input_sample.to_bits();
                            if self.last_adsr_input_sample_bits != adsr_input_sample_bits {
                                self.last_adsr_input_sample_bits = adsr_input_sample_bits;

                                // Use adsr_input (0 <= x <= 1) to determine the influence
                                // of params.adsr_sweep_octaves on the filter frequency.
                                let frequency_hz = base_frequency_hz
                                    * defs::Sample::exp2(adsr_sweep_octaves * adsr_input_sample);

                                (biquad_coefficient_func)(
                                        frequency_hz, quality_factor, self.sample_rate, &mut self.coeffs);
                            }

                            process_biquad(
                                &mut self.history,
                                &self.coeffs,
                                sample)
                        })
                    });
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
                if let EngineEvent::ModulateParameter { param_id, value } = event {
                   match *param_id {
                        PARAM_FILTER_FREQUENCY |
                        PARAM_FILTER_RESONANCE |
                        PARAM_FILTER_SWEEP_RANGE => {
                            params.set_parameter(*param_id, *value);
                        },
                        _ => (),
                    }
                };
            }
        }
    }
}

#[derive(Default)]
pub struct BiquadCoefficients {
    b0: defs::Sample,
    b1: defs::Sample,
    b2: defs::Sample,
    negative_a1: defs::Sample,
    negative_a2: defs::Sample,
}

impl BiquadCoefficients {
    pub fn new() -> Self {
        Self::default()
    }
}


#[derive(Default)]
pub struct BiquadSampleHistory {
    x0: defs::Sample,
    x1: defs::Sample,
    x2: defs::Sample,
    y1: defs::Sample,
    y2: defs::Sample,
}

impl BiquadSampleHistory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.x0 = 0.0;
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

/// Process a biquad frame.
/// Input consts must have be normalised such that a0 == 1.0,
/// by dividing a0 from all other "a" and "b" consts.
///
/// The general biquad implementation for Direct Form 1:
///
/// y_0 = b0 * x_0 + b1 * x_1 + b2 * x_2
///                - a1 * y_1 - a2 * y_2
pub fn process_biquad(history: &mut BiquadSampleHistory,
                      coeffs: &BiquadCoefficients,
                      input_sample: defs::Sample) -> defs::Sample
{
    // Update input ringbuffer with the new x:
    history.x2 = history.x1;
    history.x1 = history.x0;
    history.x0 = input_sample;

    // Calculate the output
    let mut output_sample = coeffs.b0 * history.x0;
    output_sample += coeffs.b1 * history.x1;
    output_sample += coeffs.b2 * history.x2;
    output_sample += coeffs.negative_a1 * history.y1;
    output_sample += coeffs.negative_a2 * history.y2;

    // Avoid arithmetic underflow by rounding very small values to zero.
    // This improves performance during periods of silence in the input.
    if output_sample.abs() < 1.0e-25 {
        output_sample = 0.0;
    }

    // Update output ringbuffer and return the output
    history.y2 = history.y1;
    history.y1 = output_sample;

    output_sample
}

/// Accepts: frequency_hz, quality_factor, sample_rate.
/// Returns a BiquadCoefficients.
type BiquadCoefficientGeneratorFunc = fn (defs::Sample,
                                          defs::Sample,
                                          defs::Sample,
                                          &mut BiquadCoefficients);

pub fn get_lowpass_second_order_biquad_consts(frequency_hz: defs::Sample,
                                          quality_factor: defs::Sample,
                                          sample_rate: defs::Sample,
                                          coeffs: &mut BiquadCoefficients)
{
    // Limit frequency_hz to just under half of the sample rate for stability.
    let frequency_hz = frequency_hz.min(0.495 * sample_rate);

    // Intermediate variables:
    let two_q_inv = 1.0 / (2.0 * quality_factor);
    let theta_c = defs::TWOPI * frequency_hz / sample_rate;
    let cos_theta_c = theta_c.cos();
    let sin_theta_c = theta_c.sin();
    let alpha = sin_theta_c * two_q_inv;

    // Calculate the coefficients.
    // a0 was divided off from each one to save on computation.
    let a0_inv = 1.0 / (1.0 + alpha);

    coeffs.negative_a1 = 2.0 * cos_theta_c * a0_inv;
    coeffs.negative_a2 = (alpha - 1.0) * a0_inv;

    coeffs.b1 = (1.0 - cos_theta_c) * a0_inv;
    coeffs.b0 = 0.5 * coeffs.b1;
    coeffs.b2 = coeffs.b0; // b2 = b0

}

pub fn get_highpass_second_order_biquad_consts(frequency_hz: defs::Sample,
                                          quality_factor: defs::Sample,
                                          sample_rate: defs::Sample,
                                          coeffs: &mut BiquadCoefficients)
{
    // Limit frequency_hz to just under half of the sample rate for stability.
    let frequency_hz = frequency_hz.min(0.495 * sample_rate);

    // Intermediate variables:
    let theta_c = 2.0 * defs::PI * frequency_hz / sample_rate;
    let cos_theta_c = theta_c.cos();
    let sin_theta_c = theta_c.sin();
    let alpha = sin_theta_c / (2.0 * quality_factor);

    // Calculate the coefficients.
    // a0 was divided off from each one to save on computation.
    let a0 = 1.0 + alpha;

    coeffs.negative_a1 = 2.0 * cos_theta_c / a0;
    coeffs.negative_a2 = (alpha - 1.0) / a0;

    coeffs.b0 = (1.0 + cos_theta_c) / (2.0 * a0);
    coeffs.b1 = -(1.0 + cos_theta_c) / a0;
    coeffs.b2 = coeffs.b0;
}
