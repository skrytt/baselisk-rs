use event::{EngineEvent, MidiEvent, ModulatableParameter};

/// A modulation matrix implementation.
/// Routes MIDI CC message data to the appropriate SingleController instance.
pub struct ModulationMatrix
{
    controllers: Vec<SingleController>,
    parameter_to_learn: Option<ModulatableParameter>,
}

impl ModulationMatrix
{
    pub fn new() -> ModulationMatrix {
        ModulationMatrix {
            controllers: vec![SingleController::new(); 128], // Space for CCs 0 through 127
            parameter_to_learn: None,
        }
    }

    /// Set that a parameter should be bound to the next MIDI CC received.
    pub fn learn_parameter(&mut self, parameter: ModulatableParameter)
                           -> Result<(), &'static str> 
    {
        self.parameter_to_learn = Some(parameter);
        Ok(())
    }

    pub fn bind_cc(&mut self, number: u8) {
        let controller = self.controllers.get_mut(number as usize).unwrap();
        let parameter = self.parameter_to_learn.take().unwrap();

        // Printing from audio thread is a bad idea, but leaving this in for now
        // for debugging purposes. TODO: remove.
        println!("Binding SingleController (CC {}) to {:?}", number, parameter);

        controller.bind(parameter);
        self.parameter_to_learn = None;
    }

    /// Process a MidiEvent.
    /// Maybe emit an EngineEvent::ModulateParameter.
    pub fn process_event(&mut self, event: &MidiEvent) -> Option<EngineEvent> {
        if let MidiEvent::ControlChange { number, value } = event {
            let is_midi_learn = self.parameter_to_learn.is_some();
            match is_midi_learn {
                false => {
                    let controller = self.controllers.get(*number as usize).unwrap();
                    return controller.process(*value)
                },
                true => {
                    self.bind_cc(*number);
                }
            }
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

    /// Bind this controller to a ModulatableParameter.
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
