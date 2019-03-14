extern crate portmidi;

use std::slice::Iter;

use defs;
use event::Event;
use event::midi;

/// Enumeration of MIDI event types
pub enum MidiEvent {
    NoteOff { note: u8 },
    NoteOn { note: u8, velocity: u8 },
    PolyphonicAftertouch { note: u8, pressure: u8 },
    ControlChange { controller: u8, value: u8 },
    ProgramChange { program: u8 },
    ChannelPressure { pressure: u8 },
    PitchBend { value: u16 },
    SystemExclusive { data1: u8, data2: u8 },
    TimeCodeQuarterFrame { message_type: u8, values: u8 },
    SongPositionPointer { beats: u16 },
    SongSelect { value: u8 },
    TuneRequest {},
    EndOfExclusive {},
    TimingClock {},
    Start {},
    Continue {},
    Stop {},
    ActiveSensing {},
    Reset {},
}

impl MidiEvent {
    /// Process a portmidi::MidiEvent into our format of midi event.
    /// If the event is recognised, return Some(MidiEventProcessed).
    /// Otherwise, return None.
    pub fn parse(event: portmidi::MidiEvent) -> Option<Event> {
        let message = event.message;
        let status = message.status;

        match status & 0xF0 {
            0x80 => Some(Event::Midi(MidiEvent::NoteOff {
                note: message.data1,
            })),
            0x90 => {
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
            },
            0xA0 => Some(Event::Midi(MidiEvent::PolyphonicAftertouch {
                note: message.data1,
                pressure: message.data2,
            })),
            0xB0 => Some(Event::Midi(MidiEvent::ControlChange {
                controller: message.data1,
                value: message.data2,
            })),
            0xC0 => Some(Event::Midi(MidiEvent::ProgramChange {
                program: message.data1,
            })),
            0xD0 => Some(Event::Midi(MidiEvent::ChannelPressure {
                pressure: message.data1,
            })),
            0xE0 => Some(Event::Midi(MidiEvent::PitchBend {
                value: ((message.data2 as u16) << 7) + (message.data1 as u16),
            })),
            0xF0 => {
                // System message. Consider the second four bits
                match status & 0x0F {
                    0x00 => Some(Event::Midi(MidiEvent::SystemExclusive {
                        data1: message.data1,
                        data2: message.data2,
                    })),
                    0x01 => Some(Event::Midi(MidiEvent::TimeCodeQuarterFrame {
                        message_type: message.data1 >> 4,
                        values: message.data1 & 0x0F,
                    })),
                    0x02 => Some(Event::Midi(MidiEvent::SongPositionPointer {
                        beats: ((message.data2 as u16) << 7) + (message.data1 as u16),
                    })),
                    0x03 => Some(Event::Midi(MidiEvent::SongSelect {
                        value: message.data1,
                    })),
                    0x06 => Some(Event::Midi(MidiEvent::TuneRequest {})),
                    0x07 => Some(Event::Midi(MidiEvent::EndOfExclusive {})),
                    0x08 => Some(Event::Midi(MidiEvent::TimingClock {})),
                    0x10 => Some(Event::Midi(MidiEvent::Start {})),
                    0x11 => Some(Event::Midi(MidiEvent::Continue {})),
                    0x12 => Some(Event::Midi(MidiEvent::Stop {})),
                    0x14 => Some(Event::Midi(MidiEvent::ActiveSensing {})),
                    0x15 => Some(Event::Midi(MidiEvent::Reset {})),
                    _ => None,
                }
            }
            _ => {
                None
            }
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
