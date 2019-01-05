
extern crate portmidi;

use comms;
use defs;
use dsp;
use dsp_node;
use event;
use std::rc::Rc;
use std::cell::RefCell;

pub type Graph = dsp::Graph<defs::Frame, dsp_node::DspNode<f32>>;

pub struct Context {
    pub graph: Graph,
    pub selected_node: dsp::NodeIndex,
    pub comms: comms::AudioThreadComms,
    // Events is refcounted because audio nodes also need to hold references to it
    pub events: Rc<RefCell<event::Buffer>>,
}

impl Context {
    pub fn new(
        comms: comms::AudioThreadComms,
        portmidi: portmidi::PortMidi,
    ) -> Context {
        let mut graph = dsp::Graph::new();
        let master_node = graph.add_node(dsp_node::DspNode::Master);
        graph.set_master(Some(master_node));

        Context {
            graph,
            selected_node: master_node,
            comms,
            events: Rc::new(RefCell::new(event::Buffer::new(portmidi))),
        }
    }
}
