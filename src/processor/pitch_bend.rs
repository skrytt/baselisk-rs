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
