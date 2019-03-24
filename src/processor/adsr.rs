use defs;
use event::{Event, MidiEvent};
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
#[derive(Clone)]
pub struct AdsrParams {
    attack_duration: f32,
    decay_duration: f32,
    sustain_level: f32,
    release_duration: f32,
}

impl AdsrParams {
    pub fn new() -> AdsrParams {
        AdsrParams {
            attack_duration: 0.02,
            decay_duration: 0.707,
            sustain_level: 0.0,
            release_duration: 0.707,
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
    last_current_note: Option<u8>,
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
                last_current_note: None,
            },
        }
    }

    pub fn set_attack(&mut self, duration: defs::Sample) -> Result<(), &'static str> {
        if duration < 0.0 {
            return Err("attack duration must be >= 0.0");
        }
        self.params.attack_duration = duration;
        Ok(())
    }

    pub fn set_decay(&mut self, duration: defs::Sample) -> Result<(), &'static str> {
        if duration < 0.0 {
            return Err("decay duration must be >= 0.0");
        }
        self.params.decay_duration = duration;
        Ok(())
    }

    pub fn set_sustain(&mut self, level: defs::Sample) -> Result<(), &'static str> {
        if level < 0.0 || level > 1.0 {
            return Err("sustain level must be 0.0 >= level >= 1.0");
        }
        self.params.sustain_level = level;
        Ok(())
    }

    pub fn set_release(&mut self, duration: defs::Sample) -> Result<(), &'static str> {
        if duration < 0.0 {
            return Err("release duration must be >= 0.0");
        }
        self.params.release_duration = duration;
        Ok(())
    }

    fn set_sample_rate(&mut self, sample_rate: defs::Sample) {
        self.state.sample_duration = 1.0 / sample_rate as f32;
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
        loop {
            match self.state.stage {
                AdsrStages::Off => return 0.0,
                AdsrStages::HeldAttack => {
                    return self.state.gain_at_stage_start
                        + self.state.relative_gain_at_stage_end
                            * (self.state.phase_time / self.params.attack_duration)
                }
                AdsrStages::HeldDecay => {
                    return self.state.gain_at_stage_start
                        + self.state.relative_gain_at_stage_end
                            * (self.state.phase_time / self.params.decay_duration)
                }
                AdsrStages::HeldSustain => return self.params.sustain_level,
                AdsrStages::Released => {
                    return self.state.gain_at_stage_start
                        + self.state.relative_gain_at_stage_end
                            * (self.state.phase_time / self.params.release_duration)
                }
            }
        }
    }

    pub fn process_buffer(&mut self,
                          buffer: &mut defs::MonoFrameBufferSlice,
                          selected_note_iter: slice::Iter<(usize, Option<u8>)>,
                          midi_iter: slice::Iter<(usize, Event)>,
                          sample_rate: defs::Sample)
    {
        self.set_sample_rate(sample_rate);

        let mut selected_note = self.state.last_current_note;

        // TODO: needs to account for times of events.
        for (_frame_num, note) in selected_note_iter {
            selected_note = *note;
        }
        let any_notes_held: bool = selected_note.is_some();
        let current_note_changed: bool = selected_note != self.state.last_current_note;
        self.state.last_current_note = selected_note;

        {
            for (_frame_num, event) in midi_iter {
                if let Event::Midi(midi_event) = event {
                    match midi_event {
                        MidiEvent::AllNotesOff | MidiEvent::AllSoundOff => {
                            // Release all notes and reset state to "Off"
                            self.state.notes_held_count = 0;
                            self.state.stage = AdsrStages::Off;
                        },

                        _ => (),
                    }
                }
            }
        }
        self.update_state(any_notes_held, current_note_changed);

        for frame in buffer.iter_mut() {
            let value = self.next().unwrap();

            assert!(value >= 0.0);
            assert!(value <= 1.0);

            for sample in frame.iter_mut() {
                *sample = value;
            }
        }
    }
}

impl Iterator for Adsr {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.state.phase_time += self.state.sample_duration;
        // Handle phase advancing
        match self.state.stage {
            AdsrStages::HeldAttack => {
                if self.state.phase_time >= self.params.attack_duration {
                    self.state.stage = AdsrStages::HeldDecay;
                    self.state.gain_at_stage_start = 1.0;
                    self.state.relative_gain_at_stage_end =
                        self.params.sustain_level - self.state.gain_at_stage_start;
                    self.state.phase_time -= self.params.attack_duration;
                }
            }
            AdsrStages::HeldDecay => {
                if self.state.phase_time >= self.params.decay_duration {
                    self.state.stage = AdsrStages::HeldSustain;
                }
            }
            AdsrStages::HeldSustain => return Some(self.params.sustain_level),
            AdsrStages::Released => {
                if self.state.phase_time >= self.params.release_duration {
                    self.state.stage = AdsrStages::Off;
                }
            }
            _ => (),
        }
        return Some(self.get_gain());
    }
}
