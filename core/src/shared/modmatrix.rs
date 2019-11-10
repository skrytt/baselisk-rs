use defs;
use shared::{
    event::{
        EngineEvent,
        MidiEvent
    },
    parameter::{
        BaseliskPluginParameters,
        ParameterId,
    },
};
use std::sync::{
    Arc,
    atomic::{AtomicI32, Ordering},
};

/// A modulation matrix implementation.
/// Routes MIDI CC message data to the appropriate SingleController instance.
pub struct ModulationMatrix
{
    parameters: Arc<BaseliskPluginParameters>,
    controllers: Vec<SingleController>,
    param_id_to_learn: AtomicI32, // Using -1 to mean None, 0+ to mean Some(value).
                                   // not sure how else to do this in an atomic way...
}

impl ModulationMatrix
{
    pub fn new(parameters: Arc<BaseliskPluginParameters>) -> Self {
        let mut controllers = Vec::with_capacity(128);
        for _ in 0..128 {
            controllers.push(SingleController::new());
        }
        Self {
            parameters,
            controllers,
            param_id_to_learn: AtomicI32::new(-1),
        }
    }

    /// Set that a param_id should be bound to the next MIDI CC received.
    pub fn learn_parameter(&self, param: ParameterId)
    {
        println!("Starting MIDI learn for parameter {}",
                 self.parameters.get_parameter_name(param));
        self.param_id_to_learn.store(param as i32, Ordering::Relaxed);
    }

    pub fn bind_parameter(&self, number: u8, param: ParameterId)
    {
        println!("Binding CC {} to parameter {}",
                 number,
                 self.parameters.get_parameter_name(param));
        self.controllers[number as usize].bind(param as i32);
    }

    /// Process a MidiEvent.
    /// Maybe emit an EngineEvent::ModulateParameter.
    pub fn process_event(&self, event: &MidiEvent) -> Option<EngineEvent> {
        if let MidiEvent::ControlChange { number, value } = event {
            let param_id = self.param_id_to_learn.load(Ordering::Relaxed);
            if param_id >= 0 {
                self.param_id_to_learn.store(-1, Ordering::Relaxed);
                self.bind_parameter(*number, ParameterId::from(param_id));
            } else {
                return self.controllers[*number as usize].process(*value);
            }
        }
        None
    }
}

/// A handler for messages from a single MIDI CC controller.
/// A SingleController can modulate a single param_id.
struct SingleController
{
    param_id: AtomicI32,
}

impl SingleController
{
    pub fn new() -> Self {
        Self {
            param_id: AtomicI32::new(-1), // Using -1 to mean None, 0+ to mean Some(value).
                                          // not sure how else to do this in an atomic way...
        }
    }

    /// Bind this controller to a ModulatableParameter.
    pub fn bind(&self, param_id: i32) {
        self.param_id.store(param_id, Ordering::Relaxed);
    }

    /// Process an incoming MIDI CC value.
    /// Maybe emit an EngineEvent::ModulateParameter.
    pub fn process(&self, cc_value: u8) -> Option<EngineEvent> {
        let param_id = self.param_id.load(Ordering::Relaxed);
        if param_id >= 0 {
            // Convert the MIDI value into a value in the range 0.0 <= val <= 1.0
            let value = defs::Sample::from(cc_value) / 127.0;
            return Some(EngineEvent::ModulateParameter{
                param_id: ParameterId::from(param_id),
                value,
            })
        }
        None
    }
}
