extern crate dsp;

use arraydeque::ArrayDeque;
use dsp::Sample;
use event;
use processor;
use std::fmt;
use std::f64;
use std::f32::consts::PI;
use std::rc::Rc;
use std::cell::RefCell;

/// Parameters available for filters.
#[derive(Clone)]
struct FilterParams {
    frequency_hz: f32,
    quality_factor: f32,
}

impl FilterParams {
    /// Constructor for FilterParams instances
    fn new() -> FilterParams {
        FilterParams {
            frequency_hz: 10000.0,
            quality_factor: 1.0 / f32::sqrt(2.0),
        }
    }

    /// Set parameters for this filter.
    fn set(&mut self, param_name: String, param_val: String, sample_rate: f64) -> Result<(), String> {
        let param_val = param_val
            .parse::<f32>()
            .or_else(|_| return Err(String::from("param_val can't be parsed as a float")))
            .unwrap();

        match param_name.as_str() {
            "f" | "freq" | "frequency" | "frequency_hz" => {
                if param_val > 0.0 && param_val < ((sample_rate as f32) / 2.0) {
                    self.frequency_hz = param_val;
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

/// Function to construct new filter processors
pub fn new<S>(
    name: &str,
    _event_buffer: Rc<RefCell<event::Buffer>>,
) -> Result<Box<dyn processor::Processor<S>>, &'static str>
where
    S: dsp::sample::FloatSample + dsp::FromSample<f32> + fmt::Display + 'static,
    f32: dsp::FromSample<S>,
{
    match name {
        "lowpass" => {
            let name = String::from(name);
            let params = FilterParams::new();
            Ok(Box::new(LowPassFilter::new(
                name.clone(),
                params.clone(),
            )))
        }
        _ => Err("Unknown filter name"),
    }
}

/// A low pass filter type that can be used for audio processing.
/// This is to be a constant-peak-gain two-pole resonator with
/// parameterized cutoff frequency and resonance.
struct LowPassFilter
{
    name: String,
    params: FilterParams,
    sample_rate: f64,
    ringbuffer_input: ArrayDeque<[f32; 3]>,
    ringbuffer_output: ArrayDeque<[f32; 2]>,
}

impl LowPassFilter
{
    /// Constructor for new LowPassFilter instances
    fn new(
        name: String,
        params: FilterParams,
    ) -> LowPassFilter {
        let mut result = LowPassFilter {
            name,
            params,
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
}

impl processor::ProcessorView for LowPassFilter
{
    fn name(&self) -> String {
        self.name.clone()
    }

    fn details(&self) -> String {
        panic!("Not implemented: details for LowPassFilter");
    }

    fn set_param(&mut self, param_name: String, param_val: String) -> Result<(), String> {
        // Sample rate isn't important for the user-facing params
        self.params.set(param_name, param_val, self.sample_rate)
    }
}

impl<S> processor::Processor<S> for LowPassFilter
where
    S: dsp::sample::FloatSample + dsp::FromSample<f32> + fmt::Display,
    f32: dsp::FromSample<S>,
{
    fn update_state(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
    }

    fn process(&mut self, input: S) -> S
    where
        S: dsp::sample::FloatSample + dsp::FromSample<f32> + fmt::Display,
        f32: dsp::FromSample<S>,
    {
        // Update input buffer:
        self.ringbuffer_input.pop_back().unwrap();
        self.ringbuffer_input.push_front(f32::from_sample(input)).unwrap();

        let mut input_iter = self.ringbuffer_input.iter();
        let x_0 = *input_iter.next().unwrap();
        let x_1 = *input_iter.next().unwrap();
        let x_2 = *input_iter.next().unwrap();

        let y_1: f32;
        let y_2: f32;
        {
            let mut output_iter = self.ringbuffer_output.iter();
            y_1 = *output_iter.next().unwrap();
            y_2 = *output_iter.next().unwrap();
        } // End borrow of ringbuffer_output

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
        let theta_c = 2.0 * PI * self.params.frequency_hz / self.sample_rate as f32;
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

        y_0.to_sample::<S>()
    }

    fn get_view(&self) -> Box<dyn processor::ProcessorView> {
        Box::new(LowPassFilterView {
            name: self.name.clone(),
            params: self.params.clone(),
        })
    }
}

/// A view representation of a LowPassFilter.
struct LowPassFilterView {
    name: String,
    params: FilterParams,
}

impl processor::ProcessorView for LowPassFilterView {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn details(&self) -> String {
        panic!("Not implemented: details for LowPassFilter");
    }

    fn set_param(&mut self, param_name: String, param_val: String) -> Result<(), String> {
        // Sample rate isn't important for the user-facing params
        self.params.set(param_name, param_val, f64::MAX)
    }
}

