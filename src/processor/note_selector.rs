
use event::{Event, MidiEvent};
use std::slice;

/// A note selector with high-note-priority selection.
pub struct MonoNoteSelector {
    notes_held: Vec<bool>,
    note_priority_stack: Vec<u8>,
    note_selected: Option<u8>,
}

impl MonoNoteSelector {
    pub fn new() -> MonoNoteSelector {
        MonoNoteSelector {
            notes_held: vec![false; 128],
            note_priority_stack: Vec::with_capacity(128),
            note_selected: None,
        }
    }

    pub fn get_note(&self) -> Option<u8> {
        self.note_selected
    }

    pub fn process_midi_events(&mut self, midi_iter: slice::Iter<Event>) {
        for event in midi_iter {
            if let Event::Midi(midi_event) = event {
                match midi_event {
                    MidiEvent::NoteOn { note, .. } => self.note_on(*note),
                    MidiEvent::NoteOff { note } => self.note_off(*note),
                    MidiEvent::AllNotesOff | MidiEvent::AllSoundOff => self.all_notes_off(),
                    _ => (),
                }
            }
        }
    }

    fn note_on(&mut self, note: u8) {
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
            }
        }
    }

    fn note_off(&mut self, note: u8) {
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
                }
            }
        }
    }

    fn all_notes_off(&mut self) {
        // Small optimization: if no notes are on, there's nothing to do.
        if let Some(_) = self.note_selected {
            for mut note in self.notes_held.iter_mut() {
                *note = false;
            }
            self.note_priority_stack.clear();
            self.note_selected = None;
        }
    }
}
