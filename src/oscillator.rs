
extern crate dsp;

use std::fmt;
use std::f64::consts::PI;
use std::sync::Arc;
use std::cell::RefCell;
use dsp::Sample;
use defs;
use midi;
use processor;

pub struct Params {
    phase: defs::Phase,
    frequency: defs::Frequency,
}

impl Params {
    pub fn new() -> Params {
        Params{
            phase: 0.0,
            frequency: 0.0,
        }
    }

    fn update_state(&mut self, midi_input_buffer: Arc<RefCell<midi::InputBuffer>>) {
        // Iterate over any midi events and mutate the frequency accordingly
        let midi_events = midi_input_buffer.borrow();
        for midi_event in midi_events.iter() {
            match midi_event {
                midi::MidiEvent::NoteOn{note, ..} => {
                    // Set the active note and frequency to match this new note
                    self.frequency = midi::note_to_frequency(*note);
                },
                _ => ()
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
        let res = (params.phase.sin() as f32).to_sample::<S>();

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
        let mut res = if phase < 0.5 {
            1.0
        } else {
            -1.0
        };

        // PolyBLEP smoothing to reduce aliasing by smoothing discontinuities,
        let polyblep = |phase: f64, step: f64| -> f64 {
            // Apply PolyBLEP Smoothing for 0 < phase < (freq / sample_rate)
            //   phase == 0:    x = 0.0
            //   phase == step: x = 1.0
            if phase < step {
                let x = phase / step;
                return 2.0*x - x*x - 1.0;
            }
            // Apply PolyBLEP Smoothing for (1.0 - (freq / sample_rate)) < phase < 1.0:
            //   phase == (1.0 - step): x = 1.0
            //   phase == 1.0:          x = 0.0
            else if phase > (1.0 - step) {
                let x = (phase - 1.0) / step;
                return 2.0*x + x*x + 1.0;
            }
            else {
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
        let mut res = 1.0 - 2.0*phase;

        // PolyBLEP smoothing to reduce aliasing by smoothing discontinuities,
        // which always occur at phase == 0.0.
        // Apply PolyBLEP Smoothing for 0 < phase < (freq / sample_rate)
        //   phase == 0:    x = 0.0
        //   phase == step: x = 1.0
        if phase < step {
            let x = phase / step;
            res += 2.0*x - x*x - 1.0;
        }
        // Apply PolyBLEP Smoothing for (1.0 - (freq / sample_rate)) < phase < 1.0:
        //   phase == (1.0 - step): x = 1.0
        //   phase == 1.0:          x = 0.0
        else if phase > (1.0 - step) {
            let x = (phase - 1.0) / step;
            res += 2.0*x + x*x + 1.0;
        }

        // Store the phase for next iteration
        params.phase = phase;

        (res as f32).to_sample::<S>()
    }
}
