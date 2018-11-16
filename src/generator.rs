
extern crate dsp;
extern crate portmidi;

use std::f64::consts::PI;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use dsp::Sample;
use defs;
use midi;

/// An oscillator must implement the Oscillator trait
pub trait Generator<S> {
    fn update_state(&mut self);

    fn generate(&mut self) -> S
    where S: dsp::Sample + dsp::FromSample<f32> + fmt::Display;
}

/// SineOscillator is a type that will implement the trait above:
pub struct OscillatorParams {
    pub phase: defs::Phase,
    pub frequency: defs::Frequency,
    pub volume: defs::Volume,
    pub note: Option<u8>,
}

impl OscillatorParams {
    pub fn new(volume: defs::Volume) -> OscillatorParams {
        OscillatorParams{
            phase: 0.0,
            frequency: 0.0,
            volume,
            note: None
        }
    }

    fn update_state(&mut self, midi_input_buffer: Rc<RefCell<midi::InputBuffer>>) {
        // Iterate over any midi events and mutate the oscillator params accordingly
        let midi_events = midi_input_buffer.borrow();
        for midi_event in midi_events.iter() {
            match midi_event {
                midi::MidiEvent::NoteOff{note} => {
                    // If this note was already playing, deactivate it
                    if let Some(active_note) = self.note {
                        if *note == active_note {
                            self.frequency = 0.0;
                            self.phase = 0.0;
                            self.note = None;
                            self.volume = 0.0;
                        }
                    }
                },
                midi::MidiEvent::NoteOn{note, ..} => {
                    // Set the active note and frequency to match this new note
                    self.frequency = midi::note_to_frequency(*note);
                    self.note = Some(*note);
                    self.volume = 0.2;
                },
            }
        }
    }

}

pub struct SineOscillator {
    pub params: OscillatorParams,
    pub midi_input_buffer: Rc<RefCell<midi::InputBuffer>>,
}
pub struct SquareOscillator   {
    pub params: OscillatorParams,
    pub midi_input_buffer: Rc<RefCell<midi::InputBuffer>>,
}
pub struct SawtoothOscillator {
    pub params: OscillatorParams,
    pub midi_input_buffer: Rc<RefCell<midi::InputBuffer>>,
}

/// This is the code that implements the Oscillator trait for the SineOscillator struct
impl<S> Generator<S> for SineOscillator {
    fn update_state(&mut self) {
        self.params.update_state(Rc::clone(&self.midi_input_buffer))
    }

    fn generate(&mut self) -> S
    where S: dsp::Sample + dsp::FromSample<f32> + fmt::Display,
    {
        let params = &mut self.params;
        let res = (params.phase.sin() as f32 * params.volume).to_sample::<S>();

        params.phase += 2.0 * PI * params.frequency / defs::SAMPLE_HZ;
        while params.phase >= PI {
            println!("Phase bounding (pre): {}", params.phase);
            params.phase -= PI * 2.0;
            println!("Phase bounding (post): {}", params.phase);
        }

        res
    }
}

/// This is the code that implements the Oscillator trait for the SquareOscillator struct
impl<S> Generator<S> for SquareOscillator {
    fn update_state(&mut self) {
        self.params.update_state(Rc::clone(&self.midi_input_buffer))
    }

    fn generate(&mut self) -> S
    where S: dsp::Sample + dsp::FromSample<f32> + fmt::Display,
    {
        // TODO: Do something with the midi events
        let midi_events = self.midi_input_buffer.borrow();
        let _midi_event_iter = midi_events.iter();

        let params = &mut self.params;
        let res = if params.phase < 0.0 {
            params.volume
        } else {
            -params.volume
        };
        let res = res.to_sample::<S>();

        params.phase += 2.0 * PI * params.frequency / defs::SAMPLE_HZ;
        while params.phase >= PI {
            params.phase -= PI * 2.0;
        }

        res
    }
}

/// This is the code that implements the Oscillator trait for the SquareOscillator struct
impl<S> Generator<S> for SawtoothOscillator {
    fn update_state(&mut self) {
        self.params.update_state(Rc::clone(&self.midi_input_buffer))
    }

    fn generate(&mut self) -> S
    where S: dsp::Sample + dsp::FromSample<f32> + fmt::Display,
    {
        // TODO: Do something with the midi events
        let midi_events = self.midi_input_buffer.borrow();
        let _midi_event_iter = midi_events.iter();

        let params = &mut self.params;
        let res = ((PI - (params.phase)) as f32 * params.volume).to_sample::<S>();

        params.phase += 2.0 * PI * params.frequency / defs::SAMPLE_HZ;
        while params.phase >= PI {
            params.phase -= PI * 2.0;
        }

        res
    }
}
