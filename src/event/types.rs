
extern crate portmidi;

/// Generic event type enum that can be used for notifications
pub enum Event {
    Midi(MidiEvent),
    Patch(PatchEvent),
}

pub enum PatchEvent {
    TestEvent,
    _Other,
}

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
