extern crate sample;

pub mod adsr;
//pub mod delay;
pub mod gain;
pub mod oscillator;
pub mod filter;
pub mod modmatrix;
pub mod note_selector;
pub mod pitch_bend;
pub mod waveshaper;

pub use processor::{
    adsr::Adsr as Adsr,
    //delay::Delay as Delay,
    gain::Gain as Gain,
    oscillator::Oscillator as Oscillator,
    filter::Filter as Filter,
    modmatrix::ModulationMatrix as ModulationMatrix,
    note_selector::MonoNoteSelector as MonoNoteSelector,
    pitch_bend::PitchBend as PitchBend,
    waveshaper::Waveshaper as Waveshaper,
};
