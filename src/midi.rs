
extern crate portmidi;

use std::slice::Iter;

use defs;

/// Buffer will contain midi events received in the last block.
pub struct InputBuffer {
    events: Vec<portmidi::MidiEvent>,
    port: portmidi::InputPort,
    _context: portmidi::PortMidi,
}

impl InputBuffer {
    /// Create a new buffer for receiving MIDI from one input device.
    pub fn new(device_id: i32) -> InputBuffer {

        println!("Setting up PortMidi input buffer...");

        // Code based on "monitor-all" example of portmidi crate
        let context = portmidi::PortMidi::new().unwrap();

        let info = context.device(device_id).unwrap();
        println!("Listening on MIDI input: {}) {}", info.id(), info.name());

        let port = context.input_port(info, defs::MIDI_BUF_LEN).unwrap();

        InputBuffer {
            events: Vec::<portmidi::MidiEvent>::with_capacity(0),
            port,
            _context: context, // "Never used", but must remain in scope,
                               // otherwise PortMidi is dropped and terminated
        }
    }

    /// Fill the buffer with MIDI events since the last buffer update.
    pub fn update(&mut self) {

        // First, clear any old MIDI events.
        self.events.clear();

        // If we have MIDI events to process, get those MIDI events.
        // PortMidi doesn't have a blocking receive method, so use
        // poll to check once if there are events.
        if let Ok(_) = self.port.poll() {
            // Yes, there are events, let's try to retrieve them
            if let Ok(Some(events)) = self.port.read_n(defs::MIDI_BUF_LEN) {

                for event in events.iter() {
                    println!("t={}: Got MIDI event: {}", event.timestamp, event.message);
                }
                self.events = events;
            }
        }
    }

    /// Get an iterator over the MIDI events in the buffer.
    pub fn iter(&self) -> Iter<portmidi::MidiEvent> {
        self.events.iter()
    }
}

