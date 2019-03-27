
use event::MidiEvent;
use std::slice;

// Number of note changes we can buffer per callback.
// If the note changes more times than this, the behaviour is to drop note change events.
const NOTE_CHANGE_VEC_SIZE: usize = 64;

/// A note selector with high-note-priority selection.
pub struct MonoNoteSelector {
    notes_held: Vec<bool>,
    note_priority_stack: Vec<u8>,
    note_selected: Option<u8>,
    note_changes_vec: Vec<(usize, Option<u8>)>,
}

impl MonoNoteSelector {
    pub fn new() -> MonoNoteSelector {
        MonoNoteSelector {
            notes_held: vec![false; 128],
            note_priority_stack: Vec::with_capacity(128),
            note_selected: None,
            note_changes_vec: Vec::with_capacity(NOTE_CHANGE_VEC_SIZE),
        }
    }

    pub fn midi_panic(&mut self) {
        // Small optimization: if no notes are on, there's nothing to do.
        if let Some(_) = self.note_selected {
            for mut note in self.notes_held.iter_mut() {
                *note = false;
            }
            self.note_priority_stack.clear();
            self.note_selected = None;
            self.note_changes_vec.clear();
        }
    }

    /// Return an iterator of note changes from this callback
    /// based on the provided iterator of midi events.
    pub fn update_note_changes_vec(&mut self,
                                   midi_iter: slice::Iter<(usize, MidiEvent)>)
    {
        self.note_changes_vec.clear();

        for (frame_num, midi_event) in midi_iter {
            let note_change = match midi_event {
                MidiEvent::NoteOn { note, .. } => {
                    self.note_on(*note)
                }
                MidiEvent::NoteOff { note } => {
                    self.note_off(*note)
                }
                _ => None,
            };

            // note_change is an Option<Option<u8>> indicating whether the note changed as a
            // result of the MIDI event.
            if let Some(note_selected) = note_change {
                // note_selected is an Option<u8> indicating the Some(note) if there is a note,
                // or otherwise, None.
                self.note_changes_vec.push((*frame_num, note_selected));
                if self.note_changes_vec.len() == self.note_changes_vec.capacity() {
                    // Buffer full - drop further MIDI events.
                    break
                };
            }
        }
    }

    pub fn iter_note_changes(&self) -> slice::Iter<(usize, Option<u8>)> {
        self.note_changes_vec.iter()
    }

    /// Return Some(Option<u8>) if the note changed as a result of this event.
    /// Otherwise, return None.
    fn note_on(&mut self, note: u8) -> Option<Option<u8>> {
        if let Some(note_held_ref) = self.notes_held.get_mut(note as usize) {
            // It's possible (due to dropped note events)
            // that the note was not actually off. Check for that here.
            if *note_held_ref == false {
                *note_held_ref = true;
                // Due to the note_held_ref check, it should never be the case
                // that the unwrap_err call here fails
                let insert_index = self.note_priority_stack.binary_search(&note).unwrap_err();
                self.note_priority_stack.insert(insert_index, note);
                // Update the selected note.
                // After inserting to the note priority stack, it should never
                // be the case that the unwrap call here fails
                self.note_selected = Some(*self.note_priority_stack.last().unwrap());

                // Indicate that the note held changed.
                return Some(self.note_selected)
            }
        }
        None
    }

    /// Return Some(Option<u8>) if the note changed as a result of this event.
    /// Otherwise, return None.
    fn note_off(&mut self, note: u8) -> Option<Option<u8>> {
        if let Some(note_held_ref) = self.notes_held.get_mut(note as usize) {
            // It's possible (due to dropped note events or midi panics)
            // that the note was not actually on. Check for that here.
            if *note_held_ref == true {
                *note_held_ref = false;
                 // Due to the note_held_ref check, it should never be the case
                // that the unwrap_err call here fails
                let remove_index = self.note_priority_stack.binary_search(&note).unwrap();
                self.note_priority_stack.remove(remove_index);
                // Update the selected note
                self.note_selected = match self.note_priority_stack.last() {
                    Some(note) => Some(*note),
                    None => None,
                };
                // Indicate that the note held changed.
                return Some(self.note_selected)
            }
        }
        None
    }
}
