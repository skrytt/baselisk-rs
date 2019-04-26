use defs;
use event::{EngineEvent,
            ModulatableParameter,
            ModulatableParameterUpdateData,
};
use parameter::{Parameter, LinearParameter};
use std::slice;

/// States that ADSR can be in
enum AdsrStages {
    Off,         // No note is held and any release phase has ended
    HeldAttack,  // Attack
    HeldDecay,   // Decay
    HeldSustain, // Sustain
    Released,    // Release
}

/// Struct to hold user configurable parameters for an ADSR processor
pub struct AdsrParams {
    attack_duration: LinearParameter,
    decay_duration: LinearParameter,
    sustain_level: LinearParameter,
    release_duration: LinearParameter,
}

impl AdsrParams {
    pub fn new() -> AdsrParams {
        AdsrParams {
            attack_duration: LinearParameter::new(0.0, 10.0, 0.02),
            decay_duration: LinearParameter::new(0.0, 10.0, 0.707),
            sustain_level: LinearParameter::new(0.0, 1.0, 0.0),
            release_duration: LinearParameter::new(0.0, 10.0, 0.4),
        }
    }
}

/// Struct to hold the current state of an ADSR processor
struct AdsrState {
    stage: AdsrStages,
    notes_held_count: i32,
    gain_at_stage_start: f32,
    relative_gain_at_stage_end: f32,
    sample_duration: f32,
    phase_time: f32,
    selected_note: Option<u8>,
}

/// An ADSR struct with all the bits plugged together:
pub struct Adsr {
    params: AdsrParams,
    state: AdsrState,
}

impl Adsr {
    pub fn new() -> Adsr {
        Adsr {
            params: AdsrParams::new(),
            state: AdsrState {
                stage: AdsrStages::Off,
                notes_held_count: 0,
                gain_at_stage_start: 0.0,
                relative_gain_at_stage_end: 0.0,
                sample_duration: 1.0, // Needs to be set by update_sample_rate
                phase_time: 0.0,
                selected_note: None,
            },
        }
    }

    pub fn update_attack(&mut self, data: ModulatableParameterUpdateData)
                         -> Result<(), &'static str>
    {
        self.params.attack_duration.update_patch(data)
    }

    pub fn update_decay(&mut self, data: ModulatableParameterUpdateData)
                        -> Result<(), &'static str> {
        self.params.decay_duration.update_patch(data)
    }

    pub fn update_sustain(&mut self, data: ModulatableParameterUpdateData)
                          -> Result<(), &'static str> {
        self.params.sustain_level.update_patch(data)
    }

    pub fn update_release(&mut self, data: ModulatableParameterUpdateData)
                          -> Result<(), &'static str> {
        self.params.release_duration.update_patch(data)
    }

    pub fn update_state(&mut self,
                        any_notes_held: bool,
                        current_note_changed: bool)
    {
        // Percussive algorithm
        if any_notes_held && current_note_changed {
            // Transition to attack stage
            match self.state.stage {
                // Otherwise, avoid discontinuities
                _ => {
                    self.state.gain_at_stage_start = self.get_gain();
                    self.state.relative_gain_at_stage_end = 1.0 - self.state.gain_at_stage_start;
                    self.state.phase_time = 0.0;
                }
            }
            self.state.stage = AdsrStages::HeldAttack;
        } else if !any_notes_held {
            // Transition to release stage
            match self.state.stage {
                // This case shouldn't generally happen and is ignored
                AdsrStages::Off => (),
                // Avoid discontinuities
                _ => {
                    self.state.gain_at_stage_start = self.get_gain();
                    self.state.relative_gain_at_stage_end = -self.state.gain_at_stage_start;
                    self.state.phase_time = 0.0;
                }
            }
            self.state.stage = AdsrStages::Released;
        }
    }

    fn get_gain(&self) -> f32 {
        match self.state.stage {
            AdsrStages::Off => 0.0,
            AdsrStages::HeldAttack => {
                self.state.gain_at_stage_start
                + self.state.relative_gain_at_stage_end * (
                    self.state.phase_time / self.params.attack_duration.get())
            }
            AdsrStages::HeldDecay => {
                self.state.gain_at_stage_start
                + self.state.relative_gain_at_stage_end * (
                    self.state.phase_time / self.params.decay_duration.get())
            }
            AdsrStages::HeldSustain => self.params.sustain_level.get(),
            AdsrStages::Released => {
                self.state.gain_at_stage_start
                + self.state.relative_gain_at_stage_end * (
                    self.state.phase_time / self.params.release_duration.get())
            }
        }
    }

    pub fn midi_panic(&mut self) {
        // Release all notes and reset state to "Off"
        self.state.notes_held_count = 0;
        self.state.stage = AdsrStages::Off;
    }

    // Process the buffer of audio.
    // Return a bool indicating whether the output contains non-zero samples.
    pub fn process_buffer(&mut self,
                          buffer: &mut defs::MonoFrameBufferSlice,
                          mut engine_event_iter: slice::Iter<(usize, EngineEvent)>,
                          sample_rate: defs::Sample) -> bool
    {
        self.state.sample_duration = 1.0 / sample_rate as f32;

        let mut any_nonzero_output = match self.state.stage {
            AdsrStages::Off => false,
            _ => true,
        };

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
                        EngineEvent::NoteChange{ .. } => (),
                        EngineEvent::ModulateParameter { parameter, .. } => match parameter {
                            ModulatableParameter::AdsrAttack => (),
                            ModulatableParameter::AdsrDecay => (),
                            ModulatableParameter::AdsrSustain => (),
                            ModulatableParameter::AdsrRelease => (),
                            _ => continue,
                        },
                        _ => continue,
                    }
                    next_keyframe = *frame_num;
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
                        *sample = self.advance();
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

                        // If any note is pressed, assume this means there will be some output
                        any_nonzero_output = any_notes_held_next;

                        self.update_state(any_notes_held_next, current_note_changed_next);

                        self.state.selected_note = *note;
                    },
                    EngineEvent::ModulateParameter { parameter, value } => match parameter {
                        ModulatableParameter::AdsrAttack => {
                            self.params.attack_duration.update_cc(*value);
                        },
                        ModulatableParameter::AdsrDecay => {
                            self.params.decay_duration.update_cc(*value);
                        },
                        ModulatableParameter::AdsrSustain => {
                            self.params.sustain_level.update_cc(*value);
                        },
                        ModulatableParameter::AdsrRelease => {
                            self.params.release_duration.update_cc(*value);
                        },
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

    fn advance(&mut self) -> defs::Sample {
        self.state.phase_time += self.state.sample_duration;
        // Handle phase advancing
        match self.state.stage {
            AdsrStages::HeldAttack => {
                if self.state.phase_time >= self.params.attack_duration.get() {
                    self.state.stage = AdsrStages::HeldDecay;
                    self.state.gain_at_stage_start = 1.0;
                    self.state.relative_gain_at_stage_end =
                        self.params.sustain_level.get() - self.state.gain_at_stage_start;
                    self.state.phase_time -= self.params.attack_duration.get();
                }
            }
            AdsrStages::HeldDecay => {
                if self.state.phase_time >= self.params.decay_duration.get() {
                    self.state.stage = AdsrStages::HeldSustain;
                }
            }
            AdsrStages::HeldSustain => return self.params.sustain_level.get(),
            AdsrStages::Released => {
                if self.state.phase_time >= self.params.release_duration.get() {
                    self.state.stage = AdsrStages::Off;
                }
            }
            _ => (),
        }
        self.get_gain()
    }
}
