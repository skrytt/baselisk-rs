use defs;

/// These events represent things the audio thread will do for us while it is running,
/// to avoid unsafe data access.
pub enum PatchEvent {
    PitchBendRangeSet { semitones: defs::Sample },
    OscillatorTypeSet { type_name: String },
    ModulatableParameterUpdate { parameter: ModulatableParameter,
                                 data: ModulatableParameterUpdateData },
    ControllerBindUpdate { parameter: ModulatableParameter,
                           bind_type: ControllerBindData },
}

/// Enum of modulatable parameters.
#[derive(Clone, Debug)]
pub enum ModulatableParameter {
    AdsrAttack,
    AdsrDecay,
    AdsrSustain,
    AdsrRelease,
    DelayFeedback,
    DelayHighPassFilterFrequency,
    DelayHighPassFilterQuality,
    DelayLowPassFilterFrequency,
    DelayLowPassFilterQuality,
    FilterFrequency,
    FilterQuality,
    FilterSweepRange,
    OscillatorPitch,
    OscillatorPulseWidth,
    WaveshaperInputGain,
    WaveshaperOutputGain,
}

/// Types of float parameter updates
pub enum ModulatableParameterUpdateData {
    Base(defs::Sample),
    Max(defs::Sample),
}

pub enum ControllerBindData {
    CliInput(u8),
    MidiLearn,
}

