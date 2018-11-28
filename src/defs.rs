pub type Phase = f64;
pub type Frequency = f64;
pub type Volume = f32;
pub type Output = f32;
pub type Frame = [Output; CHANNELS];

pub const CHANNELS: usize = 1;
pub const FRAMES: u32 = 0;
pub const SAMPLE_HZ: f64 = 44_100.0;
pub const MIDI_BUF_LEN: usize = 1024;
pub const PROGNAME: &str = "baselisk";
