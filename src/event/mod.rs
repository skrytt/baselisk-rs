extern crate portmidi;

pub mod midi;
pub mod types;

pub use event::types::{Event, MidiEvent, PatchEvent};

use std::slice;

pub struct Buffer {
    pub midi: midi::InputBuffer,
}

impl Buffer {
    pub fn new(portmidi: portmidi::PortMidi) -> Buffer {
        Buffer {
            midi: midi::InputBuffer::new(portmidi),
        }
    }

    /// Update: should be called prior to audio processing each block.
    /// Will update the internal event vector, so that successive
    /// calls to .iter() will provide any new events.
    pub fn update_midi(&mut self) {
        self.midi.update();
    }

    /// Get an iterator over MIDI events from PortMidi.
    /// This is intended to be
    pub fn iter_midi(&self) -> slice::Iter<types::Event> {
        self.midi.iter()
    }
}
