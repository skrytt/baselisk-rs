use defs;
use shared::parameter::{
    BaseliskPluginParameters,
    PARAM_GENERATOR_PITCH_BEND_RANGE,
};

pub fn get_pitch_bend_semitones(midi_pitch_wheel_value: u16,
                                params: &BaseliskPluginParameters) -> defs::Sample
{
    // Value is 14-bit (range 0 <= value <= 16383)
    // For the default range of 2 semitones:
    // 0 => -2
    // 8192 => 0
    // 16383 => ~= +2 (0.012% of a semitone below 2; but who's going to notice?)
    params.get_real_value(PARAM_GENERATOR_PITCH_BEND_RANGE) * (
            defs::Sample::from(midi_pitch_wheel_value) - 8192.0) / 8192.0
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

        let params = BaseliskPluginParameters::default();

        // Optionally set a pitch bend range
        if let Some(range_to_set) = range_to_set {
            let result = params.update_real_value_from_string(
                PARAM_GENERATOR_PITCH_BEND_RANGE, format!("{}", range_to_set));
            assert!(result.is_ok());
        }

        // Simulate what happens when a MIDI pitch bend event is received
        let semitones = get_pitch_bend_semitones(pitch_bend_event_value, &params);

        // Verify result
        let error_abs = defs::Sample::abs(semitones - bend_semitones_to_assert);
        if error_abs > std::f32::EPSILON {
            panic!("Pitch bend {}, absolute error {} (exceeds f32::EPSILON)",
                   semitones, error_abs);
        }
    }
}
