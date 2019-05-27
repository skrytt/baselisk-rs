use defs;
use event::{EngineEvent, MidiEvent, ModulatableParameterUpdateData};
use parameter::{Parameter, LinearParameter};

pub struct PitchBend {
    range_semitones: LinearParameter,
}

impl PitchBend {
    pub fn new() -> Self {
        Self {
            range_semitones: LinearParameter::new(0.0, 36.0, 2.0),
        }
    }

    pub fn set_range(&mut self, range: defs::Sample)
        -> Result<(), &'static str> {
        // TODO: improve on this hack...
        let data = ModulatableParameterUpdateData::Base(range);
        self.range_semitones.update_patch(data)
    }

    pub fn process_event(&self, midi_event: &MidiEvent) -> Option<EngineEvent> {
        match midi_event {
            MidiEvent::PitchBend { value } => {
                // Value is 14-bit (range 0 <= value <= 16383)
                // For the default range of 2 semitones:
                // 0 => -2
                // 8192 => 0
                // 16383 => ~= +2 (0.012% of a semitone below 2; but who's going to notice?)
                let semitones = self.range_semitones.get_real_value() * (
                        defs::Sample::from(*value) - 8192.0) / 8192.0;
                Some(EngineEvent::PitchBend{ semitones })
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_two_semitones_bend_max() {
        // 16384 is out of the 14-bit range, but easier to test with.
        _test(None, 16384, 2.0);
    }

    #[test]
    fn test_range_two_semitones_no_bend() {
        _test(None, 8192, 0.0);
    }

    #[test]
    fn test_range_two_semitones_bend_min() {
        _test(None, 0, -2.0);
    }

    #[test]
    fn test_range_octave_bend_max() {
        // 16384 is out of the 14-bit range, but easier to test with.
        _test(Some(12.0), 16384, 12.0);
    }

    #[test]
    fn test_range_octave_no_bend() {
        _test(Some(12.0), 8192, 0.0);
    }

    #[test]
    fn test_range_octave_bend_min() {
        _test(Some(12.0), 0, -12.0);
    }

    #[test]
    fn test_range_low_limit_0_semitones() {
        _test(Some(-12.0), 0, 0.0);
    }

    #[test]
    fn test_range_high_limit_36_semitones() {
        _test(Some(48.0), 16384, 36.0);
    }

    fn _test(range_to_set: Option<defs::Sample>,
             pitch_bend_event_value: u16,
             bend_semitones_to_assert: defs::Sample) {

        let mut pitch_bend = PitchBend::new();

        // Optionally set a pitch bend range
        if let Some(range_to_set) = range_to_set {
            let result = pitch_bend.set_range(range_to_set);
            assert!(result.is_ok());
        }

        // Simulate what happens when a MIDI pitch bend event is received
        let engine_event = pitch_bend.process_event(
            &MidiEvent::PitchBend { value: pitch_bend_event_value });
        assert!(engine_event.is_some());

        // Extract the pitch bend amount in semitones and assert it's acceptable
        if let EngineEvent::PitchBend { semitones } = engine_event.unwrap() {
            let error_abs = defs::Sample::abs(semitones - bend_semitones_to_assert);
            if error_abs > std::f32::EPSILON {
                panic!("Pitch bend {}, absolute error {} (exceeds f32::EPSILON)",
                       semitones, error_abs);
            }
        } else {
            panic!("EngineEvent variant was not PitchBend");
        }

    }
}
