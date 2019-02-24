extern crate dsp;

use arraydeque::ArrayDeque;
use defs;
use dsp::sample::{slice, frame};
use dsp::Frame;

/// Parameters available for filters.
#[derive(Clone)]
struct FilterParams {
    base_frequency_hz: defs::Output,
    adsr_sweep_octaves: defs::Output,
    quality_factor: defs::Output,
}

impl FilterParams {
    /// Constructor for FilterParams instances
    fn new() -> FilterParams {
        FilterParams {
            base_frequency_hz: 100.0,
            adsr_sweep_octaves: 7.0, // Note: the filter becomes unstable if the value of
                                     // adsr_sweep_octaves is increased beyond 7.0.
            quality_factor: 2.0,
        }
    }

    /// Set parameters for this filter.
    fn set(&mut self, param_name: String, param_val: String, sample_rate: defs::Output) -> Result<(), String> {
        let param_val = param_val
            .parse::<defs::Output>()
            .or_else(|_| return Err(String::from("param_val can't be parsed as a float")))
            .unwrap();

        match param_name.as_str() {
            "f" | "freq" | "frequency" | "frequency_hz" => {
                if param_val > 0.0 && param_val < ((sample_rate as defs::Output) / 2.0) {
                    self.base_frequency_hz = param_val;
                } else {
                    return Err(String::from("Filter frequency must be 0.0 > f > sample_rate/2.0"))
                }
            },
            "q" | "qual" | "quality" | "quality_factor" => {
                if param_val >= 0.5 && param_val <= 10.0 {
                    self.quality_factor = param_val;
                } else {
                    return Err(String::from("Filter resonance must be 0.5 >= r >= 10.0"))
                }
            }
            _ => return Err(String::from("Unknown param_name")),
        }
        Ok(())
    }
}

/// A low pass filter type that can be used for audio processing.
/// This is to be a constant-peak-gain two-pole resonator with
/// parameterized cutoff frequency and resonance.
pub struct LowPassFilter
{
    params: FilterParams,
    sample_rate: defs::Output,
    ringbuffer_input: ArrayDeque<[defs::Output; 3]>,
    ringbuffer_output: ArrayDeque<[defs::Output; 2]>,
}

impl LowPassFilter
{
    /// Constructor for new LowPassFilter instances
    pub fn new() -> LowPassFilter {
        let mut result = LowPassFilter {
            params: FilterParams::new(),
            sample_rate: 0.0,
            ringbuffer_input: ArrayDeque::new(),
            ringbuffer_output: ArrayDeque::new(),
        };

        // Prepopulate the buffers
        for _ in 1..=3 {
            result.ringbuffer_input.push_front(0.0).unwrap();
        }
        for _ in 1..=2 {
            result.ringbuffer_output.push_front(0.0).unwrap();
        }

        result
    }

    pub fn process_buffer(&mut self,
                          adsr_input_buffer: &[frame::Mono<defs::Output>],
                          output_buffer: &mut [frame::Mono<defs::Output>],
                          sample_rate: defs::Output) {
        self.sample_rate = sample_rate;

        // Iterate over two buffers at once using a zip method
        slice::zip_map_in_place(output_buffer, adsr_input_buffer,
                                |output_frame, adsr_input_frame|
        {
            // Iterate over the samples in each frame using a zip method
            output_frame.zip_map(adsr_input_frame,
                                 |output_sample, adsr_input_sample| {
                self.process(output_sample, adsr_input_sample)
            })
        })
    }

    fn process(&mut self, input: defs::Output, adsr_input: defs::Output) -> defs::Output
    {
        // Update input buffer:
        self.ringbuffer_input.pop_back().unwrap();
        self.ringbuffer_input.push_front(input).unwrap();

        let mut input_iter = self.ringbuffer_input.iter();
        let x_0 = *input_iter.next().unwrap();
        let x_1 = *input_iter.next().unwrap();
        let x_2 = *input_iter.next().unwrap();

        let y_1: defs::Output;
        let y_2: defs::Output;
        {
            let mut output_iter = self.ringbuffer_output.iter();
            y_1 = *output_iter.next().unwrap();
            y_2 = *output_iter.next().unwrap();
        } // End borrow of ringbuffer_output

        // Use adsr_input (0 <= x <= 1) to determine the influence
        // of self.params.adsr_sweep_octaves on the filter frequency.
        let frequency_hz = self.params.base_frequency_hz
                            * defs::Output::exp2(self.params.adsr_sweep_octaves * adsr_input);

        // We implement a biquad section with coefficients selected to achieve
        // a low-pass filter.
        //
        // The general biquad implementation for Direct Form 1:
        //
        // y_0 = b0 * x_0 + b1 * x_1 + b2 * x_2
        //                - a1 * y_1 - a2 * y_2
        //
        // The above assumes a constant a0 has been divided off all the other coefficients
        // to save on computation steps.
        // There are some intermediate variables:
        //
        let theta_c = 2.0 * defs::PI * frequency_hz / self.sample_rate as defs::Output;
        let cos_theta_c = theta_c.cos();
        let sin_theta_c = theta_c.sin();
        let alpha = sin_theta_c / (2.0 * self.params.quality_factor);

        // Calculate the coefficients.
        // We'll just divide off a_0 from each one to save on computation.
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_theta_c / a0;
        let a2 = (1.0 - alpha) / a0;

        let b1 = (1.0 - cos_theta_c) / a0;
        let b0 = b1 / 2.0;
        let b2 = b0;

        let y_0 = b0 * x_0 + b1 * x_1 + b2 * x_2
                           - a1 * y_1 - a2 * y_2;

        // Update output buffer for next time:
        self.ringbuffer_output.pop_back().unwrap();
        self.ringbuffer_output.push_front(y_0).unwrap();

        // TODO: remove these once confident in filter stability
        assert!(y_0 >= -5.0);
        assert!(y_0 <= 5.0);

        // Set sample to output
        y_0
    }
}
