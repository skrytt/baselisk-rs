
extern crate portmidi;

use comms;
use defs;
use dsp;
use dsp_node;
use event;
use std::rc::Rc;
use std::cell::RefCell;

pub type Graph = dsp::Graph<defs::Frame, dsp_node::DspNode<f32>>;

pub struct AudioThreadContext {
    pub graph: Graph,
    pub selected_node: dsp::NodeIndex,
    pub comms: comms::AudioThreadComms,
    // Events is refcounted because audio nodes also need to hold references to it
    pub events: Rc<RefCell<event::Buffer>>,
}

impl AudioThreadContext {
    pub fn new(
        comms: comms::AudioThreadComms,
        portmidi: portmidi::PortMidi,
    ) -> AudioThreadContext {
        let mut graph = dsp::Graph::new();
        let master_node = graph.add_node(dsp_node::DspNode::Master);
        graph.set_master(Some(master_node));

        AudioThreadContext {
            graph,
            selected_node: master_node,
            comms,
            events: Rc::new(RefCell::new(event::Buffer::new(portmidi))),
        }
    }
}
