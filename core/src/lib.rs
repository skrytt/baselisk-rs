//! Synthesizer.
//!
extern crate sample;
extern crate rand;

#[cfg(feature = "jack")]
extern crate rustyline;

extern crate time;

// vst crate used here for AtomicFloat type
extern crate vst;

#[cfg(feature = "jack")]
extern crate jack;

#[cfg(feature = "jack")]
pub mod cli;

pub mod defs;
pub mod engine;
pub mod shared;
