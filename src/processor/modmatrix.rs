use event::{EngineEvent, MidiEvent, ModulatableParameter};

/// A modulation matrix implementation.
/// Routes MIDI CC message data to the appropriate SingleController instance.
pub struct ModulationMatrix
{
    controllers: Vec<SingleController>,
}

impl ModulationMatrix
{
    pub fn new() -> ModulationMatrix {
        ModulationMatrix {
            controllers: vec![SingleController::new(); 128], // Space for CCs 0 through 127
        }
    }

    pub fn bind_cc(&mut self, number: u8, parameter: ModulatableParameter) {
        let controller = self.controllers.get_mut(number as usize).unwrap();
        controller.bind(parameter);
    }

    /// Process a MidiEvent.
    /// Maybe emit an EngineEvent::ModulateParameter.
    pub fn process_event(&self, event: &MidiEvent) -> Option<EngineEvent> {
        if let MidiEvent::ControlChange { number, value } = event {
            let controller = self.controllers.get(*number as usize).unwrap();
            return controller.process(*value)
        }
        None
    }
}

/// A handler for messages from a single MIDI CC controller.
/// A SingleController can modulate a single parameter.
#[derive(Clone)]
struct SingleController
{
    parameter: Option<ModulatableParameter>,
}

impl SingleController
{
    pub fn new() -> SingleController {
        SingleController {
            parameter: None,
        }
    }

    /// Bind a CC number to a target Parameter.
    pub fn bind(&mut self, parameter: ModulatableParameter) {
        self.parameter = Some(parameter);
    }

    /// Process an incoming MIDI CC value.
    /// Maybe emit an EngineEvent::ModulateParameter.
    pub fn process(&self, value: u8) -> Option<EngineEvent> {
        if let Some(ref parameter) = self.parameter {
            let parameter = parameter.clone();
            return Some(EngineEvent::ModulateParameter{ parameter, value })
        }
        None
    }
}
