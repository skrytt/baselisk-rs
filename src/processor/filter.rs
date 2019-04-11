extern crate sample;

use defs;
use event::{EngineEvent, ModulatableParameter, ModulatableParameterUpdateData};
use parameter::{Parameter, FrequencyParameter, LinearParameter};
use sample::{Frame, slice, ring_buffer};

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
pub struct LowPassFilter
{
    params: FilterParams,
    sample_rate: defs::Sample,
    ringbuffer_input: ring_buffer::Fixed<[defs::Sample; 3]>,
    ringbuffer_output: ring_buffer::Fixed<[defs::Sample; 2]>,
}

impl LowPassFilter
{
    /// Constructor for new LowPassFilter instances
    pub fn new() -> LowPassFilter {
        LowPassFilter {
            params: FilterParams::new(),
            sample_rate: 0.0,
            ringbuffer_input: ring_buffer::Fixed::from([0.0; 3]),
            ringbuffer_output: ring_buffer::Fixed::from([0.0; 2]),
        }
    }

    pub fn midi_panic(&mut self) {
        // Reset the buffers
        for _ in 0..self.ringbuffer_input.len() {
            self.ringbuffer_input.push(0.0);
        }
        for _ in 0..self.ringbuffer_output.len() {
            self.ringbuffer_output.push(0.0);
        }
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

                // Iterate over two buffer slices at once using a zip method
                slice::zip_map_in_place(output_buffer_slice, adsr_input_buffer_slice,
                                        |output_frame, adsr_input_frame|
                {
                    // Iterate over the samples in each frame using a zip method
                    output_frame.zip_map(adsr_input_frame,
                                         |output_sample, adsr_input_sample| {
                        self.process(output_sample, adsr_input_sample)
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

    fn process(&mut self, input: defs::Sample, adsr_input: defs::Sample) -> defs::Sample
    {
        // Update input buffer:
        self.ringbuffer_input.push(input);

        let mut input_iter = self.ringbuffer_input.iter();
        let x_2 = *input_iter.next().unwrap();
        let x_1 = *input_iter.next().unwrap();
        let x_0 = *input_iter.next().unwrap();

        let y_1: defs::Sample;
        let y_2: defs::Sample;
        {
            let mut output_iter = self.ringbuffer_output.iter();
            y_2 = *output_iter.next().unwrap();
            y_1 = *output_iter.next().unwrap();
        } // End borrow of ringbuffer_output

        // Use adsr_input (0 <= x <= 1) to determine the influence
        // of self.params.adsr_sweep_octaves on the filter frequency.
        let mut frequency_hz = self.params.frequency.get()
                               * defs::Sample::exp2(self.params.adsr_sweep_octaves.get()
                               * adsr_input);

        // Limit frequency_hz to just under half of the sample rate for stability.
        frequency_hz = frequency_hz.min(0.495 * self.sample_rate);

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
        let theta_c = 2.0 * defs::PI * frequency_hz / self.sample_rate as defs::Sample;
        let cos_theta_c = theta_c.cos();
        let sin_theta_c = theta_c.sin();
        let alpha = sin_theta_c / (2.0 * self.params.quality_factor.get());

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
        self.ringbuffer_output.push(y_0);

        // TODO: remove these once confident in filter stability
        assert!(y_0 >= -5.0);
        assert!(y_0 <= 5.0);

        // Set sample to output
        y_0
    }
}
