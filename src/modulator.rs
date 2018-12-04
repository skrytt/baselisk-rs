/// States that ADSR can be in
enum AdsrStages {
    Off,         // No note is held and any release phase has ended
    HeldAttack,  // Attack
    HeldDecay,   // Decay
    HeldSustain, // Sustain
    Released,    // Release
}

struct AdsrParams {
    attack_duration: f32,
    decay_duration: f32,
    sustain_level: f32,
    release_duration: f32,
}

pub struct Adsr {
    params: AdsrParams,
    stage: AdsrStages,
    notes_held_count: i32,
    gain_at_stage_start: f32,
    relative_gain_at_stage_end: f32,
    sample_duration: f32,
    phase_time: f32,
}

impl Adsr {
    pub fn new() -> Adsr {
        Adsr {
            params: AdsrParams {
                attack_duration: 0.02,
                decay_duration: 0.707,
                sustain_level: 0.0,
                release_duration: 0.707,
            },
            stage: AdsrStages::Off,
            notes_held_count: 0,
            gain_at_stage_start: 0.0,
            relative_gain_at_stage_end: 0.0,
            sample_duration: 1.0, // Needs to be set by update_sample_rate
            phase_time: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_duration = 1.0 / sample_rate as f32;
    }

    pub fn update_notes_held_count(&mut self, notes_pressed: i32, notes_released: i32) {
        let old_notes_held_count = self.notes_held_count;
        self.notes_held_count += notes_pressed - notes_released;

        // Shouldn't happen but could do if MIDI events are missed
        if self.notes_held_count <= 0 {
            self.notes_held_count = 0;
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
        if notes_pressed > 0 && self.notes_held_count > 0 {
            // Transition to attack stage
            match self.stage {
                // Otherwise, avoid discontinuities
                _ => {
                    self.gain_at_stage_start = self.get_gain();
                    self.relative_gain_at_stage_end = 1.0 - self.gain_at_stage_start;
                    self.phase_time = 0.0;
                }
            }
            self.stage = AdsrStages::HeldAttack;
        } else if old_notes_held_count > 0 && self.notes_held_count == 0 {
            // Transition to release stage
            match self.stage {
                // This case shouldn't generally happen and is ignored
                AdsrStages::Off => (),
                // Avoid discontinuities
                _ => {
                    self.gain_at_stage_start = self.get_gain();
                    self.relative_gain_at_stage_end = -self.gain_at_stage_start;
                    self.phase_time = 0.0;
                }
            }
            self.stage = AdsrStages::Released;
        }
    }

    fn get_gain(&self) -> f32 {
        loop {
            match self.stage {
                AdsrStages::Off => return 0.0,
                AdsrStages::HeldAttack => {
                    return self.gain_at_stage_start
                        + self.relative_gain_at_stage_end
                            * (self.phase_time / self.params.attack_duration)
                }
                AdsrStages::HeldDecay => {
                    return self.gain_at_stage_start
                        + self.relative_gain_at_stage_end
                            * (self.phase_time / self.params.decay_duration)
                }
                AdsrStages::HeldSustain => return self.params.sustain_level,
                AdsrStages::Released => {
                    return self.gain_at_stage_start
                        + self.relative_gain_at_stage_end
                            * (self.phase_time / self.params.release_duration)
                }
            }
        }
    }
}

impl Iterator for Adsr {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.phase_time += self.sample_duration;
        // Handle phase advancing
        match self.stage {
            AdsrStages::HeldAttack => {
                if self.phase_time >= self.params.attack_duration {
                    self.stage = AdsrStages::HeldDecay;
                    self.gain_at_stage_start = 1.0;
                    self.relative_gain_at_stage_end =
                        self.params.sustain_level - self.gain_at_stage_start;
                    self.phase_time -= self.params.attack_duration;
                }
            }
            AdsrStages::HeldDecay => {
                if self.phase_time >= self.params.decay_duration {
                    self.stage = AdsrStages::HeldSustain;
                }
            }
            AdsrStages::HeldSustain => return Some(self.params.sustain_level),
            AdsrStages::Released => {
                if self.phase_time >= self.params.release_duration {
                    self.stage = AdsrStages::Off;
                }
            }
            _ => (),
        }
        return Some(self.get_gain());
    }
}
