extern crate sample;

use defs;
use event;
use std::rc::Rc;
use std::cell::RefCell;


/// Internal state used by oscillator types.
#[derive(Clone)]
pub struct State {
    note: u8,
    pitch_offset: defs::Sample, // Semitones
    pitch_bend: defs::Sample,   // Semitones
    pulse_width: defs::Sample,  // 0.001 <= pulse_width <= 0.999
    phase: defs::Sample,        // 0 <= phase <= 1
    frequency: defs::Sample,
    sample_rate: defs::Sample,
}

impl State {
    pub fn new() -> State {
        State {
            note: 69,
            pitch_offset: 0.0,
            pitch_bend: 0.0,
            pulse_width: 0.5,
            phase: 0.0,
            frequency: 0.0,
            sample_rate: 0.0,
        }
    }

    pub fn set_pitch_bend(&mut self, value: u16, range: defs::Sample) {
        // Value is 14-bit (range 0 <= value <= 16383)
        // 0 => -2
        // 8192 => 0
        // 16383 => ~= +2
        self.pitch_bend = range * (value as defs::Sample - 8192.0) / 8192.0;
    }

    /// Convert a u8 note number to a corresponding frequency,
    /// using 440 Hz as the pitch of the A above middle C.
    fn update_frequency(&mut self) {
        self.frequency = 440.0 * ((self.note as defs::Sample + self.pitch_offset + self.pitch_bend - 69.0) / 12.0).exp2()
    }

    /// Process any events and update the internal state accordingly.
    fn update(&mut self, event_buffer: &Rc<RefCell<event::Buffer>>, sample_rate: defs::Sample) {
        // Iterate over any midi events and mutate the frequency accordingly
        self.sample_rate = sample_rate;
        let events = event_buffer.borrow();
        for event in events.iter_midi() {
            if let event::Event::Midi(midi_event) = event {
                match midi_event {
                    event::MidiEvent::NoteOn { note, .. } => {
                        // Set the active note and frequency to match this new note
                        self.note = *note;
                        self.update_frequency();
                    },
                    event::MidiEvent::PitchBend { value } => {
                        self.set_pitch_bend(*value, 2.0);
                        self.update_frequency();
                    }
                    _ => (),
                }
            }
        }
    }
}

/// Oscillator type that will be used for audio processing.
pub struct Oscillator {
    state: State,
    event_buffer: Rc<RefCell<event::Buffer>>,
    generator_func: fn(&mut State) -> defs::Sample,
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
            "pulse" => pulse_generator,
            _ => return Err("Unknown oscillator name"),
        };
        self.generator_func = generator_func;
        Ok(())
    }

    pub fn set_pitch(&mut self, semitones: defs::Sample) -> Result<(), &'static str>
    {
        if semitones < -36.0 {
            Err("Pitch offset must be >= -36.0 semitones")
        } else if semitones > 36.0 {
            Err("Pitch offset must be <= 36.0 semitones")
        } else {
            self.state.pitch_offset = semitones;
            Ok(())
        }
    }

    pub fn set_pulse_width(&mut self, width: defs::Sample) -> Result<(), &'static str>
    {
        if width < 0.001 || width > 0.999 {
            Err("Pulse width must be in range 0.001 <= width <= 0.999")
        } else {
            self.state.pulse_width = width;
            Ok(())
        }
    }

    pub fn process_buffer(&mut self,
               buffer: &mut defs::FrameBuffer,
               sample_rate: defs::Sample,
    ) {
        self.state.update(&self.event_buffer, sample_rate);

        // Generate all the samples for this buffer
        for frame in buffer.iter_mut() {
            let sample: defs::Sample = (self.generator_func)(&mut self.state);
            let this_frame: defs::Frame = [sample];
            *frame = this_frame;
        }
    }
}

/// Generator function that produces a sine wave.
fn sine_generator(state: &mut State) -> defs::Sample
{
    let res = state.phase.sin();

    state.phase += 2.0 as defs::Sample * defs::PI * state.frequency / state.sample_rate;
    while state.phase >= defs::PI {
        state.phase -= defs::PI * 2.0;
    }

    res
}

/// Generator function that produces a pulse wave.
/// Uses PolyBLEP smoothing to reduce aliasing.
fn pulse_generator(state: &mut State) -> defs::Sample
{
    let step = state.frequency / state.sample_rate;

    // Advance phase
    // Enforce range 0.0 <= phase < 1.0
    let mut phase = state.phase + step;
    while phase >= 1.0 {
        phase -= 1.0;
    }

    let mut res = if phase < state.pulse_width {
        1.0
    } else {
        -1.0
    };

    // PolyBLEP smoothing to reduce aliasing by smoothing discontinuities,
    let polyblep = |phase: defs::Sample, step: defs::Sample| -> defs::Sample {
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
    res -= polyblep((phase + 1.0 - state.pulse_width) % 1.0, step);

    // Store the phase for next iteration
    state.phase = phase;

    res as f32
}

/// Generator function that produces a sawtooth wave.
/// Uses PolyBLEP smoothing to reduce aliasing.
fn sawtooth_generator(state: &mut State) -> defs::Sample
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

    res as defs::Sample
}
