use defs;
use engine::traits;
use shared::{
    event::EngineEvent,
    parameter::{
        BaseliskPluginParameters,
        ParameterId,
    },
};
use std::slice;

/// States that ADSR can be in
enum AdsrStages {
    HeldAttack,  // Attack
    HeldDecay,   // Decay
    HeldSustain, // Sustain
    Released,    // Release
}

/// Struct to hold the current state of an ADSR processor
struct AdsrState {
    stage: Option<AdsrStages>,
    notes_held_count: i32,
    gain_at_stage_start: f32,
    relative_gain_at_stage_end: f32,
    sample_duration: f32,
    phase_time: f32,
    selected_note: Option<u8>,
}

/// An ADSR struct with all the bits plugged together:
pub struct Adsr {
    state: AdsrState,
}

impl Adsr {
    pub fn new() -> Self {
        Self {
            state: AdsrState {
                stage: None,
                notes_held_count: 0,
                gain_at_stage_start: 0.0,
                relative_gain_at_stage_end: 0.0,
                sample_duration: 1.0, // Needs to be set by update_sample_rate
                phase_time: 0.0,
                selected_note: None,
            },
        }
    }

    pub fn update_state(&mut self,
                        any_notes_held: bool,
                        current_note_changed: bool,
                        params: &BaseliskPluginParameters)
    {
        // Percussive algorithm
        if any_notes_held && current_note_changed {
            // Transition to attack stage
            match self.state.stage {
                // Otherwise, avoid discontinuities
                _ => {
                    self.state.gain_at_stage_start = self.get_gain(params);
                    self.state.relative_gain_at_stage_end = 1.0 - self.state.gain_at_stage_start;
                    self.state.phase_time = 0.0;
                }
            }
            self.state.stage = Some(AdsrStages::HeldAttack);
        } else if !any_notes_held {
            // Transition to release stage
            if self.state.stage.is_some() {
                self.state.gain_at_stage_start = self.get_gain(params);
                self.state.relative_gain_at_stage_end = -self.state.gain_at_stage_start;
                self.state.phase_time = 0.0;
            }
            self.state.stage = Some(AdsrStages::Released);
        }
    }

    fn get_gain(&self, params: &BaseliskPluginParameters) -> f32 {
        match self.state.stage {
            None => 0.0,
            Some(AdsrStages::HeldAttack) => {
                self.state.gain_at_stage_start
                + self.state.relative_gain_at_stage_end * (
                    self.state.phase_time / params.get_real_value(ParameterId::AdsrAttack))
            }
            Some(AdsrStages::HeldDecay) => {
                self.state.gain_at_stage_start
                + self.state.relative_gain_at_stage_end * (
                    self.state.phase_time / params.get_real_value(ParameterId::AdsrDecay))
            }
            Some(AdsrStages::HeldSustain) => params.get_real_value(ParameterId::AdsrSustain),
            Some(AdsrStages::Released) => {
                self.state.gain_at_stage_start
                + self.state.relative_gain_at_stage_end * (
                    self.state.phase_time / params.get_real_value(ParameterId::AdsrRelease))
            }
        }
    }

    // Process the buffer of audio.
    // Return a bool indicating whether the output contains non-zero samples.
    pub fn process_buffer(&mut self,
                          buffer: &mut defs::MonoFrameBufferSlice,
                          mut engine_event_iter: slice::Iter<(usize, EngineEvent)>,
                          sample_rate: defs::Sample,
                          params: &BaseliskPluginParameters) -> bool
    {
        self.state.sample_duration = 1.0 / sample_rate as f32;

        let mut any_nonzero_output = self.state.stage.is_some();

        // Calculate the output values per-frame
        let mut this_keyframe: usize = 0;
        let mut next_keyframe: usize;
        loop {
            // Get next selected note, if there is one.
            let next_event = engine_event_iter.next();

            if let Some((frame_num, engine_event)) = next_event {
                match engine_event {
                    // All note changes and ADSR parameter changes will trigger keyframes
                    EngineEvent::NoteChange{ .. } => (),
                    EngineEvent::ModulateParameter { param_id, .. } => match *param_id {
                        ParameterId::AdsrAttack |
                        ParameterId::AdsrDecay |
                        ParameterId::AdsrSustain |
                        ParameterId::AdsrRelease => (),
                        _ => continue,
                    },
                    _ => continue,
                }
                next_keyframe = *frame_num;
            } else {
                // No more note change events, so we'll process to the end of the buffer.
                next_keyframe = buffer.len();
            }

            // Apply the old parameters up until next_keyframe.
            if let Some(buffer_slice) = buffer.get_mut(this_keyframe..next_keyframe) {
                for frame in buffer_slice {
                    for sample in frame {
                        *sample = self.advance(params);
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
                    EngineEvent::NoteChange{ note } => {
                        let any_notes_held_next = note.is_some();
                        let current_note_changed_next = *note != self.state.selected_note;

                        // If any note is pressed, assume this means there will be some output.
                        any_nonzero_output |= any_notes_held_next;

                        self.update_state(any_notes_held_next,
                                          current_note_changed_next,
                                          params);

                        self.state.selected_note = *note;
                    },
                    EngineEvent::ModulateParameter { param_id, value } => match *param_id {
                        ParameterId::AdsrAttack |
                        ParameterId::AdsrDecay |
                        ParameterId::AdsrSustain |
                        ParameterId::AdsrRelease => params.set_parameter(*param_id, *value),
                        _ => (),
                    },
                    _ => (),
                };
            }
        }
        // Return a bool to tell the engine whether there's any nonzero output, to
        // allow it to make any desired optimizations.
        any_nonzero_output
    }

    fn advance(&mut self, params: &BaseliskPluginParameters) -> defs::Sample {
        self.state.phase_time += self.state.sample_duration;

        // Handle attack -> decay advancing
        if let Some(AdsrStages::HeldAttack) = self.state.stage {
            if self.state.phase_time >= params.get_real_value(ParameterId::AdsrAttack) {
                self.state.stage = Some(AdsrStages::HeldDecay);
                self.state.gain_at_stage_start = 1.0;
                self.state.relative_gain_at_stage_end =
                    params.get_real_value(ParameterId::AdsrSustain) - self.state.gain_at_stage_start;
                self.state.phase_time -= params.get_real_value(ParameterId::AdsrAttack);
            }
        }
        // Handle decay -> sustain advancing
        if let Some(AdsrStages::HeldDecay) = self.state.stage {
            if self.state.phase_time >= params.get_real_value(ParameterId::AdsrDecay) {
                self.state.stage = Some(AdsrStages::HeldSustain);
            }
        }
        // Handle release -> off advancing
        if let Some(AdsrStages::Released) = self.state.stage {
            if self.state.phase_time >= params.get_real_value(ParameterId::AdsrRelease) {
                self.state.stage = None;
            }
        }

        // Return the output
        self.get_gain(params)
    }
}

impl traits::Processor for Adsr {
    fn panic(&mut self) {
        // Release all notes and reset state to "Off"
        self.state.notes_held_count = 0;
        self.state.stage = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // All tests use a sample rate of 1 second, so each sample is the result of
    // advancing the simulation time by 1 second.

    #[test]
    /// Test minimum-length attack and decay.
    /// We advance 1 second before computing the first sample,
    /// reaching the sustain immediately.
    fn test_ar_impulse_sustain_zero_with_note_hold() {
        let mut engine_events = Vec::new();
        engine_events.push((0, EngineEvent::NoteChange {note: Some(0)} ));

        let comparison_buffer = vec![[0.0], [0.0], [0.0], [0.0]];
        _test(0.02, 0.02, 0.0, 0.02, engine_events, comparison_buffer);
    }

    #[test]
    /// Test minimum-length attack and decay with high sustain.
    /// We advance 1 second before computing the first sample,
    /// reaching the sustain immediately.
    fn test_ar_impulse_sustain_one_with_note_hold() {
        let mut engine_events = Vec::new();
        engine_events.push((0, EngineEvent::NoteChange {note: Some(0)} ));

        let comparison_buffer = vec![[1.0], [1.0], [1.0], [1.0]];
        _test(0.02, 0.02, 1.0, 0.02, engine_events, comparison_buffer);
    }

    #[test]
    /// Test minimum-length attack and decay with half sustain.
    /// We advance 1 second before computing the first sample,
    /// reaching the sustain immediately.
    fn test_ar_impulse_sustain_half_with_note_hold() {
        let mut engine_events = Vec::new();
        engine_events.push((0, EngineEvent::NoteChange {note: Some(0)} ));

        let comparison_buffer = vec![[0.5], [0.5], [0.5], [0.5]];
        _test(0.02, 0.02, 0.5, 0.02, engine_events, comparison_buffer);
    }
    #[test]
    /// Test one-second attack and decay.
    /// The frame duration used in the test is 1 second. We advance 1 second
    /// before computing the first sample, reaching the end of the attack
    /// on the first sample, and the end of the decay on the second sample.
    fn test_ad_one_second_each_sustain_zero_with_note_held() {
        let mut engine_events = Vec::new();
        engine_events.push((0, EngineEvent::NoteChange {note: Some(0)} ));

        let comparison_buffer = vec![[1.0], [0.0], [0.0], [0.0]];
        _test(1.0, 1.0, 0.0, 0.02, engine_events, comparison_buffer);
    }

    #[test]
    /// Test two-second attack and decay.
    /// The frame duration used in the test is 1 second. We reach the end
    /// of the attack on the second sample, and the end of the decay on the
    /// fourth sample.
    fn test_ad_two_seconds_each_sustain_zero_with_note_held() {
        let mut engine_events = Vec::new();
        engine_events.push((0, EngineEvent::NoteChange {note: Some(0)} ));

        let comparison_buffer = vec![[0.5], [1.0], [0.5], [0.0], [0.0], [0.0]];
        _test(2.0, 2.0, 0.0, 0.02, engine_events, comparison_buffer);
    }

    #[test]
    /// Test minimum-length attack and decay with high sustain and note release
    /// on fourth sample, minimum length release.
    /// The frame duration used in the test is 1 second. We advance 1 second
    /// before computing the first sample, reaching the sustain immediately.
    fn test_ar_impulse_sustain_one_with_note_press_and_release_impulse() {
        let mut engine_events = Vec::new();
        engine_events.push((0, EngineEvent::NoteChange {note: Some(0)} ));
        engine_events.push((3, EngineEvent::NoteChange {note: None} ));

        let comparison_buffer = vec![[1.0], [1.0], [1.0], [0.0], [0.0], [0.0]];
        _test(0.02, 0.02, 1.0, 0.02, engine_events, comparison_buffer);
    }

    #[test]
    /// Test minimum-length attack and decay with high sustain and note release
    /// on fourth sample, two-second release.
    /// The frame duration used in the test is 1 second. We advance 1 second
    /// before computing the first sample, reaching the sustain immediately.
    fn test_ar_impulse_sustain_one_with_note_press_and_release_two_seconds() {
        let mut engine_events = Vec::new();
        engine_events.push((0, EngineEvent::NoteChange {note: Some(0)} ));
        engine_events.push((3, EngineEvent::NoteChange {note: None} ));

        let comparison_buffer = vec![[1.0], [1.0], [1.0], [0.5], [0.0], [0.0]];
        _test(0.02, 0.02, 1.0, 2.000, engine_events, comparison_buffer);
    }

    #[test]
    /// Test a slow attack interrupted by note release.
    /// The release phase duration should not change, but its gradient will
    /// start from the amplitude when the note was released (0.5).
    fn test_release_mid_attack() {
        let mut engine_events = Vec::new();
        engine_events.push((0, EngineEvent::NoteChange {note: Some(0)} ));
        engine_events.push((2, EngineEvent::NoteChange {note: None} ));

        let comparison_buffer = vec![
            [0.25], [0.5], [0.375], [0.25], [0.125], [0.0], [0.0]];
        _test(4.0, 0.02, 0.0, 4.0, engine_events, comparison_buffer);
    }

    #[test]
    /// Test a slow attack interrupted by note release.
    /// The release phase duration should not change, but its gradient will
    /// start from the amplitude when the note was released (0.5).
    fn test_release_mid_decay() {
        let mut engine_events = Vec::new();
        engine_events.push((0, EngineEvent::NoteChange {note: Some(0)} ));
        engine_events.push((3, EngineEvent::NoteChange {note: None} ));

        let comparison_buffer = vec![
            [1.0], [0.75], [0.5], [0.375], [0.25], [0.125], [0.0], [0.0]];
        _test(1.0, 4.0, 0.0, 4.0, engine_events, comparison_buffer);
    }

    #[test]
    /// Test pressing a note during a long release.
    /// The attack phase duration should not change, but its gradient will
    /// start from the amplitude when the note was released (0.5).
    fn test_retrigger_mid_release() {
        let mut engine_events = Vec::new();
        engine_events.push((0, EngineEvent::NoteChange {note: Some(0)} ));
        engine_events.push((4, EngineEvent::NoteChange {note: None} ));
        engine_events.push((6, EngineEvent::NoteChange {note: Some(0)} ));

        let comparison_buffer = vec![
            [0.25], [0.5], [0.75], [1.0], // then note off
            [0.75], [0.5], // then note on and hold
            [0.625], [0.75], [0.875], [1.0]];
        _test(4.0, 1.0, 0.0, 4.0, engine_events, comparison_buffer);
    }

    /// This function abstracts some test functionality around the Adsr.process_buffer method.
    /// params: optional params where the Baselisk defaults need to be overridden
    /// engine_events: a vector containing (frame_num, EngineEvent) pairs to iterate over.
    /// comparison_buffer: a vector of MonoFrames to compare the results against.
    ///                    the test will generate the same number of samples as the length
    ///                    of the comparison buffer.
    fn _test(attack_duration: defs::Sample,
             decay_duration: defs::Sample,
             sustain_level: defs::Sample,
             release_duration: defs::Sample,
             engine_events: Vec<(usize, EngineEvent)>,
             comparison_buffer: Vec<defs::MonoFrame>) {

        // setup
        let mut adsr = Adsr::new();
        let sample_rate = 1.0;

        let params = BaseliskPluginParameters::default();
        params.update_real_value_from_string(
            ParameterId::AdsrAttack, format!("{}", attack_duration)).unwrap();
        params.update_real_value_from_string(
            ParameterId::AdsrDecay, format!("{}", decay_duration)).unwrap();
        params.update_real_value_from_string(
            ParameterId::AdsrSustain, format!("{}", sustain_level)).unwrap();
        params.update_real_value_from_string(
            ParameterId::AdsrRelease, format!("{}", release_duration)).unwrap();

        let mut buffer = vec![[0.0]; comparison_buffer.len()];

        // test
        adsr.process_buffer(&mut buffer, engine_events.iter(), sample_rate, &params);

        // verify results
        for i in 0..buffer.len() {
            let error_abs = defs::Sample::abs(buffer[i][0] - comparison_buffer[i][0]);
            if error_abs > std::f32::EPSILON {
                panic!("For sample index {}, actual output == {}, expected == {}, absolute error = {}",
                       i, buffer[i][0], comparison_buffer[i][0], error_abs);
            }
        }
    }
}
