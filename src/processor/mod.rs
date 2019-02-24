
pub mod adsr;
pub mod gain;
pub mod oscillator;
pub mod filter;

pub use processor::adsr::Adsr as Adsr;
pub use processor::gain::Gain as Gain;
pub use processor::oscillator::Oscillator as Oscillator;
pub use processor::filter::LowPassFilter as LowPassFilter;

extern crate dsp;

