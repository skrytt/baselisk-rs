use defs;

pub enum EngineEvent {
    NoteChange { note: Option<u8> },
    PitchBend { semitones: defs::Sample },
    ModulateParameter { param_id: i32, value: f32 },
}
