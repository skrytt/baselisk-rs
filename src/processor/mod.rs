extern crate sample;

pub mod adsr;
pub mod gain;
pub mod oscillator;
pub mod filter;
pub mod note_selector;
pub mod waveshaper;

pub use processor::{
    adsr::Adsr as Adsr,
    gain::Gain as Gain,
    oscillator::Oscillator as Oscillator,
    filter::LowPassFilter as LowPassFilter,
    note_selector::MonoNoteSelector as MonoNoteSelector,
    waveshaper::Waveshaper as Waveshaper,
};
