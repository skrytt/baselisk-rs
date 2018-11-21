
/// States that ADSR can be in
enum AdsrStages {
    Off,        // No note is held and any release phase has ended
    HeldAttack, // Attack
    HeldDecay,  // Decay
    HeldSustain,// Sustain
    Released,   // Release
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
    sample_duration: f32,
    time: f32,
}

impl Adsr {
    pub fn new(sample_rate: f64) -> Adsr {
        Adsr{
            params: AdsrParams {
                attack_duration: 0.0,
                decay_duration: 0.707,
                sustain_level: 0.33,
                release_duration: 0.303,
            },
            stage: AdsrStages::Off,
            sample_duration: 1.0 / sample_rate as f32,
            time: 0.0,
        }
    }

    pub fn start_attack(&mut self) {
        // This isn't right but is a starting point.
        // More can be done to avoid discontinuities during key presses/releases.
        println!("debug: start_attack called");
        self.time = 0.0;
        self.stage = AdsrStages::HeldAttack;
    }

    pub fn start_release(&mut self) {
        // This isn't right but is a starting point.
        // More can be done to avoid discontinuities during key presses/releases.
        println!("debug: start_release called");
        self.time = 0.0;
        self.stage = AdsrStages::Released;
    }
}

impl Iterator for Adsr
{
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.time += self.sample_duration;
        loop {
            match self.stage {
                AdsrStages::Off => return Some(0.0),
                AdsrStages::HeldAttack => {
                    if self.time >= self.params.attack_duration {
                        self.stage = AdsrStages::HeldDecay;
                        continue
                    }
                    return Some(self.time / self.params.attack_duration)
                },
                AdsrStages::HeldDecay => {
                    if self.time >= self.params.attack_duration + self.params.decay_duration {
                        self.stage = AdsrStages::HeldSustain;
                        continue
                    }
                    return Some(1.0 - ((1.0 - self.params.sustain_level) * ((self.time - self.params.attack_duration) / self.params.decay_duration)))
                },
                AdsrStages::HeldSustain => return Some(self.params.sustain_level),
                AdsrStages::Released => {
                    if self.time >= self.params.release_duration {
                        self.stage = AdsrStages::Off;
                        continue
                    }
                    return Some(self.params.sustain_level * (1.0 - self.time / self.params.release_duration))
                },
            }
        }
    }

}
