extern crate sample;

use defs;
use event::{
    EngineEvent,
    //ModulatableParameter,
    //ModulatableParameterUpdateData,
};
use sample::ring_buffer;
use std::slice::Iter;

pub struct Delay {
    delay_buffer: ring_buffer::Fixed<Vec<defs::Sample>>,
    feedback: defs::Sample,
}

impl Delay {
    pub fn new() -> Delay {
        let delay_buffer_size = 24000;
        let mut delay_buffer_vec = Vec::with_capacity(delay_buffer_size);
        for _ in 0..delay_buffer_size {
            delay_buffer_vec.push(0.0);
        }

        let delay_buffer = ring_buffer::Fixed::from(delay_buffer_vec);

        Delay{
            delay_buffer,
            feedback: 0.8,
        }
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

            // This match block continues on events that are unimportant to this processor.
            match next_event {
                Some((_frame_num, _engine_event)) => {
                    // Placeholder for actual event handling, but does nothing for now
                    continue
                },
                None => {
                    // No more note change events, so we'll process to the end of the buffer.
                    next_keyframe = buffer.len();
                },
            };

            // Apply the old parameters up until next_keyframe.
            if let Some(buffer_slice) = buffer.get_mut(this_keyframe..next_keyframe) {
                for frame in buffer_slice {
                    for sample in frame {
                        // Combine the original sample with an attenuated copy of the
                        // delayed sample.
                        let delayed_sample = self.feedback * self.delay_buffer.get(0);
                        *sample += delayed_sample;
                        self.delay_buffer.push(*sample);
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
                match event {
                    EngineEvent::ModulateParameter { parameter, .. } => match parameter {
                        _ => (),
                    },
                    _ => (),
                };
            }
        }
    }
}
