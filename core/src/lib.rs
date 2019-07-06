//! Synthesizer.
//!
extern crate sample;
extern crate rand;

#[cfg(feature = "plugin_jack")]
extern crate rustyline;

extern crate time;

// vst crate used here for AtomicFloat type
extern crate vst;

#[cfg(feature = "plugin_jack")]
extern crate jack;

#[cfg(feature = "plugin_jack")]
pub mod cli;

pub mod defs;
pub mod engine;
pub mod shared;
