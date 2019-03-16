use defs;
use event::{Buffer, Event, MidiEvent};
use std::rc::Rc;
use std::cell::RefCell;

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
}

/// An ADSR struct with all the bits plugged together:
pub struct Adsr {
    params: AdsrParams,
    state: AdsrState,
    event_buffer: Rc<RefCell<Buffer>>,
}

impl Adsr {
    pub fn new(event_buffer: &Rc<RefCell<Buffer>>) -> Adsr {
        Adsr {
            params: AdsrParams::new(),
            state: AdsrState {
                stage: AdsrStages::Off,
                notes_held_count: 0,
                gain_at_stage_start: 0.0,
                relative_gain_at_stage_end: 0.0,
                sample_duration: 1.0, // Needs to be set by update_sample_rate
                phase_time: 0.0,
            },
            event_buffer: Rc::clone(event_buffer),
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

    pub fn update_notes_held_count(&mut self, notes_pressed: i32, notes_released: i32) {
        let old_notes_held_count = self.state.notes_held_count;
        self.state.notes_held_count += notes_pressed - notes_released;

        // Could happen during all notes/sound off events
        // or if note on messages were missed somehow
        if self.state.notes_held_count <= 0 {
            self.state.notes_held_count = 0;
        }

        // Smooth algorithm
        //if old_notes_held_count == 0 && self.notes_held_count > 0 {
        //    // Transition to attack stage
        //    match self.stage {
        //        // If a note is pressed during an existing note, continue the existing A/D/S
        //        AdsrStages::HeldAttack | AdsrStages::HeldDecay | AdsrStages::HeldSustain => (),
        //        // Otherwise, avoid discontinuities
        //        _ => {
        //            self.gain_at_stage_start = self.get_gain();
        //            self.relative_gain_at_stage_end = 1.0 - self.gain_at_stage_start;
        //            self.phase_time = 0.0;
        //        }
        //    }
        //    self.stage = AdsrStages::HeldAttack;
        //}

        // Percussive algorithm
        if notes_pressed > 0 && self.state.notes_held_count > 0 {
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
        } else if old_notes_held_count > 0 && self.state.notes_held_count == 0 {
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
                      buffer: &mut defs::FrameBuffer,
                      sample_rate: defs::Sample)
    {
        self.set_sample_rate(sample_rate);

        let mut notes_pressed: i32 = 0;
        let mut notes_released: i32 = 0;

        {
            let events = self.event_buffer.borrow();
            for event in events.iter_midi() {
                if let Event::Midi(midi_event) = event {
                    match midi_event {
                        MidiEvent::NoteOn { .. } => {
                            notes_pressed += 1;
                        },
                        MidiEvent::NoteOff { .. } => {
                            notes_released += 1;
                        },
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
        if notes_pressed > 0 || notes_released > 0 {
            self.update_notes_held_count(notes_pressed, notes_released);
        }

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
