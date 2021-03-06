use std::f32;
use sample::frame;

pub const RAW_MIDI_BUF_LEN: usize = 1024;
pub const ENGINE_EVENT_BUF_LEN: usize = 1024;
pub const PLUGIN_NAME: &str = "baselisk";
pub const PROMPT: &str = "baselisk> ";

pub type Sample = f32;                       // A single sample
pub type MonoFrame = frame::Mono<Sample>;    // A slice of samples
pub type MonoFrameBufferSlice = [MonoFrame]; // A slice of frames

pub const PI: Sample = f32::consts::PI;
pub const TWOPI: Sample = 2.0 * PI;

