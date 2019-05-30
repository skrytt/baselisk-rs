use defs;

/// These events represent things the audio thread will do for us while it is running,
/// to avoid unsafe data access.
pub enum PatchEvent {
    PitchBendRangeSet { semitones: defs::Sample },
    ModulatableParameterUpdate { param_id: i32,
                                 value_string: String },
    ControllerBindUpdate { param_id: i32,
                           bind_type: ControllerBindData },
}

pub enum ControllerBindData {
    CliInput(u8),
    MidiLearn,
}

