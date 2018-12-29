extern crate dsp;

use dsp::Sample;
use event;
use processor;
use std::f64::consts::PI;
use std::fmt;
use std::sync::{Arc, RwLock};

#[derive(Clone)]

pub struct State {
    phase: f64,
    frequency: f64,
    sample_rate: f64,
}

impl State {
    pub fn new() -> State {
        State {
            phase: 0.0,
            frequency: 0.0,
            sample_rate: 0.0,
        }
    }

    fn update_state(&mut self, event_buffer: &Arc<RwLock<event::Buffer>>, sample_rate: f64) {
        // Iterate over any midi events and mutate the frequency accordingly
        self.sample_rate = sample_rate;
        let events = event_buffer.try_read()
            .expect("Event buffer unexpectedly locked");
        for event in events.iter_midi() {
            if let event::Event::Midi(midi_event) = event {
                match midi_event {
                    event::MidiEvent::NoteOn { note, .. } => {
                        // Set the active note and frequency to match this new note
                        self.frequency = event::midi::note_to_frequency(*note);
                    }
                    _ => (),
                }
            }
        }
    }
}

pub struct OscillatorView {
    name: String,
}

impl processor::ProcessorView for OscillatorView {
    fn name(&self) -> String {
        self.name.clone()
    }
}

pub struct Oscillator<S> {
    name: String,
    state: State,
    event_buffer: Arc<RwLock<event::Buffer>>,
    generator_func: fn(&mut State) -> S,
}

/// Function to construct new oscillators
pub fn new<S>(
    name: &str,
    event_buffer: Arc<RwLock<event::Buffer>>,
) -> Result<(Box<dyn processor::Processor<S>>, Box<dyn processor::ProcessorView>), &'static str>
where
    S: dsp::Sample + dsp::FromSample<f32> + fmt::Display + 'static,
{
    let generator_func = match name {
        "sine" => sine_generator,
        "saw" => sawtooth_generator,
        "square" => square_generator,
        _ => return Err("Unknown oscillator name"),
    };
    let state = State::new();
    Ok((
        Box::new(Oscillator {
            name: String::from(name),
            state: state.clone(),
            event_buffer,
            generator_func,
        }),
        Box::new(OscillatorView {
            name: String::from(name),
        }),
    ))
}
impl<S> processor::ProcessorView for Oscillator<S> {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn details(&self) -> String {
        String::from("")
    }
}

impl<S> processor::Processor<S> for Oscillator<S> {
    fn update_state(&mut self, sample_rate: f64) {
        self.state
            .update_state(&self.event_buffer, sample_rate)
    }

    fn process(&mut self, _input: S) -> S {
        (self.generator_func)(&mut self.state)
    }
}

fn sine_generator<S>(state: &mut State) -> S
where
    S: dsp::Sample + dsp::FromSample<f32> + fmt::Display,
{
    let res = (state.phase.sin() as f32).to_sample::<S>();

    state.phase += 2.0 * PI * state.frequency / state.sample_rate;
    while state.phase >= PI {
        state.phase -= PI * 2.0;
    }

    res
}

fn square_generator<S>(state: &mut State) -> S
where
    S: dsp::Sample + dsp::FromSample<f32> + fmt::Display,
{
    let step = state.frequency / state.sample_rate;

    // Advance phase
    // Enforce range 0.0 <= phase < 1.0
    let mut phase = state.phase + step;
    while phase >= 1.0 {
        phase -= 1.0;
    }

    // Naive sawtooth is:
    //   phase == 0:   res = 1.0
    //   phase == 0.5: res = 0.0
    //   phase == 1.0: res = -1.0
    let mut res = if phase < 0.5 { 1.0 } else { -1.0 };

    // PolyBLEP smoothing to reduce aliasing by smoothing discontinuities,
    let polyblep = |phase: f64, step: f64| -> f64 {
        // Apply PolyBLEP Smoothing for 0 < phase < (freq / sample_rate)
        //   phase == 0:    x = 0.0
        //   phase == step: x = 1.0
        if phase < step {
            let x = phase / step;
            return 2.0 * x - x * x - 1.0;
        }
        // Apply PolyBLEP Smoothing for (1.0 - (freq / sample_rate)) < phase < 1.0:
        //   phase == (1.0 - step): x = 1.0
        //   phase == 1.0:          x = 0.0
        else if phase > (1.0 - step) {
            let x = (phase - 1.0) / step;
            return 2.0 * x + x * x + 1.0;
        } else {
            0.0
        }
    };
    // PolyBLEP for the first (upward) discontinuity
    res += polyblep(phase, step);
    // PolyBLEP for the second (downward) discontinuity
    res -= polyblep((phase + 0.5) % 1.0, step);

    // Store the phase for next iteration
    state.phase = phase;

    (res as f32).to_sample::<S>()
}

fn sawtooth_generator<S>(state: &mut State) -> S
where
    S: dsp::Sample + dsp::FromSample<f32> + fmt::Display,
{
    let step = state.frequency / state.sample_rate;

    // Advance phase
    // Enforce range 0.0 <= phase < 1.0
    let mut phase = state.phase + step;
    while phase >= 1.0 {
        phase -= 1.0;
    }

    // Naive sawtooth is:
    //   phase == 0:   res = 1.0
    //   phase == 0.5: res = 0.0
    //   phase == 1.0: res = -1.0
    let mut res = 1.0 - 2.0 * phase;

    // PolyBLEP smoothing to reduce aliasing by smoothing discontinuities,
    // which always occur at phase == 0.0.
    // Apply PolyBLEP Smoothing for 0 < phase < (freq / sample_rate)
    //   phase == 0:    x = 0.0
    //   phase == step: x = 1.0
    if phase < step {
        let x = phase / step;
        res += 2.0 * x - x * x - 1.0;
    }
    // Apply PolyBLEP Smoothing for (1.0 - (freq / sample_rate)) < phase < 1.0:
    //   phase == (1.0 - step): x = 1.0
    //   phase == 1.0:          x = 0.0
    else if phase > (1.0 - step) {
        let x = (phase - 1.0) / step;
        res += 2.0 * x + x * x + 1.0;
    }

    // Store the phase for next iteration
    state.phase = phase;

    (res as f32).to_sample::<S>()
}
