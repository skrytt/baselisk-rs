pub mod midi;
pub mod patch;

pub use event::midi::MidiEvent;
pub use event::patch::PatchEvent;

use std::slice;

/// Generic event type enum that can be used for notifications
pub enum Event {
    Midi(MidiEvent),
    Patch(PatchEvent),
}

/// Aggregate buffer for different types of events.
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
    pub fn update_midi(&mut self, raw_midi_iter: jack::MidiIter) {
        self.midi.update(raw_midi_iter);
    }

    /// Get an iterator over MIDI events that were collected in the
    /// previous call to update_midi. This is intended to be called
    /// once per processor that uses MIDI events.
    pub fn iter_midi(&self) -> slice::Iter<(usize, Event)> {
        self.midi.iter()
    }
}
