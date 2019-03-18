use defs;
use event::Event;
use event::midi;
use jack::RawMidi;
use std::slice::Iter;

/// Enumeration of MIDI event types
pub enum MidiEvent {
    // Channel Voice Messages
    NoteOff { note: u8 },
    NoteOn { note: u8, velocity: u8 },
    PitchBend { value: u16 },
    PolyphonicAftertouch { note: u8, pressure: u8 },
    ControlChange { controller: u8, value: u8 },
    ProgramChange { program: u8 },
    ChannelPressure { pressure: u8 },
    // Channel Mode Messages
    AllSoundOff,
    ResetAllControllers,
    LocalControlOff,
    LocalControlOn,
    AllNotesOff,
    OmniModeOff,
    OmniModeOn,
    MonoModeOn,
    PolyModeOn,
    // System Common Messages
    SystemExclusive { data1: u8, data2: u8 },
    TimeCodeQuarterFrame { message_type: u8, values: u8 },
    SongPositionPointer { beats: u16 },
    SongSelect { value: u8 },
    TuneRequest,
    EndOfExclusive,
    // System Real-Time Messages
    TimingClock,
    Start,
    Continue,
    Stop,
    ActiveSensing,
    Reset,
}

impl MidiEvent {
    /// Process a portmidi::MidiEvent into our format of midi event.
    /// If the event is recognised, return Some(MidiEventProcessed).
    /// Otherwise, return None.
    pub fn parse(raw_event: RawMidi, filter_by_channel: Option<u8>) -> Option<Event> {
        let status = raw_event.bytes[0];
        let status_category = status & 0xF0;
        let status_extra = status & 0x0F;

        let data1 = raw_event.bytes[1];
        let data2 = raw_event.bytes[2];

        // Suppress MIDI messages according to the filter_by_channel parameter.
        if let Some(channel_requested) = filter_by_channel {
            // Anything except system messages can be filtered by channel
            if status_category != 0xF0 && status_extra != channel_requested {
                return None;
            }
        }

        match status_category {
            0x80 => Some(Event::Midi(MidiEvent::NoteOff {
                note: data1,
            })),
            0x90 => {
                // Often MIDI devices send a Note On with velocity == 0 to indicate
                // a Note Off event. Handle that here.
                let velocity = data2;
                match velocity {
                    0 => Some(Event::Midi(MidiEvent::NoteOff {
                        note: data1,
                    })),
                    _ => Some(Event::Midi(MidiEvent::NoteOn {
                        note: data1,
                        velocity: data2,
                    })),
                }
            },
            0xA0 => Some(Event::Midi(MidiEvent::PolyphonicAftertouch {
                note: data1,
                pressure: data2,
            })),
            0xB0 => {
                match data1 {
                    0..=119 => Some(Event::Midi(MidiEvent::ControlChange {
                        controller: data1,
                        value: data2,
                    })),
                    120 => Some(Event::Midi(MidiEvent::AllSoundOff)),
                    121 => Some(Event::Midi(MidiEvent::ResetAllControllers)),
                    122 => {
                        match data2 {
                            0 => Some(Event::Midi(MidiEvent::LocalControlOff)),
                            127 => Some(Event::Midi(MidiEvent::LocalControlOn)),
                            _ => None, // Undefined
                        }
                    }
                    123 => Some(Event::Midi(MidiEvent::AllNotesOff)),
                    124 => Some(Event::Midi(MidiEvent::OmniModeOff)),
                    125 => Some(Event::Midi(MidiEvent::OmniModeOn)),
                    126 => Some(Event::Midi(MidiEvent::MonoModeOn)),
                    127 => Some(Event::Midi(MidiEvent::PolyModeOn)),
                    _ => None, // Undefined
                }
            },
            0xC0 => Some(Event::Midi(MidiEvent::ProgramChange {
                program: data1,
            })),
            0xD0 => Some(Event::Midi(MidiEvent::ChannelPressure {
                pressure: data1,
            })),
            0xE0 => Some(Event::Midi(MidiEvent::PitchBend {
                value: ((data2 as u16) << 7) + (data1 as u16),
            })),
            0xF0 => {
                // System message. Consider the second four bits
                match status & 0x0F {
                    0x00 => Some(Event::Midi(MidiEvent::SystemExclusive {
                        data1: data1,
                        data2: data2,
                    })),
                    0x01 => Some(Event::Midi(MidiEvent::TimeCodeQuarterFrame {
                        message_type: data1 >> 4,
                        values: data1 & 0x0F,
                    })),
                    0x02 => Some(Event::Midi(MidiEvent::SongPositionPointer {
                        beats: ((data2 as u16) << 7) + (data1 as u16),
                    })),
                    0x03 => Some(Event::Midi(MidiEvent::SongSelect {
                        value: data1,
                    })),
                    0x06 => Some(Event::Midi(MidiEvent::TuneRequest)),
                    0x07 => Some(Event::Midi(MidiEvent::EndOfExclusive)),
                    0x08 => Some(Event::Midi(MidiEvent::TimingClock)),
                    0x10 => Some(Event::Midi(MidiEvent::Start)),
                    0x11 => Some(Event::Midi(MidiEvent::Continue)),
                    0x12 => Some(Event::Midi(MidiEvent::Stop)),
                    0x14 => Some(Event::Midi(MidiEvent::ActiveSensing)),
                    0x15 => Some(Event::Midi(MidiEvent::Reset)),
                    _ => None,
                }
            },
            _ => None
        }
    }
}
/// Buffer will contain midi events received in the last block.
pub struct InputBuffer {
    events: Vec<Event>,
}

impl InputBuffer {
    /// Create a new buffer for receiving MIDI from one input device.
    pub fn new() -> InputBuffer {
        // Code based on "monitor-all" example of portmidi crate
        InputBuffer {
            events: Vec::<Event>::with_capacity(defs::MIDI_BUF_LEN),
        }
    }

    /// Fill the buffer with MIDI events since the last buffer update.
    pub fn update(&mut self, raw_midi_iter: jack::MidiIter) {
        // First, clear any old MIDI events.
        self.events.clear();

        for raw_midi_event in raw_midi_iter {
            if let Some(event) = midi::MidiEvent::parse(raw_midi_event, None) {
                self.events.push(event);
            }
        }
    }

    /// Get an iterator over the MIDI events in the buffer.
    pub fn iter(&self) -> Iter<Event> {
        self.events.iter()
    }
}
