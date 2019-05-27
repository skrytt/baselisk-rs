use defs;
use event::{
    EngineEvent,
};
use std::slice;
use parameter::{
    BaseliskPluginParameters,
    PARAM_ADSR_ATTACK,
    PARAM_ADSR_DECAY,
    PARAM_ADSR_SUSTAIN,
    PARAM_ADSR_RELEASE,
};
use vst::plugin::PluginParameters;

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
                    self.state.phase_time / params.get_real_value(PARAM_ADSR_ATTACK))
            }
            Some(AdsrStages::HeldDecay) => {
                self.state.gain_at_stage_start
                + self.state.relative_gain_at_stage_end * (
                    self.state.phase_time / params.get_real_value(PARAM_ADSR_DECAY))
            }
            Some(AdsrStages::HeldSustain) => params.get_real_value(PARAM_ADSR_SUSTAIN),
            Some(AdsrStages::Released) => {
                self.state.gain_at_stage_start
                + self.state.relative_gain_at_stage_end * (
                    self.state.phase_time / params.get_real_value(PARAM_ADSR_RELEASE))
            }
        }
    }

    pub fn midi_panic(&mut self) {
        // Release all notes and reset state to "Off"
        self.state.notes_held_count = 0;
        self.state.stage = None;
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
                        PARAM_ADSR_ATTACK |
                        PARAM_ADSR_DECAY |
                        PARAM_ADSR_SUSTAIN |
                        PARAM_ADSR_RELEASE => (),
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
                        PARAM_ADSR_ATTACK |
                        PARAM_ADSR_DECAY |
                        PARAM_ADSR_SUSTAIN |
                        PARAM_ADSR_RELEASE => params.set_parameter(*param_id, *value),
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
        // Handle phase advancing
        match self.state.stage {
            Some(AdsrStages::HeldAttack) => {
                if self.state.phase_time >= params.get_real_value(PARAM_ADSR_ATTACK) {
                    self.state.stage = Some(AdsrStages::HeldDecay);
                    self.state.gain_at_stage_start = 1.0;
                    self.state.relative_gain_at_stage_end =
                        params.get_real_value(PARAM_ADSR_SUSTAIN) - self.state.gain_at_stage_start;
                    self.state.phase_time -= params.get_real_value(PARAM_ADSR_ATTACK);
                }
            }
            Some(AdsrStages::HeldDecay) => {
                if self.state.phase_time >= params.get_real_value(PARAM_ADSR_DECAY) {
                    self.state.stage = Some(AdsrStages::HeldSustain);
                }
            }
            Some(AdsrStages::HeldSustain) => return params.get_real_value(PARAM_ADSR_SUSTAIN),
            Some(AdsrStages::Released) => {
                if self.state.phase_time >= params.get_real_value(PARAM_ADSR_RELEASE) {
                    self.state.stage = None;
                }
            }
            _ => (),
        }
        self.get_gain(params)
    }
}
