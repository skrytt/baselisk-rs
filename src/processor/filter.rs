extern crate sample;

use defs;
use event::{EngineEvent, ModulatableParameter, ModulatableParameterUpdateData};
use parameter::{Parameter, FrequencyParameter, LinearParameter};
use sample::{Frame, slice};
use std::default::Default;

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

/// A low pass filter type that can be used for audio processing.
/// This is to be a constant-peak-gain two-pole resonator with
/// parameterized cutoff frequency and resonance.
pub struct Filter
{
    params: FilterParams,
    sample_rate: defs::Sample,
    last_adsr_input_sample: defs::Sample,
    biquad_coefficient_func: BiquadCoefficientGeneratorFunc,
    history: BiquadSampleHistory,
    coeffs: BiquadCoefficients,
}

impl Filter
{
    /// Constructor for new Filter instances
    pub fn new() -> Filter {
        Filter {
            params: FilterParams::new(),
            sample_rate: 0.0,
            last_adsr_input_sample: 2.0, // Force computation the first time
            biquad_coefficient_func: get_lowpass_second_order_biquad_consts,
            history: BiquadSampleHistory::new(),
            coeffs: BiquadCoefficients::new(),
        }
    }

    pub fn midi_panic(&mut self) {
        self.history.reset();
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
        self.sample_rate = sample_rate;

        // Calculate the output values per-frame
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
                let output_buffer_slice = output_buffer.get_mut(
                        this_keyframe..next_keyframe).unwrap();
                let adsr_input_buffer_slice = adsr_input_buffer.get(
                        this_keyframe..next_keyframe).unwrap();

                let quality_factor = self.params.quality_factor.get();
                let base_frequency_hz = self.params.frequency.get();
                let adsr_sweep_octaves = self.params.adsr_sweep_octaves.get();

                // Iterate over two buffer slices at once using a zip method
                slice::zip_map_in_place(output_buffer_slice, adsr_input_buffer_slice,
                                        |output_frame, adsr_input_frame|
                {
                    // Iterate over the samples in each frame using a zip method
                    output_frame.zip_map(adsr_input_frame,
                                         |sample, adsr_input_sample|
                    {
                        if self.last_adsr_input_sample != adsr_input_sample {
                            self.last_adsr_input_sample = adsr_input_sample;
                            // Use adsr_input (0 <= x <= 1) to determine the influence
                            // of self.params.adsr_sweep_octaves on the filter frequency.
                            let frequency_hz = base_frequency_hz
                                * defs::Sample::exp2(adsr_sweep_octaves * adsr_input_sample);

                            (self.biquad_coefficient_func)(
                                    frequency_hz, quality_factor, self.sample_rate, &mut self.coeffs);
                        }

                        process_biquad(
                            &mut self.history,
                            &self.coeffs,
                            sample)
                    })
                });
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
    pub fn new() -> BiquadCoefficients {
        Default::default()
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
    pub fn new() -> BiquadSampleHistory {
        Default::default()
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
