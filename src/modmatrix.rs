use defs;
use event::{EngineEvent, MidiEvent};

use std::sync::atomic::{AtomicI32, Ordering};

/// A modulation matrix implementation.
/// Routes MIDI CC message data to the appropriate SingleController instance.
pub struct ModulationMatrix
{
    controllers: Vec<SingleController>,
    parameter_to_learn: AtomicI32, // Using -1 to mean None, 0+ to mean Some(value).
                                   // not sure how else to do this in an atomic way...
}

impl ModulationMatrix
{
    pub fn new() -> Self {
        let mut controllers = Vec::with_capacity(128);
        for _ in 0..128 {
            controllers.push(SingleController::new());
        }
        Self {
            controllers,
            parameter_to_learn: AtomicI32::new(-1),
        }
    }

    pub fn parameter_to_learn(&self) -> i32 {
        self.parameter_to_learn.load(Ordering::Relaxed)
    }

    /// Set that a parameter should be bound to the next MIDI CC received.
    pub fn learn_parameter(&self, parameter: i32)
    {
        println!("Starting MIDI learn for parameter {:?}", parameter);
        self.parameter_to_learn.store(parameter, Ordering::Relaxed);
    }

    pub fn bind_parameter(&self, number: u8, parameter: i32)
    {
        println!("Binding SingleController (CC {}) to {:?}", number, parameter);
        self.controllers[number as usize].bind(parameter);
    }

    /// Process a MidiEvent.
    /// Maybe emit an EngineEvent::ModulateParameter.
    pub fn process_event(&self, event: &MidiEvent) -> Option<EngineEvent> {
        if let MidiEvent::ControlChange { number, value } = event {
            let parameter = self.parameter_to_learn.load(Ordering::Relaxed);
            if parameter >= 0 {
                self.parameter_to_learn.store(-1, Ordering::Relaxed);
                self.bind_parameter(*number, parameter);
            } else {
                return self.controllers[*number as usize].process(*value);
            }
        }
        None
    }
}

/// A handler for messages from a single MIDI CC controller.
/// A SingleController can modulate a single parameter.
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
                param_id: param_id.clone(),
                value,
            })
        }
        None
    }
}
