extern crate dsp;

use defs;
use dsp::Sample;
use event;
use modulator;
use processor;
use std::fmt;
use std::sync::{Arc, RwLock};

/// AdsrGain links together a midi::InputBuffer and an ADSR into an audio processor.
/// It knows how to adjust gain based on MIDI events.
struct AdsrGain {
    adsr: modulator::Adsr,
    event_buffer: Arc<RwLock<event::Buffer>>,
    volume: defs::Volume,
}

impl AdsrGain {
    fn new(event_buffer: Arc<RwLock<event::Buffer>>) -> AdsrGain {
        AdsrGain {
            adsr: modulator::Adsr::new(defs::SAMPLE_HZ),
            event_buffer,
            volume: 0.2,
        }
    }
}

impl<S> processor::Processor<S> for AdsrGain {
    fn type_name(&self) -> &'static str {
        "AdsrGain"
    }

    fn update_state(&mut self) {
        let mut notes_pressed: i32 = 0;
        let mut notes_released: i32 = 0;

        let events = self.event_buffer.try_read()
            .expect("Event buffer unexpectedly locked");
        for event in events.iter() {
            if let event::Event::Midi(midi_event) = event {
                match midi_event {
                    event::MidiEvent::NoteOn { .. } => {
                        notes_pressed += 1;
                    }
                    event::MidiEvent::NoteOff { .. } => {
                        notes_released += 1;
                    }
                }
            }
        }
        if notes_pressed > 0 || notes_released > 0 {
            self.adsr
                .update_notes_held_count(notes_pressed, notes_released);
        }
    }

    fn process(&mut self, input: S) -> S
    where
        S: dsp::sample::FloatSample + dsp::FromSample<f32> + fmt::Display,
    {
        let next = (self.adsr.next().unwrap() * self.volume).to_sample::<S>();
        input.mul_amp(next)
    }
}

pub fn new<S>(
    name: &str,
    event_buffer: Arc<RwLock<event::Buffer>>,
) -> Result<Box<dyn processor::Processor<S>>, &'static str> {
    match name {
        "adsrgain" => Ok(Box::new(AdsrGain::new(event_buffer))),
        _ => Err("Unknown gain filter name"),
    }
}
