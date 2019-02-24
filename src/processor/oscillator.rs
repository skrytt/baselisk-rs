extern crate dsp;

use defs;
use dsp::sample::frame;
use event;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone)]

/// Internal state used by oscillator types.
pub struct State {
    phase: defs::Output,
    frequency: defs::Output,
    sample_rate: defs::Output,
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
    fn update(&mut self, event_buffer: &Rc<RefCell<event::Buffer>>, sample_rate: defs::Output) {
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
pub fn note_to_frequency(note: u8) -> defs::Output {
    440.0 as defs::Output * ((note as defs::Output - 69.0) / 12.0).exp2()
}

/// Oscillator type that will be used for audio processing.
pub struct Oscillator {
    state: State,
    event_buffer: Rc<RefCell<event::Buffer>>,
    generator_func: fn(&mut State) -> defs::Output,
}

impl Oscillator {
/// Function to construct new oscillators.
    pub fn new(
        event_buffer: &Rc<RefCell<event::Buffer>>,
    ) -> Oscillator
    {
        let generator_func = sine_generator;
        let state = State::new();

        Oscillator {
            state: state.clone(),
            event_buffer: Rc::clone(event_buffer),
            generator_func,
        }
    }

    pub fn set_type(&mut self, type_name: &str) -> Result<(), &'static str>
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
               buffer: &mut [frame::Mono<defs::Output>],
               sample_rate: defs::Output,
    ) {
        self.state.update(&self.event_buffer, sample_rate);

        // Generate all the samples for this buffer
        for frame in buffer.iter_mut() {
            let sample: defs::Output = (self.generator_func)(&mut self.state);
            let this_frame: frame::Mono<defs::Output> = [sample];
            *frame = this_frame;
        }
    }
}

/// Generator function that produces a sine wave.
fn sine_generator(state: &mut State) -> defs::Output
{
    let res = state.phase.sin();

    state.phase += 2.0 as defs::Output * defs::PI * state.frequency / state.sample_rate;
    while state.phase >= defs::PI {
        state.phase -= defs::PI * 2.0;
    }

    res
}

/// Generator function that produces a square wave.
/// Uses PolyBLEP smoothing to reduce aliasing.
fn square_generator(state: &mut State) -> defs::Output
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
    let polyblep = |phase: defs::Output, step: defs::Output| -> defs::Output {
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

    res as f32
}

/// Generator function that produces a sawtooth wave.
/// Uses PolyBLEP smoothing to reduce aliasing.
fn sawtooth_generator(state: &mut State) -> defs::Output
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

    res as defs::Output
}
