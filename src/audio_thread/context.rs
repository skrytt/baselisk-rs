
extern crate portmidi;

use audio_thread;
use comms;
use defs;
use dsp;
use dsp_unit;
use event;
use std::rc::Rc;
use std::cell::RefCell;

pub type Graph = dsp::Graph<defs::Frame, dsp_unit::DspUnit<f32>>;

pub struct Context {
    pub engine: audio_thread::Engine,
    pub comms: comms::AudioThreadComms,
    // Events is refcounted because audio nodes also need to hold references to it
    pub events: Rc<RefCell<event::Buffer>>,
}

impl Context {
    pub fn new(
        comms: comms::AudioThreadComms,
        portmidi: portmidi::PortMidi,
    ) -> Context {
        Context {
            engine: audio_thread::Engine::new(),
            comms,
            events: Rc::new(RefCell::new(event::Buffer::new(portmidi))),
        }
    }
}
