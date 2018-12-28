
extern crate portmidi;

use std::slice::Iter;

use defs;
use event::types;

/// Buffer will contain midi events received in the last block.
pub struct InputBuffer {
    events: Vec<types::Event>,
    port: Option<portmidi::InputPort>,
    context: portmidi::PortMidi,
}

impl InputBuffer {
    /// Create a new buffer for receiving MIDI from one input device.
    pub fn new(portmidi: portmidi::PortMidi) -> InputBuffer {
        // Code based on "monitor-all" example of portmidi crate
        InputBuffer {
            events: Vec::<types::Event>::with_capacity(0),
            port: None,
            context: portmidi,
        }
    }

    pub fn set_port(&mut self, device_id: i32) -> Result<(), String> {
        let info = match self.context.device(device_id) {
            Err(e) => return Err(e.to_string()),
            Ok(t) => t,
        };
        println!("Listening on MIDI input: {}) {}", info.id(), info.name());

        match self.context.input_port(info, defs::MIDI_BUF_LEN) {
            Err(e) => Err(e.to_string()),
            Ok(port) => {
                self.port = Some(port);
                Ok(())
            }
        }
    }

    /// Fill the buffer with MIDI events since the last buffer update.
    pub fn update(&mut self) {
        // First, clear any old MIDI events.
        self.events.clear();

        if let Some(ref port) = self.port {
            // If we have MIDI events to process, get those MIDI events.
            // PortMidi doesn't have a blocking receive method, so use
            // poll to check once if there are events.
            if let Ok(_) = port.poll() {
                // Yes, there are events, let's try to retrieve them.
                // Then, convert them to our MidiEvent types, filtering out
                // events that we don't know how to use.
                if let Ok(Some(events)) = port.read_n(defs::MIDI_BUF_LEN) {
                    self.events = events
                        .into_iter()
                        .filter_map(|raw_midi_event| {
                            types::MidiEvent::parse(raw_midi_event)
                        })
                        .collect()
                }
            }
        }
    }

    /// Get an iterator over the MIDI events in the buffer.
    pub fn iter(&self) -> Iter<types::Event> {
        self.events.iter()
    }
}

pub fn note_to_frequency(note: u8) -> f64 {
    440.0 * ((note as f64 - 69.0) / 12.0).exp2()
}
