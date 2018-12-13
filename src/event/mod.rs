
pub mod midi;
pub mod types;

pub use event::types::{
    Event,
    MidiEvent,
};

use std::slice::Iter;
use std::sync::mpsc;

pub struct Buffer {
    pub midi: midi::InputBuffer,
    _receiver: mpsc::Receiver<Event>,
}

impl Buffer {
    pub fn new(_receiver: mpsc::Receiver<Event>) -> Buffer {
        Buffer {
            midi: midi::InputBuffer::new(),
            _receiver
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

