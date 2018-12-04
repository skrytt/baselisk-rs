pub type Volume = f32;
pub type Output = f32;
pub type Frame = [Output; CHANNELS];

pub const CHANNELS: usize = 1;
pub const FRAMES: u32 = 0;
pub const MIDI_BUF_LEN: usize = 1024;
pub const PROGNAME: &str = "baselisk";
