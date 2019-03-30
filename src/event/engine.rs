use defs;
use event::ModulatableParameter;

pub enum EngineEvent {
    NoteChange { note: Option<u8> },
    PitchBend { semitones: defs::Sample },
    ModulateParameter { parameter: ModulatableParameter, value: u8 },
}
