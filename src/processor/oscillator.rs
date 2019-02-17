extern crate dsp;

use dsp::Sample;
use event;
use processor;
use std::f64::consts::PI;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone)]

/// Internal state used by oscillator types.
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

    /// Process any events and update the internal state accordingly.
    fn update_state(&mut self, event_buffer: &Rc<RefCell<event::Buffer>>, sample_rate: f64) {
        // Iterate over any midi events and mutate the frequency accordingly
        self.sample_rate = sample_rate;
        let events = event_buffer.borrow();
        for event in events.iter_midi() {
            if let event::Event::Midi(midi_event) = event {
                match midi_event {
                    event::MidiEvent::NoteOn { note, .. } => {
                        // Set the active note and frequency to match this new note
                        self.frequency = note_to_frequency(*note);
                    }
                    _ => (),
                }
            }
        }
    }
}

/// Convert a u8 note number to a corresponding frequency,
/// using 440 Hz as the pitch of the A above middle C.
pub fn note_to_frequency(note: u8) -> f64 {
    440.0 * ((note as f64 - 69.0) / 12.0).exp2()
}

/// View representation of an oscillator.
pub struct OscillatorView {
    name: String,
}

impl processor::ProcessorView for OscillatorView {
    fn name(&self) -> String {
        self.name.clone()
    }
}

/// Oscillator type that will be used for audio processing.
pub struct Oscillator<S> {
    name: String,
    state: State,
    event_buffer: Rc<RefCell<event::Buffer>>,
    generator_func: fn(&mut State) -> S,
}

impl<S> Oscillator<S> {
/// Function to construct new oscillators.
    pub fn new(
        event_buffer: &Rc<RefCell<event::Buffer>>,
    ) -> Result<Oscillator<S>, &'static str>
    where
        S: dsp::Sample + dsp::FromSample<f32> + fmt::Display + 'static,
    {
        let generator_func = sine_generator;
        let state = State::new();

        Ok(Oscillator {
            name: String::from("oscillator"),
            state: state.clone(),
            event_buffer: Rc::clone(event_buffer),
            generator_func,
        })
    }

    pub fn set_type(&mut self, type_name: &str) -> Result<(), &'static str>
    where
        S: dsp::Sample + dsp::FromSample<f32> + fmt::Display + 'static,
    {
        let generator_func = match type_name {
            "sine" => sine_generator,
            "saw" => sawtooth_generator,
            "square" => square_generator,
            _ => return Err("Unknown oscillator name"),
        };
        self.generator_func = generator_func;
        Ok(())
    }

    pub fn process_buffer(&mut self,
               output_buffer: &mut [[S; 1]],
               _sample_rate: f64,
    ) {
        // Generate all the samples for this buffer
        for frame in output_buffer.iter_mut() {
            let sample: S = (self.generator_func)(&mut self.state);
            let this_frame: [S; 1] = [sample];
            *frame = this_frame;
        }
    }
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
        self.state.update_state(&self.event_buffer, sample_rate)
    }

    fn get_view(&self) -> Box<dyn processor::ProcessorView> {
        Box::new(OscillatorView {
            name: self.name.clone(),
        })
    }
}

/// Generator function that produces a sine wave.
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

/// Generator function that produces a square wave.
/// Uses PolyBLEP smoothing to reduce aliasing.
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

/// Generator function that produces a sawtooth wave.
/// Uses PolyBLEP smoothing to reduce aliasing.
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
