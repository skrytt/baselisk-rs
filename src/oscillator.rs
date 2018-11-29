extern crate dsp;

use defs;
use dsp::Sample;
use midi;
use processor;
use std::cell::RefCell;
use std::f64::consts::PI;
use std::fmt;
use std::sync::Arc;

pub struct Params {
    phase: defs::Phase,
    frequency: defs::Frequency,
}

impl Params {
    pub fn new() -> Params {
        Params {
            phase: 0.0,
            frequency: 0.0,
        }
    }

    fn update_state(&mut self, midi_input_buffer: Arc<RefCell<midi::InputBuffer>>) {
        // Iterate over any midi events and mutate the frequency accordingly
        let midi_events = midi_input_buffer.borrow();
        for midi_event in midi_events.iter() {
            match midi_event {
                midi::MidiEvent::NoteOn { note, .. } => {
                    // Set the active note and frequency to match this new note
                    self.frequency = midi::note_to_frequency(*note);
                }
                _ => (),
            }
        }
    }
}

pub struct Oscillator<S> {
    name: String,
    params: Params,
    midi_input_buffer: Arc<RefCell<midi::InputBuffer>>,
    generator_func: fn(&mut Params) -> S,
}

pub fn new<S>(
    name: &str,
    midi_input_buffer: Arc<RefCell<midi::InputBuffer>>,
) -> Result<Box<dyn processor::Source<S>>, &'static str>
where
    S: dsp::Sample + dsp::FromSample<f32> + fmt::Display + 'static,
{
    let generator_func = match name {
        "sine" => sine_generator,
        "saw" => sawtooth_generator,
        "square" => square_generator,
        _ => return Err("Unknown oscillator name"),
    };
    Ok(Box::new(Oscillator {
        name: String::from(name),
        params: Params::new(),
        midi_input_buffer,
        generator_func,
    }))
}

/// This is the code that implements the Oscillator trait for the SineOscillator struct
impl<S> processor::Source<S> for Oscillator<S> {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn update_state(&mut self) {
        self.params
            .update_state(Arc::clone(&self.midi_input_buffer))
    }

    fn generate(&mut self) -> S {
        (self.generator_func)(&mut self.params)
    }
}

fn sine_generator<S>(params: &mut Params) -> S
where
    S: dsp::Sample + dsp::FromSample<f32> + fmt::Display,
{
    let res = (params.phase.sin() as f32).to_sample::<S>();

    params.phase += 2.0 * PI * params.frequency / defs::SAMPLE_HZ;
    while params.phase >= PI {
        params.phase -= PI * 2.0;
    }

    res
}

fn square_generator<S>(params: &mut Params) -> S
where
    S: dsp::Sample + dsp::FromSample<f32> + fmt::Display,
{
    let step = params.frequency / defs::SAMPLE_HZ;

    // Advance phase
    // Enforce range 0.0 <= phase < 1.0
    let mut phase = params.phase + step;
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
    params.phase = phase;

    (res as f32).to_sample::<S>()
}

fn sawtooth_generator<S>(params: &mut Params) -> S
where
    S: dsp::Sample + dsp::FromSample<f32> + fmt::Display,
{
    let step = params.frequency / defs::SAMPLE_HZ;

    // Advance phase
    // Enforce range 0.0 <= phase < 1.0
    let mut phase = params.phase + step;
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
    params.phase = phase;

    (res as f32).to_sample::<S>()
}
