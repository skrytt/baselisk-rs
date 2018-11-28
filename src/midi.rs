extern crate portmidi;

use std::slice::Iter;

use defs;

/// Enum describes types of midi event. Not all types are implemented.
pub enum MidiEvent {
    NoteOff { note: u8 },
    NoteOn { note: u8, velocity: u8 },
}

impl MidiEvent {
    /// Process a portmidi::MidiEvent into our format of midi event.
    /// If the event is recognised, return Some(MidiEventProcessed).
    /// Otherwise, return None.
    fn process(event: portmidi::MidiEvent) -> Option<MidiEvent> {
        let message = event.message;
        let status = message.status;

        if status == 0x80 {
            Some(MidiEvent::NoteOff {
                note: message.data1,
            })
        } else if status == 0x90 {
            // Often MIDI devices send a Note On with velocity == 0 to indicate
            // a Note Off event. Handle that here.
            let velocity = message.data2;
            match velocity {
                0 => Some(MidiEvent::NoteOff {
                    note: message.data1,
                }),
                _ => Some(MidiEvent::NoteOn {
                    note: message.data1,
                    velocity: message.data2,
                }),
            }
        } else {
            println!(
                "t={}: Dropping unknown MIDI event: {}",
                event.timestamp, message
            );
            None
        }
    }
}

/// Buffer will contain midi events received in the last block.
pub struct InputBuffer {
    events: Vec<MidiEvent>,
    port: Option<portmidi::InputPort>,
    context: portmidi::PortMidi,
}

impl InputBuffer {
    /// Create a new buffer for receiving MIDI from one input device.
    pub fn new() -> InputBuffer {
        println!("Setting up PortMidi input buffer...");

        // Code based on "monitor-all" example of portmidi crate
        let context = portmidi::PortMidi::new().unwrap();

        InputBuffer {
            events: Vec::<MidiEvent>::with_capacity(0),
            port: None,
            context: context,
        }
    }

    pub fn print_devices(&self) {
        for dev in self.context.devices().unwrap() {
            println!("{}", dev);
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
                        .filter_map(|event| MidiEvent::process(event))
                        .collect()
                }
            }
        }
    }

    /// Get an iterator over the MIDI events in the buffer.
    pub fn iter(&self) -> Iter<MidiEvent> {
        self.events.iter()
    }
}

pub fn note_to_frequency(note: u8) -> defs::Frequency {
    440.0 as defs::Frequency * ((note as defs::Frequency - 69.0) / 12.0).exp2()
}
