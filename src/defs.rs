use std::f64;
use sample::frame;

pub const MIDI_BUF_LEN: usize = 1024;
pub const JACK_CLIENT_NAME: &'static str = "baselisk";
pub const PROMPT: &'static str = "baselisk> ";

pub type Sample = f32;                // A single sample
pub type Frame = frame::Mono<Sample>; // A slice of samples
pub type FrameBuffer = [Frame];       // A slice of frames

pub const PI: Sample = f64::consts::PI as Sample;


