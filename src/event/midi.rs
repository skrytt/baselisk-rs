extern crate portmidi;

use std::slice::Iter;

use defs;
use event::Event;
use event::midi;

/// Midi events are one type of event
pub enum MidiEvent {
    NoteOff { note: u8 },
    NoteOn { note: u8, velocity: u8 },
}

impl MidiEvent {
    /// Process a portmidi::MidiEvent into our format of midi event.
    /// If the event is recognised, return Some(MidiEventProcessed).
    /// Otherwise, return None.
    pub fn parse(event: portmidi::MidiEvent) -> Option<Event> {
        let message = event.message;
        let status = message.status;

        if status == 0x80 {
            Some(Event::Midi(MidiEvent::NoteOff {
                note: message.data1,
            }))
        } else if status == 0x90 {
            // Often MIDI devices send a Note On with velocity == 0 to indicate
            // a Note Off event. Handle that here.
            let velocity = message.data2;
            match velocity {
                0 => Some(Event::Midi(MidiEvent::NoteOff {
                    note: message.data1,
                })),
                _ => Some(Event::Midi(MidiEvent::NoteOn {
                    note: message.data1,
                    velocity: message.data2,
                })),
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
    events: Vec<Event>,
    port: Option<portmidi::InputPort>,
    context: portmidi::PortMidi,
}

impl InputBuffer {
    /// Create a new buffer for receiving MIDI from one input device.
    pub fn new(portmidi: portmidi::PortMidi) -> InputBuffer {
        // Code based on "monitor-all" example of portmidi crate
        InputBuffer {
            events: Vec::<Event>::with_capacity(0),
            port: None,
            context: portmidi,
        }
    }

    /// Set the MIDI port we will receive input from.
    pub fn set_port(&mut self, device_id: i32) -> Result<(), &'static str> {
        let info = match self.context.device(device_id) {
            Err(_) => return Err("PortMidi returned an error for this device ID"),
            Ok(info) => info,
        };
        println!("Listening on MIDI input: {}) {}", info.id(), info.name());

        match self.context.input_port(info, defs::MIDI_BUF_LEN) {
            Err(_) => Err("PortMidi returned an error trying to use this device ID as input"),
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
                        .filter_map(|raw_midi_event| midi::MidiEvent::parse(raw_midi_event))
                        .collect()
                }
            }
        }
    }

    /// Get an iterator over the MIDI events in the buffer.
    pub fn iter(&self) -> Iter<Event> {
        self.events.iter()
    }
}
