use defs;
use event::{EngineEvent, MidiEvent};

/// A modulation matrix implementation.
/// Routes MIDI CC message data to the appropriate SingleController instance.
#[derive(Default)]
pub struct ModulationMatrix
{
    controllers: Vec<SingleController>,
    parameter_to_learn: Option<i32>,
}

impl ModulationMatrix
{
    pub fn new() -> Self {
        Self {
            controllers: vec![SingleController::new(); 128], // Space for CCs 0 through 127
            parameter_to_learn: None,
        }
    }

    /// Set that a parameter should be bound to the next MIDI CC received.
    pub fn learn_parameter(&mut self, parameter: i32)
                           -> Result<(), &'static str>
    {
        self.parameter_to_learn = Some(parameter);
        Ok(())
    }

    pub fn bind_parameter(&mut self, number: u8, parameter: i32)
                          -> Result<(), &'static str>
    {
        let controller = self.controllers.get_mut(number as usize).unwrap();

        // Printing from audio thread is a bad idea, but leaving this in for now
        // for debugging purposes. TODO: remove.
        println!("Binding SingleController (CC {}) to {:?}", number, parameter);

        controller.bind(parameter);
        Ok(())
    }

    /// Process a MidiEvent.
    /// Maybe emit an EngineEvent::ModulateParameter.
    pub fn process_event(&mut self, event: &MidiEvent) -> Option<EngineEvent> {
        if let MidiEvent::ControlChange { number, value } = event {
            if self.parameter_to_learn.is_some() {
                let parameter = self.parameter_to_learn.take().unwrap();
                self.bind_parameter(*number, parameter).unwrap();
            } else {
                let controller = self.controllers.get(*number as usize).unwrap();
                return controller.process(*value)
            }
        }
        None
    }
}

/// A handler for messages from a single MIDI CC controller.
/// A SingleController can modulate a single parameter.
#[derive(Clone, Default)]
struct SingleController
{
    param_id: Option<i32>,
}

impl SingleController
{
    pub fn new() -> Self {
        Self {
            param_id: None,
        }
    }

    /// Bind this controller to a ModulatableParameter.
    pub fn bind(&mut self, param_id: i32) {
        self.param_id = Some(param_id);
    }

    /// Process an incoming MIDI CC value.
    /// Maybe emit an EngineEvent::ModulateParameter.
    pub fn process(&self, cc_value: u8) -> Option<EngineEvent> {
        if let Some(ref param_id) = self.param_id {
            let param_id = param_id.clone();

            // Convert the MIDI value into a value in the range 0.0 <= val <= 1.0
            let value = defs::Sample::from(cc_value) / 127.0;

            return Some(EngineEvent::ModulateParameter{ param_id, value })
        }
        None
    }
}
