extern crate sample;

pub mod adsr;
pub mod gain;
pub mod oscillator;
pub mod filter;
pub mod modmatrix;
pub mod note_selector;
pub mod pitch_bend;
pub mod waveshaper;

pub use processor::{
    adsr::Adsr as Adsr,
    gain::Gain as Gain,
    oscillator::Oscillator as Oscillator,
    filter::LowPassFilter as LowPassFilter,
    modmatrix::ModulationMatrix as ModulationMatrix,
    note_selector::MonoNoteSelector as MonoNoteSelector,
    pitch_bend::PitchBend as PitchBend,
    waveshaper::Waveshaper as Waveshaper,
};
