
extern crate dsp;

use std::cell::RefCell;
use std::fmt;
use std::sync::Arc;
use dsp::Sample;
use defs;
use midi;
use modulator;
use processor;

enum StateChange {
    ToAttack,
    ToRelease,
}

/// AdsrGain links together a midi::InputBuffer and an ADSR into an audio processor.
/// It knows how to adjust gain based on MIDI events.
struct AdsrGain {
    adsr: modulator::Adsr,
    midi_input_buffer: Arc<RefCell<midi::InputBuffer>>,
    note: Option<u8>,
}

impl AdsrGain {
    fn new(midi_input_buffer: Arc<RefCell<midi::InputBuffer>>) -> AdsrGain {
        AdsrGain {
            adsr: modulator::Adsr::new(defs::SAMPLE_HZ),
            midi_input_buffer,
            note: None,
        }
    }
}

impl<S> processor::Processor<S> for AdsrGain {
    fn type_name(&self) -> &'static str {
        "AdsrGain"
    }

    fn update_state(&mut self) {
        // Iterate over any midi events and consider the last midi event only
        let mut state_change: Option<StateChange> = None;

        let midi_events = self.midi_input_buffer.borrow();
        for midi_event in midi_events.iter() {
            match midi_event {
                midi::MidiEvent::NoteOff{note} => {
                    println!("debug AdsrGain: got midi NoteOff {}", note);
                    // If this note was already playing, deactivate it
                    if let Some(active_note) = self.note {
                        if *note == active_note {
                            state_change = Some(StateChange::ToRelease);
                            self.note = None;
                        }
                    }
                },
                midi::MidiEvent::NoteOn{note, ..} => {
                    println!("debug AdsrGain: got midi NoteOn {}", note);
                    state_change = Some(StateChange::ToAttack);
                    self.note = Some(*note);
                },
            }
        }
        if let Some(sc) = state_change {
            match sc {
                StateChange::ToAttack => self.adsr.start_attack(),
                StateChange::ToRelease => self.adsr.start_release(),
            }
        }
    }

    fn process(&mut self, input: S) -> S
    where S: dsp::sample::FloatSample + dsp::FromSample<f32> + fmt::Display,
    {
        let next = self.adsr.next().unwrap().to_sample::<S>();
        input.mul_amp(next)
    }
}

pub fn new<S>(name: &str, midi_input_buffer: Arc<RefCell<midi::InputBuffer>>) -> Result<Box<dyn processor::Processor<S>>, &'static str> {
    match name {
        "adsrgain" => Ok(Box::new(AdsrGain::new(midi_input_buffer))),
        _          => Err("Unknown gain filter name"),
    }
}
