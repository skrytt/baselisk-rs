use defs;

/// These events represent things the audio thread will do for us while it is running,
/// to avoid unsafe data access.
pub enum PatchEvent {
    PitchBendRangeSet { semitones: defs::Sample },
    OscillatorTypeSet { type_name: String },
    ModulatableParameterUpdate { param_id: i32,
                                 data: ModulatableParameterUpdateData },
    ControllerBindUpdate { param_id: i32,
                           bind_type: ControllerBindData },
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

