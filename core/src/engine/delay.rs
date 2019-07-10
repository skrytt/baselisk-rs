extern crate sample;

use defs;
use shared::{
    event::EngineEvent,
    parameter::{
        BaseliskPluginParameters,
        PARAM_DELAY_TIME_LEFT,
        PARAM_DELAY_TIME_RIGHT,
        PARAM_DELAY_FEEDBACK,
        PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY,
        PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY,
        PARAM_DELAY_WET_GAIN,
    },
};
use engine::{
    buffer::ResizableFrameBuffer,
    filter::{
        BiquadCoefficients,
        BiquadSampleHistory,
        get_lowpass_second_order_biquad_consts,
        get_highpass_second_order_biquad_consts,
        process_biquad,
    },
};
use sample::ring_buffer;
use std::slice::Iter;
use vst::plugin::PluginParameters;

pub struct DelayChannel {
    delay_buffer: ring_buffer::Fixed<Vec<defs::Sample>>,
    highpass_history: BiquadSampleHistory,
    lowpass_history: BiquadSampleHistory,
    wet_buffer: ResizableFrameBuffer<defs::MonoFrame>,
}

impl DelayChannel {
    pub fn new() -> Self {
        // We'll throw this temporary ringbuffer away as soon as we know the
        // real size (proportional to sample rate, which we don't know yet).
        let mut delay_buffer_vec = Vec::with_capacity(1);
        delay_buffer_vec.push(0.0);

        Self {
            delay_buffer: ring_buffer::Fixed::from(delay_buffer_vec),
            highpass_history: BiquadSampleHistory::new(),
            lowpass_history: BiquadSampleHistory::new(),
            wet_buffer: ResizableFrameBuffer::new(),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: defs::Sample) {
        let capacity = sample_rate as usize; // 1 second of buffer time

        let mut delay_buffer_vec = Vec::with_capacity(capacity);
        delay_buffer_vec.resize(capacity, 0.0);

        self.delay_buffer = ring_buffer::Fixed::from(delay_buffer_vec);
    }

    pub fn process_between_keyframes(&mut self,
                                     this_keyframe: usize,
                                     next_keyframe: usize,
                                     delay_time: defs::Sample,
                                     feedback: defs::Sample,
                                     wet_gain: defs::Sample,
                                     highpass_coeffs: &BiquadCoefficients,
                                     lowpass_coeffs: &BiquadCoefficients,
                                     buffer: &mut defs::MonoFrameBufferSlice)
    {
        let wet_buffer = self.wet_buffer.get_sized_mut(buffer.len());

        let buffer_tap_position_float = self.delay_buffer.len() as defs::Sample * (
            1.0 - delay_time);
        let buffer_tap_a_index = buffer_tap_position_float as usize;
        let buffer_tap_b_index = buffer_tap_a_index + 1;
        let delayed_sample_b_weight = buffer_tap_position_float.fract();
        let delayed_sample_a_weight = 1.0 - delayed_sample_b_weight;

        for frame_num in this_keyframe..next_keyframe {
            let delayed_sample_a = self.delay_buffer[buffer_tap_a_index];
            let delayed_sample_b = self.delay_buffer[buffer_tap_b_index];
            let mut delayed_sample = feedback * (
                                    delayed_sample_a * delayed_sample_a_weight
                                    + delayed_sample_b * delayed_sample_b_weight);

            // Apply highpass to left delayed sample
            delayed_sample = process_biquad(
                &mut self.highpass_history,
                highpass_coeffs,
                delayed_sample);

            // Apply lowpass to left delayed sample
            delayed_sample = process_biquad(
                &mut self.lowpass_history,
                lowpass_coeffs,
                delayed_sample);

            wet_buffer[frame_num] = [delayed_sample];

            // Mix in the dry signal and push back to the delay buffer
            let dry_sample = buffer[frame_num][0];
            self.delay_buffer.push(
                dry_sample + delayed_sample);

            // Mix the wet signal into the output buffer with the dry signal.
            buffer[frame_num][0] += wet_gain * wet_buffer[frame_num][0];

        } // end borrow of buffer
    }
}

pub struct Delay {
    highpass_coeffs: BiquadCoefficients,
    lowpass_coeffs: BiquadCoefficients,
    channels: [DelayChannel; 2],
}

impl Delay {
    pub fn new() -> Self {
        Self {
            highpass_coeffs: BiquadCoefficients::new(),
            lowpass_coeffs: BiquadCoefficients::new(),
            channels: [DelayChannel::new(), DelayChannel::new()],
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: defs::Sample) {
        for channel in self.channels.iter_mut() {
            channel.set_sample_rate(sample_rate);
        }
    }

    pub fn process_buffer(&mut self,
                          left_buffer: &mut defs::MonoFrameBufferSlice,
                          right_buffer: &mut defs::MonoFrameBufferSlice,
                          mut engine_event_iter: Iter<(usize, EngineEvent)>,
                          sample_rate: defs::Sample,
                          params: &BaseliskPluginParameters)
    {
        let buffer_len = left_buffer.len(); // right_buffer must be same length
        self.channels[0].wet_buffer.get_sized_mut(buffer_len);
        self.channels[1].wet_buffer.get_sized_mut(buffer_len);

        // Calculate the output values per-frame
        let mut this_keyframe: usize = 0;
        let mut next_keyframe: usize;
        loop {
            // Get next selected note, if there is one.
            let next_event = engine_event_iter.next();

            if let Some((frame_num, engine_event)) = next_event {
                match engine_event {
                    EngineEvent::ModulateParameter { param_id, .. } => match *param_id {
                        // All delay events will trigger keyframes
                        PARAM_DELAY_TIME_LEFT |
                        PARAM_DELAY_TIME_RIGHT |
                        PARAM_DELAY_FEEDBACK |
                        PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY |
                        PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY |
                        PARAM_DELAY_WET_GAIN => (),
                        _ => continue,
                    },
                    _ => continue,
                };
                next_keyframe = *frame_num;
            } else {
                // No more note change events, so we'll process to the end of the buffer.
                next_keyframe = buffer_len;
            };

            // Apply the old parameters up until next_keyframe.
            let feedback = params.get_real_value(PARAM_DELAY_FEEDBACK);
            let wet_gain = params.get_real_value(PARAM_DELAY_WET_GAIN);

            let lowpass_frequency_hz = params.get_real_value(
                PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY);
            let lowpass_quality = 0.707;

            // Lowpass filter coefficients
            get_lowpass_second_order_biquad_consts(
                    lowpass_frequency_hz,
                    lowpass_quality,
                    sample_rate,
                    &mut self.lowpass_coeffs);

            let highpass_frequency_hz = params.get_real_value(
                PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY);
            let highpass_quality = 0.707;

            // Highpass filter coefficients
            get_highpass_second_order_biquad_consts(
                    highpass_frequency_hz,
                    highpass_quality,
                    sample_rate,
                    &mut self.highpass_coeffs);

            // Left first...
            self.channels[0].process_between_keyframes(
                 this_keyframe,
                 next_keyframe,
                 params.get_real_value(PARAM_DELAY_TIME_LEFT),
                 feedback,
                 wet_gain,
                 &self.highpass_coeffs,
                 &self.lowpass_coeffs,
                 left_buffer);

            // ... Then right
            self.channels[1].process_between_keyframes(
                 this_keyframe,
                 next_keyframe,
                 params.get_real_value(PARAM_DELAY_TIME_RIGHT),
                 feedback,
                 wet_gain,
                 &self.highpass_coeffs,
                 &self.lowpass_coeffs,
                 right_buffer);

            // We've reached the next_keyframe.
            this_keyframe = next_keyframe;

            // What we do now depends on whether we reached the end of the buffer.
            if this_keyframe == buffer_len {
                // Loop exit condition: reached the end of the buffer.
                break
            } else {
                // Before the next iteration, use the event at this keyframe
                // to update the current state.
                let (_, event) = next_event.unwrap();
                if let EngineEvent::ModulateParameter { param_id, value } = event {
                    match *param_id {
                        PARAM_DELAY_TIME_LEFT |
                        PARAM_DELAY_TIME_RIGHT |
                        PARAM_DELAY_FEEDBACK |
                        PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY |
                        PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY |
                        PARAM_DELAY_WET_GAIN => {
                            params.set_parameter(*param_id, *value);
                        }
                        _ => (),
                    }
                };
            }
        }
    }
}
