extern crate sample;

use defs;
use event::{EngineEvent, ModulatableParameter, ModulatableParameterUpdateData};
use parameter::{Parameter, LinearParameter};
use std::slice::Iter;

pub struct Waveshaper {
    input_gain: LinearParameter,
    output_gain: LinearParameter,
}

impl Waveshaper {
    pub fn new() -> Self {
        Self {
            input_gain: LinearParameter::new(0.0, 1.0, 0.333),
            output_gain: LinearParameter::new(0.0, 1.0, 0.2),
        }
    }

    pub fn update_input_gain(&mut self, data: ModulatableParameterUpdateData)
                          -> Result<(), &'static str> {
        self.input_gain.update_patch(data)
    }

    pub fn update_output_gain(&mut self, data: ModulatableParameterUpdateData)
                           -> Result<(), &'static str> {
        self.output_gain.update_patch(data)
    }

    pub fn process_buffer(&mut self,
                          buffer: &mut defs::MonoFrameBufferSlice,
                          mut engine_event_iter: Iter<(usize, EngineEvent)>)
    {
        // Calculate the output values per-frame
        let mut this_keyframe: usize = 0;
        let mut next_keyframe: usize;
        loop {
            // Get next selected note, if there is one.
            let next_event = engine_event_iter.next();

            if let Some((frame_num, engine_event)) = next_event {
                match engine_event {
                    EngineEvent::ModulateParameter { parameter, .. } => match parameter {
                        // Waveshaper parameter events will trigger keyframes
                        ModulatableParameter::WaveshaperInputGain |
                        ModulatableParameter::WaveshaperOutputGain => (),
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
                for frame in buffer_slice {
                    for sample in frame {
                        *sample = {
                            // Polynomial: -x^3 + x^2 + x
                            // With input and output gain scaling
                            let x = sample.abs().min(1.0) * self.input_gain.get();
                            self.output_gain.get() * sample.signum() * (
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
                if let EngineEvent::ModulateParameter { parameter, value } = event {
                    match parameter {
                        ModulatableParameter::WaveshaperInputGain => {
                            self.input_gain.update_cc(*value);
                        },
                        ModulatableParameter::WaveshaperOutputGain => {
                            self.output_gain.update_cc(*value);
                        },
                        _ => (),
                    }
                };
            }
        }
    }
}
