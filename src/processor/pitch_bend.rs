use defs;
use event::{EngineEvent, MidiEvent};
use parameter::{Parameter, LinearParameter};

pub struct PitchBend {
    range_semitones: LinearParameter,
}

impl PitchBend {
    pub fn new() -> PitchBend {
        PitchBend {
            range_semitones: LinearParameter::new(2.0),
        }
    }

    pub fn set_range(&mut self, semitones: defs::Sample) -> Result<(), &'static str> {
        if semitones < 0.0 {
            Err("Pitch bend must be >= 0.0 semitones")
        } else if semitones > 36.0 {
            Err("Pitch bend must be <= 36.0 semitones")
        } else {
            self.range_semitones.set_base(semitones);
            Ok(())
        }
    }

    pub fn process_event(&self, midi_event: &MidiEvent) -> Option<EngineEvent> {
        match midi_event {
            MidiEvent::PitchBend { value } => {
                // Value is 14-bit (range 0 <= value <= 16383)
                // 0 => -2
                // 8192 => 0
                // 16383 => ~= +2
                let semitones = self.range_semitones.get() * (
                        *value as defs::Sample - 8192.0) / 8192.0;
                Some(EngineEvent::PitchBend{ semitones })
            },
            _ => None,
        }
    }
}
