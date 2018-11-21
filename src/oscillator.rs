
extern crate dsp;

use std::fmt;
use std::f64::consts::PI;
use std::sync::Arc;
use std::cell::RefCell;
use dsp::Sample;
use defs;
use midi;
use processor;

/// SineOscillator is a type that will implement the trait above:
pub struct Params {
    pub phase: defs::Phase,
    pub frequency: defs::Frequency,
    pub volume: defs::Volume,
    pub note: Option<u8>,
}

impl Params {
    pub fn new() -> Params {
        Params{
            phase: 0.0,
            frequency: 0.0,
            volume: 0.2,
            note: None
        }
    }

    fn update_state(&mut self, midi_input_buffer: Arc<RefCell<midi::InputBuffer>>) {
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
    pub params: Params,
    pub midi_input_buffer: Arc<RefCell<midi::InputBuffer>>,
}
pub struct SquareOscillator   {
    pub params: Params,
    pub midi_input_buffer: Arc<RefCell<midi::InputBuffer>>,
}
pub struct SawtoothOscillator {
    pub params: Params,
    pub midi_input_buffer: Arc<RefCell<midi::InputBuffer>>,
}

pub fn new<S>(name: &str, midi_input_buffer: Arc<RefCell<midi::InputBuffer>>) -> Result<Box<dyn processor::Source<S>>, &'static str> {
    match name {
        "sine"   => Ok(Box::new(SineOscillator{ params: Params::new(), midi_input_buffer })),
        "square" => Ok(Box::new(SquareOscillator{ params: Params::new(), midi_input_buffer })),
        "saw"    => Ok(Box::new(SawtoothOscillator{ params: Params::new(), midi_input_buffer })),
        _        => Err("Unknown oscillator name"),
    }

}

/// This is the code that implements the Oscillator trait for the SineOscillator struct
impl<S> processor::Source<S> for SineOscillator {
    fn type_name(&self) -> &'static str {
        "SineOscillator"
    }

    fn update_state(&mut self) {
        self.params.update_state(Arc::clone(&self.midi_input_buffer))
    }

    fn generate(&mut self) -> S
    where S: dsp::Sample + dsp::FromSample<f32> + fmt::Display,
    {
        let params = &mut self.params;
        let res = (params.phase.sin() as f32 * params.volume).to_sample::<S>();

        params.phase += 2.0 * PI * params.frequency / defs::SAMPLE_HZ;
        while params.phase >= PI {
            params.phase -= PI * 2.0;
        }

        res
    }
}

/// This is the code that implements the Oscillator trait for the SquareOscillator struct
impl<S> processor::Source<S> for SquareOscillator {
    fn type_name(&self) -> &'static str {
        "SquareOscillator"
    }

    fn update_state(&mut self) {
        self.params.update_state(Arc::clone(&self.midi_input_buffer))
    }

    fn generate(&mut self) -> S
    where S: dsp::Sample + dsp::FromSample<f32> + fmt::Display,
    {
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
impl<S> processor::Source<S> for SawtoothOscillator {
    fn type_name(&self) -> &'static str {
        "SawtoothOscillator"
    }

    fn update_state(&mut self) {
        self.params.update_state(Arc::clone(&self.midi_input_buffer))
    }

    fn generate(&mut self) -> S
    where S: dsp::Sample + dsp::FromSample<f32> + fmt::Display,
    {
        let params = &mut self.params;
        let res = ((PI - (params.phase)) as f32 * params.volume).to_sample::<S>();

        params.phase += 2.0 * PI * params.frequency / defs::SAMPLE_HZ;
        while params.phase >= PI {
            params.phase -= PI * 2.0;
        }

        res
    }
}
