extern crate dsp;

use arraydeque::ArrayDeque;
use dsp::Sample;
use event;
use processor;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone)]
struct FilterParams {
    frequency_hz: f32,
    resonance: f32,
}

impl FilterParams {
    fn new() -> FilterParams {
        FilterParams {
            frequency_hz: 1000.0,
            resonance: 0.0,
        }
    }

    fn set(&mut self, param_name: String, param_val: String, sample_rate: f64) -> Result<(), String> {
        let param_val = param_val
            .parse::<f32>()
            .or_else(|_| return Err(String::from("param_val can't be parsed as a float")))
            .unwrap();

        match param_name.as_str() {
            "frequency" | "frequency_hz" => {
                if param_val > 0.0 && param_val < (sample_rate as f32) / 2.0 {
                    self.frequency_hz = param_val;
                } else {
                    return Err(String::from("Filter frequency must be 0.0 > f > sample_rate/2.0"))
                }
            },
            "resonance" => {
                if param_val >= 0.0 && param_val <= 1.0 {
                    self.resonance = param_val;
                } else {
                    return Err(String::from("Filter resonance must be 0.0 >= r >= 1.0"))
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
        "lowpass_simple" => {
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

struct LowPassFilter
{
    name: String,
    params: FilterParams,
    sample_rate: f64,
    ringbuffer_input: ArrayDeque<[f32; 2]>,
}

impl LowPassFilter
{
    fn new(
        name: String,
        params: FilterParams,
    ) -> LowPassFilter {
        let mut result = LowPassFilter {
            name,
            params,
            sample_rate: 0.0,
            ringbuffer_input: ArrayDeque::new(),
        };
        result.ringbuffer_input.push_front(0.0).unwrap();
        result.ringbuffer_input.push_front(0.0).unwrap();
        result
    }
}

impl processor::ProcessorView for LowPassFilter
{
    fn name(&self) -> String {
        self.name.clone()
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
        // Indices reflect number of samples behind this sample.
        // Simplest lowpass filter:
        // y(0) = x(0) + x(1)

        self.ringbuffer_input.pop_back().unwrap();
        self.ringbuffer_input.push_front(f32::from_sample(input)).unwrap();

        let mut iter = self.ringbuffer_input.iter();

        let mut y: f32 = *iter.next().unwrap(); // x_0
        y = y + *iter.next().unwrap(); // x_0 + x_1

        y.to_sample::<S>()
    }

    fn get_view(&self) -> Box<dyn processor::ProcessorView> {
        Box::new(LowPassFilterView {
            name: self.name.clone(),
            params: self.params.clone(),
        })
    }
}

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
        self.params.set(param_name, param_val, 0.0)
    }
}

