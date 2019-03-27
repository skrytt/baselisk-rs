use jack;

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
    /// If the event is recognised, return Some((usize, Event)).
    /// Otherwise, return None.
    pub fn parse(raw_event: jack::RawMidi,
                 filter_by_channel: Option<u8>) -> Option<(usize, MidiEvent)> {
        let time = raw_event.time as usize;

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
            0x80 => Some((time, MidiEvent::NoteOff {
                note: data1,
            })),
            0x90 => {
                // Often MIDI devices send a Note On with velocity == 0 to indicate
                // a Note Off event. Handle that here.
                let velocity = data2;
                match velocity {
                    0 => Some((time, MidiEvent::NoteOff {
                        note: data1,
                    })),
                    _ => Some((time, MidiEvent::NoteOn {
                        note: data1,
                        velocity: data2,
                    })),
                }
            },
            0xA0 => Some((time, MidiEvent::PolyphonicAftertouch {
                note: data1,
                pressure: data2,
            })),
            0xB0 => {
                match data1 {
                    0..=119 => Some((time, MidiEvent::ControlChange {
                        controller: data1,
                        value: data2,
                    })),
                    120 => Some((time, MidiEvent::AllSoundOff)),
                    121 => Some((time, MidiEvent::ResetAllControllers)),
                    122 => {
                        match data2 {
                            0 => Some((time, MidiEvent::LocalControlOff)),
                            127 => Some((time, MidiEvent::LocalControlOn)),
                            _ => None, // Undefined
                        }
                    }
                    123 => Some((time, MidiEvent::AllNotesOff)),
                    124 => Some((time, MidiEvent::OmniModeOff)),
                    125 => Some((time, MidiEvent::OmniModeOn)),
                    126 => Some((time, MidiEvent::MonoModeOn)),
                    127 => Some((time, MidiEvent::PolyModeOn)),
                    _ => None, // Undefined
                }
            },
            0xC0 => Some((time, MidiEvent::ProgramChange {
                program: data1,
            })),
            0xD0 => Some((time, MidiEvent::ChannelPressure {
                pressure: data1,
            })),
            0xE0 => Some((time, MidiEvent::PitchBend {
                value: ((data2 as u16) << 7) + (data1 as u16),
            })),
            0xF0 => {
                // System message. Consider the second four bits
                match status & 0x0F {
                    0x00 => Some((time, MidiEvent::SystemExclusive {
                        data1: data1,
                        data2: data2,
                    })),
                    0x01 => Some((time, MidiEvent::TimeCodeQuarterFrame {
                        message_type: data1 >> 4,
                        values: data1 & 0x0F,
                    })),
                    0x02 => Some((time, MidiEvent::SongPositionPointer {
                        beats: ((data2 as u16) << 7) + (data1 as u16),
                    })),
                    0x03 => Some((time, MidiEvent::SongSelect {
                        value: data1,
                    })),
                    0x06 => Some((time, MidiEvent::TuneRequest)),
                    0x07 => Some((time, MidiEvent::EndOfExclusive)),
                    0x08 => Some((time, MidiEvent::TimingClock)),
                    0x10 => Some((time, MidiEvent::Start)),
                    0x11 => Some((time, MidiEvent::Continue)),
                    0x12 => Some((time, MidiEvent::Stop)),
                    0x14 => Some((time, MidiEvent::ActiveSensing)),
                    0x15 => Some((time, MidiEvent::Reset)),
                    _ => None,
                }
            },
            _ => None
        }
    }
}
