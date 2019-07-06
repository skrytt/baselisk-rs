
use shared::event::{
    EngineEvent,
    MidiEvent
};

/// A note selector with high-note-priority selection.
#[derive(Default)]
pub struct MonoNoteSelector {
    notes_held: Vec<bool>,
    note_priority_stack: Vec<u8>,
    note_selected: Option<u8>,
}

/// Struct to indicate whether a MIDI event resulted in a 
pub enum MidiEventResult {
    NoteChange(Option<u8>),
    Ignore,
}

impl MonoNoteSelector {
    pub fn new() -> Self {
        Self {
            notes_held: vec![false; 128],
            note_priority_stack: Vec::with_capacity(128),
            note_selected: None,
        }
    }

    pub fn midi_panic(&mut self) {
        // Small optimization: if no notes are on, there's nothing to do.
        if self.note_selected.is_some() {
            for note in &mut self.notes_held {
                *note = false;
            }
            self.note_priority_stack.clear();
            self.note_selected = None;
        }
    }

    /// Return an Option<EngineEvent> representing a possible engine event
    /// based on the provided MIDI event.
    pub fn process_event(&mut self, midi_event: &MidiEvent) -> Option<EngineEvent> {
        // result is an Option<Option<u8>> indicating whether the note changed as a
        // result of the MIDI event.
        let result = match midi_event {
            MidiEvent::NoteOn { note, .. } => {
                self.note_on(*note)
            }
            MidiEvent::NoteOff { note } => {
                self.note_off(*note)
            }
            _ => MidiEventResult::Ignore,
        };
        match result {
            MidiEventResult::NoteChange(note_change) => Some(EngineEvent::NoteChange{ note: note_change }),
            MidiEventResult::Ignore => None,
        }
    }

    /// Return Some(Option<u8>) if the note changed as a result of this event.
    /// Otherwise, return None.
    fn note_on(&mut self, note: u8) -> MidiEventResult {
        if let Some(note_held_ref) = self.notes_held.get_mut(note as usize) {
            // It's possible (due to dropped note events)
            // that the note was not actually off. Check for that here.
            if !(*note_held_ref) {
                *note_held_ref = true;
                // Due to the note_held_ref check, it should never be the case
                // that the unwrap_err call here fails
                let insert_index = self.note_priority_stack.binary_search(&note).unwrap_err();
                self.note_priority_stack.insert(insert_index, note);
                // Update the selected note.
                // After inserting to the note priority stack, it should never
                // be the case that the unwrap call here fails
                let new_note_selected = Some(*self.note_priority_stack.last().unwrap());
                if new_note_selected != self.note_selected {
                    self.note_selected = new_note_selected;
                    // Indicate that the note held changed.
                    return MidiEventResult::NoteChange(self.note_selected)
                }
            }
        }
        MidiEventResult::Ignore
    }

    /// Return Some(Option<u8>) if the note changed as a result of this event.
    /// Otherwise, return None.
    fn note_off(&mut self, note: u8) -> MidiEventResult {
        if let Some(note_held_ref) = self.notes_held.get_mut(note as usize) {
            // It's possible (due to dropped note events or midi panics)
            // that the note was not actually on. Check for that here.
            if *note_held_ref {
                *note_held_ref = false;
                 // Due to the note_held_ref check, it should never be the case
                // that the unwrap_err call here fails
                let remove_index = self.note_priority_stack.binary_search(&note).unwrap();
                self.note_priority_stack.remove(remove_index);

                // Update the selected note
                let new_note_selected = match self.note_priority_stack.last() {
                    Some(note) => Some(*note),
                    None => None,
                };

                if new_note_selected != self.note_selected {
                    self.note_selected = new_note_selected;
                    // Indicate that the note held changed.
                    return MidiEventResult::NoteChange(self.note_selected)
                }
            }
        }
        MidiEventResult::Ignore
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_note_change(event: EngineEvent,
                         expected_note: Option<u8>) -> bool
    {
        if let EngineEvent::NoteChange { note } = event {
            if note == expected_note {
                return true
            }
        }
        false
    }

    #[test]
    fn test_high_note_priority_switch_to_higher_note() {
        let mut note_selector = MonoNoteSelector::new();

        let output = note_selector.process_event(
            &MidiEvent::NoteOn{note: 20, velocity: 127});
        assert!(output.is_some());
        let output = output.unwrap();
        assert!(check_note_change(output, Some(20)));

        // Play a higher note, and assert the output note changes to that note.
        let output = note_selector.process_event(
            &MidiEvent::NoteOn{note: 30, velocity: 127});
        assert!(output.is_some());
        let output = output.unwrap();
        assert!(check_note_change(output, Some(30)));
    }

    #[test]
    fn test_high_note_priority_suppress_lower_note() {
        let mut note_selector = MonoNoteSelector::new();

        let output = note_selector.process_event(
            &MidiEvent::NoteOn{note: 20, velocity: 127});
        assert!(output.is_some());
        let output = output.unwrap();
        assert!(check_note_change(output, Some(20)));

        // Play a lower note, and assert the output note doesn't change.
        let output = note_selector.process_event(
            &MidiEvent::NoteOn{note: 10, velocity: 127});
        assert!(output.is_none());

        // Release the lower note, and assert the output note doesn't change.
        let output = note_selector.process_event(
            &MidiEvent::NoteOff{note: 10});
        assert!(output.is_none());

        // Release the original note, and assert the output stops.
        let output = note_selector.process_event(
            &MidiEvent::NoteOff{note: 20});
        assert!(output.is_some());
        let output = output.unwrap();
        assert!(check_note_change(output, None));
    }

    #[test]
    fn test_high_note_priority_switch_to_lower_note() {
        let mut note_selector = MonoNoteSelector::new();

        let output = note_selector.process_event(
            &MidiEvent::NoteOn{note: 20, velocity: 127});
        assert!(output.is_some());
        let output = output.unwrap();
        assert!(check_note_change(output, Some(20)));

        // Play a lower note, and assert the output note doesn't change.
        let output = note_selector.process_event(
            &MidiEvent::NoteOn{note: 10, velocity: 127});
        assert!(output.is_none());

        // Release the higher note, and assert the output note switches to the lower note.
        let output = note_selector.process_event(
            &MidiEvent::NoteOff{note: 20});
        assert!(output.is_some());
        let output = output.unwrap();
        assert!(check_note_change(output, Some(10)));

        // Release the original note, and assert the output stops.
        let output = note_selector.process_event(
            &MidiEvent::NoteOff{note: 10});
        assert!(output.is_some());
        let output = output.unwrap();
        assert!(check_note_change(output, None));
    }
}
