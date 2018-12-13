
use application;
use event::types;
use std::sync::mpsc;

/// Buffer will contain events sent over the comms channel in the last block.
pub struct InputBuffer {
    receiver: mpsc::Receiver<types::Event>,
}

impl InputBuffer {
    /// Create a new buffer for receiving patch events from user input.
    pub fn new(receiver: mpsc::Receiver<types::Event>) -> InputBuffer {
        InputBuffer {
            receiver
        }
    }

    /// React to the events by applying the instructions
    pub fn process_events(&mut self, context: &application::Context){
        for event in self.receiver.try_iter() {
            if let types::Event::Patch(event) = event {
                match event {
                    types::PatchEvent::SetParam{param_name, param_val} => {
                        if let Some(node) = context.graph.node_mut(context.selected_node) {
                            node.set_param(param_name, param_val);
                        }
                    },
                    _         => (),
                }
            }
        }
    }
}
