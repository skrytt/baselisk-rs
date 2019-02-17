
use defs;
use dsp;

pub struct Engine {

}

impl Engine {
    pub fn new() -> Engine {
        Engine{}
    }

    pub fn audio_requested(&mut self, output_buffer: &mut [defs::Frame], _sample_rate: f64) {
        dsp::slice::equilibrium(output_buffer);
    }
}
