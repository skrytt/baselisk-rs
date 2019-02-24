
pub mod adsr;
pub mod gain;
pub mod oscillator;
pub mod filter;
pub mod waveshaper;

pub use processor::{
    adsr::Adsr as Adsr,
    gain::Gain as Gain,
    oscillator::Oscillator as Oscillator,
    filter::LowPassFilter as LowPassFilter,
    waveshaper::Waveshaper as Waveshaper,
};

extern crate dsp;

