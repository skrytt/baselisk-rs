use super::super::parameter;

pub enum EngineEvent {
    NoteChange { note: Option<u8> },
    PitchBend { wheel_value: u16 },
    ModulateParameter { param_id: parameter::ParameterId, value: f32 },
}
