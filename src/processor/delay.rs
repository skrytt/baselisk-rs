extern crate sample;

use buffer::ResizableFrameBuffer;
use defs;
use event::EngineEvent;
use parameter::{
    BaseliskPluginParameters,
    PARAM_DELAY_FEEDBACK,
    PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY,
    PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY,
    PARAM_DELAY_WET_GAIN,
};
use processor::filter::{
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
    delay_buffer: Option<ring_buffer::Fixed<Vec<defs::Sample>>>,
    highpass_history: BiquadSampleHistory,
    highpass_coeffs: BiquadCoefficients,
    lowpass_history: BiquadSampleHistory,
    lowpass_coeffs: BiquadCoefficients,
    wet_buffer: ResizableFrameBuffer<defs::MonoFrame>,
}

impl Delay {
    pub fn new() -> Self {
        let delay_buffer_size = 24000;
        let mut delay_buffer_vec = Vec::with_capacity(delay_buffer_size);
        for _ in 0..delay_buffer_size {
            delay_buffer_vec.push(0.0);
        }

        let delay_buffer = Some(ring_buffer::Fixed::from(delay_buffer_vec));

        Self {
            delay_buffer,
            highpass_history: BiquadSampleHistory::new(),
            highpass_coeffs: BiquadCoefficients::new(),
            lowpass_history: BiquadSampleHistory::new(),
            lowpass_coeffs: BiquadCoefficients::new(),
            wet_buffer: ResizableFrameBuffer::new(),
        }
    }

    pub fn process_buffer(&mut self,
                          buffer: &mut defs::MonoFrameBufferSlice,
                          mut engine_event_iter: Iter<(usize, EngineEvent)>,
                          sample_rate: defs::Sample,
                          params: &BaseliskPluginParameters)
    {
        let wet_buffer = self.wet_buffer.get_sized_mut(buffer.len());

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
                next_keyframe = buffer.len();
            };

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

            // Apply the old parameters up until next_keyframe.
            if let Some(delay_buffer) = &mut self.delay_buffer {
                let wet_gain = params.get_real_value(PARAM_DELAY_WET_GAIN);

                for frame_num in this_keyframe..next_keyframe {
                    let mut delayed_sample = feedback * delay_buffer.get(0);

                    // Apply highpass to delayed sample
                    delayed_sample = process_biquad(
                        &mut self.highpass_history,
                        &self.highpass_coeffs,
                        delayed_sample);

                    // Apply lowpass to delayed sample
                    delayed_sample = process_biquad(
                        &mut self.lowpass_history,
                        &self.lowpass_coeffs,
                        delayed_sample);

                    wet_buffer[frame_num] = [delayed_sample];

                    // Mix in the dry signal and push back to the delay buffer
                    let dry_sample = buffer[frame_num][0];
                    delay_buffer.push(
                        dry_sample + delayed_sample);

                    // Mix the wet signal into the output buffer with the dry signal.
                    buffer[frame_num][0] += wet_gain * wet_buffer[frame_num][0];
                }
            } // end borrow of buffer

            // We've reached the next_keyframe.
            this_keyframe = next_keyframe;

            // What we do now depends on whether we reached the end of the buffer.
            if this_keyframe == buffer.len() {
                // Loop exit condition: reached the end of the buffer.
                break
            } else {
                // Before the next iteration, use the event at this keyframe
                // to update the current state.
                let (_, event) = next_event.unwrap();
                if let EngineEvent::ModulateParameter { param_id, value } = event {
                    match *param_id {
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
