use defs;

/// TODO: rename PatchEvent to ModelEvent?
/// These events represent things the audio thread will do for us while it is running,
/// to avoid unsafe data access.
pub enum PatchEvent {
    MidiDeviceSet {
        device_id: i32,
    },
    OscillatorTypeSet {
        type_name: String,
    },
    OscillatorPitchSet {
        semitones: defs::Sample,
    },
    OscillatorPulseWidthSet {
        width: defs::Sample,
    },
    AdsrAttackSet {
        duration: defs::Sample,
    },
    AdsrDecaySet {
        duration: defs::Sample,
    },
    AdsrSustainSet {
        level: defs::Sample,
    },
    AdsrReleaseSet {
        duration: defs::Sample,
    },
    FilterFrequencySet {
        hz: defs::Sample,
    },
    FilterQualitySet {
        q: defs::Sample,
    },
}

