extern crate sample;

use defs;
use event::{
    EngineEvent,
    //ModulatableParameter,
    //ModulatableParameterUpdateData,
};
use processor::filter::{
    get_lowpass_second_order_biquad_consts,
    get_highpass_second_order_biquad_consts,
    //process_biquad,
};
use sample::ring_buffer;
use std::slice::Iter;

pub struct Delay {
    delay_buffer: ring_buffer::Fixed<Vec<defs::Sample>>,
    feedback: defs::Sample,
    lowpass_ringbuffer_input: ring_buffer::Fixed<[defs::Sample; 3]>,
    lowpass_ringbuffer_output: ring_buffer::Fixed<[defs::Sample; 2]>,
    highpass_ringbuffer_input: ring_buffer::Fixed<[defs::Sample; 3]>,
    highpass_ringbuffer_output: ring_buffer::Fixed<[defs::Sample; 2]>,
}

impl Delay {
    pub fn new() -> Delay {
        let delay_buffer_size = 24000;
        let mut delay_buffer_vec = Vec::with_capacity(delay_buffer_size);
        for _ in 0..delay_buffer_size {
            delay_buffer_vec.push(0.0);
        }

        let delay_buffer = ring_buffer::Fixed::from(delay_buffer_vec);

        Delay {
            delay_buffer,
            feedback: 1.0,
            lowpass_ringbuffer_input: ring_buffer::Fixed::from([0.0; 3]),
            lowpass_ringbuffer_output: ring_buffer::Fixed::from([0.0; 2]),
            highpass_ringbuffer_input: ring_buffer::Fixed::from([0.0; 3]),
            highpass_ringbuffer_output: ring_buffer::Fixed::from([0.0; 2]),
        }
    }

    pub fn process_buffer(&mut self,
                          buffer: &mut defs::MonoFrameBufferSlice,
                          mut engine_event_iter: Iter<(usize, EngineEvent)>,
                          sample_rate: defs::Sample)
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

            //let lowpass_frequency_hz = 1000.0; // TODO: make not hard-coded
            //let highpass_frequency_hz = 250.0; // TODO: make not hard-coded
            //let quality_factor = 0.707; // TODO: make not hard-coded

            // Lowpass filter coefficients
            //let (lp_b0, lp_b1, lp_b2, lp_a1, lp_a2) = get_lowpass_second_order_biquad_consts(
            //        lowpass_frequency_hz, quality_factor, sample_rate);

            // Highpass filter coefficients
            //let (hp_b0, hp_b1, hp_b2, hp_a1, hp_a2) = get_highpass_second_order_biquad_consts(
            //        highpass_frequency_hz, quality_factor, sample_rate);

            // Apply the old parameters up until next_keyframe.
            if let Some(buffer_slice) = buffer.get_mut(this_keyframe..next_keyframe) {
                for frame in buffer_slice {
                    for sample in frame {
                        // Combine the original sample with an attenuated copy of the
                        // delayed sample.
                        let mut delayed_sample = self.feedback * self.delay_buffer.get(0);

                        // Apply lowpass
                        //delayed_sample = process_biquad(
                        //    self.lowpass_ringbuffer_input, self.lowpass_ringbuffer_output,
                        //    lp_b0, lp_b1, lp_b2,
                        //    lp_a1, lp_a2,
                        //    delayed_sample);

                        // Apply highpass
                        //delayed_sample = process_biquad(
                        //    self.highpass_ringbuffer_input, self.highpass_ringbuffer_output,
                        //    hp_b0, hp_b1, hp_b2,
                        //    hp_a1, hp_a2,
                        //    delayed_sample);

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
