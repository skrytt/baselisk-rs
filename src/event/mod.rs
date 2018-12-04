
pub mod midi;
pub mod types;

pub use event::types::{
    Event,
    MidiEvent,
};

use std::slice::Iter;

pub struct Buffer {
    pub midi: midi::InputBuffer,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            midi: midi::InputBuffer::new(),
        }
    }

    /// Update: should be called prior to audio processing each block.
    /// Will update the internal event vector, so that successive
    /// calls to .iter() will provide any new events.
    pub fn update(&mut self) {
        self.midi.update();
    }

    pub fn iter(&self) -> Iter<types::Event> {
        self.midi.iter()
    }
}

