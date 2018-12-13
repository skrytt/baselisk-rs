
pub mod midi;
pub mod patch;
pub mod types;

pub use event::types::{
    Event,
    MidiEvent,
    PatchEvent,
};

use application;
use std::slice;
use std::sync::mpsc;

pub struct Buffer {
    pub midi: midi::InputBuffer,
    patch: patch::InputBuffer,
}

impl Buffer {
    pub fn new(receiver: mpsc::Receiver<Event>) -> Buffer {
        Buffer {
            midi: midi::InputBuffer::new(),
            patch: patch::InputBuffer::new(receiver),
        }
    }

    /// Get events sent over the thread channel.
    /// This is intended to be called once per audio buffer,
    /// and consumes the events it yields.
    pub fn update_patch(&self, context: &mut application::Context) {
        self.patch.process_events(context);
    }

    /// Update: should be called prior to audio processing each block.
    /// Will update the internal event vector, so that successive
    /// calls to .iter() will provide any new events.
    pub fn update_midi_buffer(&mut self) {
        self.midi.update();
    }

    /// Get an iterator over MIDI events from PortMidi.
    /// This is intended to be
    pub fn iter_midi(&self) -> slice::Iter<types::Event> {
        self.midi.iter()
    }
}

