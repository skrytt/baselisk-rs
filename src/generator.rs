
extern crate dsp;

use std::f64::consts::PI;
use std::fmt;
use dsp::Sample;
use defs;

/// An oscillator must implement the Oscillator trait
pub trait Generator<S> {
    fn generate(&mut self) -> S
    where S: dsp::Sample + dsp::FromSample<f32> + fmt::Display;
}

/// SineOscillator is a type that will implement the trait above:
pub struct OscillatorParams {
    pub phase: defs::Phase,
    pub frequency: defs::Frequency,
    pub volume: defs::Volume,
}

impl OscillatorParams {
    pub fn new(frequency: defs::Frequency, volume: defs::Volume) -> OscillatorParams {
        OscillatorParams{
            phase: 0.0,
            frequency,
            volume
        }
    }
}

pub struct SineOscillator     { pub params: OscillatorParams }
pub struct SquareOscillator   { pub params: OscillatorParams }
pub struct SawtoothOscillator { pub params: OscillatorParams }

/// This is the code that implements the Oscillator trait for the SineOscillator struct
impl<S> Generator<S> for SineOscillator {
    fn generate(&mut self) -> S
    where S: dsp::Sample + dsp::FromSample<f32> + fmt::Display,
    {
        let params = &mut self.params;
        let res = ((params.phase * PI * 2.0).sin() as f32 * params.volume).to_sample::<S>();

        params.phase += params.frequency / defs::SAMPLE_HZ;
        while params.phase >= PI * 2.0 {
            params.phase -= PI * 2.0;
        }

        res
    }
}

/// This is the code that implements the Oscillator trait for the SquareOscillator struct
impl<S> Generator<S> for SquareOscillator {
    fn generate(&mut self) -> S
    where S: dsp::Sample + dsp::FromSample<f32> + fmt::Display,
    {
        let params = &mut self.params;
        let res = if params.phase < PI {
            params.volume
        } else {
            -params.volume
        };
        let res = res.to_sample::<S>();

        params.phase += params.frequency / defs::SAMPLE_HZ;
        while params.phase >= PI * 2.0 {
            params.phase -= PI * 2.0;
        }

        res
    }
}

/// This is the code that implements the Oscillator trait for the SquareOscillator struct
impl<S> Generator<S> for SawtoothOscillator {
    fn generate(&mut self) -> S
    where S: dsp::Sample + dsp::FromSample<f32> + fmt::Display,
    {
        let params = &mut self.params;
        let res = ((1.0 - (params.phase / PI)) as f32 * params.volume).to_sample::<S>();

        params.phase += params.frequency / defs::SAMPLE_HZ;
        while params.phase >= PI * 2.0 {
            params.phase -= PI * 2.0;
        }

        res
    }
}
