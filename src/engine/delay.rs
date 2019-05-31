extern crate sample;

use engine::buffer::ResizableFrameBuffer;
use defs;
use event::EngineEvent;
use parameter::{
    BaseliskPluginParameters,
    PARAM_DELAY_TIME_LEFT,
    PARAM_DELAY_TIME_RIGHT,
    PARAM_DELAY_FEEDBACK,
    PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY,
    PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY,
    PARAM_DELAY_WET_GAIN,
};
use engine::filter::{
    BiquadCoefficients,
    BiquadSampleHistory,
    get_lowpass_second_order_biquad_consts,
    get_highpass_second_order_biquad_consts,
    process_biquad,
};
use sample::ring_buffer;
use std::slice::Iter;
use vst::plugin::PluginParameters;

pub struct Delay {
    left_delay_buffer: ring_buffer::Fixed<Vec<defs::Sample>>,
    right_delay_buffer: ring_buffer::Fixed<Vec<defs::Sample>>,
    left_highpass_history: BiquadSampleHistory,
    right_highpass_history: BiquadSampleHistory,
    highpass_coeffs: BiquadCoefficients,
    left_lowpass_history: BiquadSampleHistory,
    right_lowpass_history: BiquadSampleHistory,
    lowpass_coeffs: BiquadCoefficients,
    left_wet_buffer: ResizableFrameBuffer<defs::MonoFrame>,
    right_wet_buffer: ResizableFrameBuffer<defs::MonoFrame>,
}

impl Delay {
    pub fn new() -> Self {
        // BUG: this won't work if sample rate is changed.
        // e.g. if sample rate is 96000, the delay is effectively half of
        // what the parameter says.
        let delay_buffer_size = 48000;

        let mut left_delay_buffer_vec = Vec::with_capacity(delay_buffer_size);
        for _ in 0..delay_buffer_size {
            left_delay_buffer_vec.push(0.0);
        }
        let left_delay_buffer = ring_buffer::Fixed::from(left_delay_buffer_vec);

        let mut right_delay_buffer_vec = Vec::with_capacity(delay_buffer_size);
        for _ in 0..delay_buffer_size {
            right_delay_buffer_vec.push(0.0);
        }
        let right_delay_buffer = ring_buffer::Fixed::from(right_delay_buffer_vec);

        Self {
            left_delay_buffer,
            right_delay_buffer,
            left_highpass_history: BiquadSampleHistory::new(),
            right_highpass_history: BiquadSampleHistory::new(),
            highpass_coeffs: BiquadCoefficients::new(),
            left_lowpass_history: BiquadSampleHistory::new(),
            right_lowpass_history: BiquadSampleHistory::new(),
            lowpass_coeffs: BiquadCoefficients::new(),
            left_wet_buffer: ResizableFrameBuffer::new(),
            right_wet_buffer: ResizableFrameBuffer::new(),
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
        let left_wet_buffer = self.left_wet_buffer.get_sized_mut(buffer_len);
        let right_wet_buffer = self.right_wet_buffer.get_sized_mut(buffer_len);

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
            //
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

            let feedback = params.get_real_value(PARAM_DELAY_FEEDBACK);
            let wet_gain = params.get_real_value(PARAM_DELAY_WET_GAIN);

            // Left first...
            let left_buffer_tap_position_float = self.left_delay_buffer.len() as defs::Sample * (
                1.0 - params.get_real_value(PARAM_DELAY_TIME_LEFT));
            let left_buffer_tap_a_index = left_buffer_tap_position_float as usize;
            let left_buffer_tap_b_index = left_buffer_tap_a_index + 1;
            let left_delayed_sample_b_weight = left_buffer_tap_position_float.fract();
            let left_delayed_sample_a_weight = 1.0 - left_delayed_sample_b_weight;

            for frame_num in this_keyframe..next_keyframe {
                let left_delayed_sample_a = self.left_delay_buffer.get(left_buffer_tap_a_index);
                let left_delayed_sample_b = self.left_delay_buffer.get(left_buffer_tap_b_index);
                let mut left_delayed_sample = feedback * (
                                        left_delayed_sample_a * left_delayed_sample_a_weight
                                        + left_delayed_sample_b * left_delayed_sample_b_weight);

                // Apply highpass to left delayed sample
                left_delayed_sample = process_biquad(
                    &mut self.left_highpass_history,
                    &self.highpass_coeffs,
                    left_delayed_sample);

                // Apply lowpass to left delayed sample
                left_delayed_sample = process_biquad(
                    &mut self.left_lowpass_history,
                    &self.lowpass_coeffs,
                    left_delayed_sample);

                left_wet_buffer[frame_num] = [left_delayed_sample];

                // Mix in the dry signal and push back to the delay buffer
                let left_dry_sample = left_buffer[frame_num][0];
                self.left_delay_buffer.push(
                    left_dry_sample + left_delayed_sample);

                // Mix the wet signal into the output buffer with the dry signal.
                left_buffer[frame_num][0] += wet_gain * left_wet_buffer[frame_num][0];
            } // end borrow of buffer

            // ... Then right
            let right_buffer_tap_position_float = self.right_delay_buffer.len() as defs::Sample * (
                1.0 - params.get_real_value(PARAM_DELAY_TIME_RIGHT));
            let right_buffer_tap_a_index = right_buffer_tap_position_float as usize;
            let right_buffer_tap_b_index = right_buffer_tap_a_index + 1;
            let right_delayed_sample_b_weight = right_buffer_tap_position_float.fract();
            let right_delayed_sample_a_weight = 1.0 - right_delayed_sample_b_weight;

            for frame_num in this_keyframe..next_keyframe {
                let right_delayed_sample_a = self.right_delay_buffer.get(right_buffer_tap_a_index);
                let right_delayed_sample_b = self.right_delay_buffer.get(right_buffer_tap_b_index);
                let mut right_delayed_sample = feedback * (
                                        right_delayed_sample_a * right_delayed_sample_a_weight
                                        + right_delayed_sample_b * right_delayed_sample_b_weight);

                // Apply highpass to right delayed sample
                right_delayed_sample = process_biquad(
                    &mut self.right_highpass_history,
                    &self.highpass_coeffs,
                    right_delayed_sample);

                // Apply lowpass to right delayed sample
                right_delayed_sample = process_biquad(
                    &mut self.right_lowpass_history,
                    &self.lowpass_coeffs,
                    right_delayed_sample);

                right_wet_buffer[frame_num] = [right_delayed_sample];

                // Mix in the dry signal and push back to the delay buffer
                let right_dry_sample = right_buffer[frame_num][0];
                self.right_delay_buffer.push(
                    right_dry_sample + right_delayed_sample);

                // Mix the wet signal into the output buffer with the dry signal.
                right_buffer[frame_num][0] += wet_gain * right_wet_buffer[frame_num][0];
            } // end borrow of buffer

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
