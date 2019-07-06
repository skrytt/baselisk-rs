extern crate sample;

use defs;
use shared::{
    event::EngineEvent,
    parameter::{
        BaseliskPluginParameters,
        PARAM_WAVESHAPER_INPUT_GAIN,
        PARAM_WAVESHAPER_OUTPUT_GAIN,
    },
};
use std::slice::Iter;
use vst::plugin::PluginParameters;

pub fn process_buffer(buffer: &mut defs::MonoFrameBufferSlice,
                      mut engine_event_iter: Iter<(usize, EngineEvent)>,
                      params: &BaseliskPluginParameters)
{
    // Calculate the output values per-frame
    let mut this_keyframe: usize = 0;
    let mut next_keyframe: usize;
    loop {
        // Get next selected note, if there is one.
        let next_event = engine_event_iter.next();

        if let Some((frame_num, engine_event)) = next_event {
            match engine_event {
                EngineEvent::ModulateParameter { param_id, .. } => match *param_id {
                    // Waveshaper parameter events will trigger keyframes
                    PARAM_WAVESHAPER_INPUT_GAIN |
                    PARAM_WAVESHAPER_OUTPUT_GAIN => (),
                    _ => continue,
                },
                _ => continue,
            }
            next_keyframe = *frame_num;
        } else {
            // No more note change events, so we'll process to the end of the buffer.
            next_keyframe = buffer.len();
        };

        // Apply the old parameters up until next_keyframe.
        if let Some(buffer_slice) = buffer.get_mut(this_keyframe..next_keyframe) {
            let input_gain = params.get_real_value(PARAM_WAVESHAPER_INPUT_GAIN);
            let output_gain = params.get_real_value(PARAM_WAVESHAPER_OUTPUT_GAIN);
            for frame in buffer_slice {
                for sample in frame {
                    *sample = {
                        // Polynomial: -x^3 + x^2 + x
                        // With input and output gain scaling
                        let x = sample.abs().min(1.0) * input_gain;
                        output_gain * sample.signum() * (
                            -x.powi(3) + x.powi(2) + x)
                    };
                }
            }
        }

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
                    PARAM_WAVESHAPER_INPUT_GAIN |
                    PARAM_WAVESHAPER_OUTPUT_GAIN => {
                        params.set_parameter(*param_id, *value);
                    },
                    _ => (),
                }
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use defs;

    #[test]
    /// Render two frames with output gain zero (silence),
    /// apply an EngineEvent on the third frame to set output gain to unity,
    /// and finally render two frames with output gain one.
    fn test_change_output_gain() {
        let mut engine_events = Vec::new();
        engine_events.push((2, EngineEvent::ModulateParameter{
            param_id: PARAM_WAVESHAPER_OUTPUT_GAIN, value: 1.0}));

        _test(1.0,
              0.0,
              vec![[1.0], [1.0], [1.0], [1.0]],
              engine_events,
              vec![[0.0], [0.0], [1.0], [1.0]],
        )
    }

    fn _test(input_gain: defs::Sample,
             output_gain: defs::Sample,
             mut buffer: Vec<defs::MonoFrame>,
             engine_events: Vec<(usize, EngineEvent)>,
             expected_buffer: Vec<defs::MonoFrame>)
    {
        let params = BaseliskPluginParameters::default();
        params.update_real_value_from_string(
            PARAM_WAVESHAPER_INPUT_GAIN, format!("{}", input_gain)).unwrap();
        params.update_real_value_from_string(
            PARAM_WAVESHAPER_OUTPUT_GAIN, format!("{}", output_gain)).unwrap();

        process_buffer(&mut buffer, engine_events.iter(), &params);

        for i in 0..buffer.len() {
            assert_eq!(buffer[i], expected_buffer[i]);
        }
    }
}
