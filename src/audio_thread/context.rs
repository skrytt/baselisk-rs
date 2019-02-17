
extern crate portmidi;

use audio_thread;
use comms;
use defs;
use event;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Context {
    pub engine: audio_thread::Engine<defs::Output>,
    pub comms: comms::AudioThreadComms,
    // Events is refcounted because audio nodes also need to hold references to it
    pub events: Rc<RefCell<event::Buffer>>,
}

impl Context {
    pub fn new(
        comms: comms::AudioThreadComms,
        portmidi: portmidi::PortMidi,
    ) -> Context {
        let events = Rc::new(RefCell::new(event::Buffer::new(portmidi)));
        let engine = audio_thread::Engine::new(&events);

        Context {
            engine,
            comms,
            events,
        }
    }
}
