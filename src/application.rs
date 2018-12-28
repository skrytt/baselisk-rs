use comms;
use defs;
use dsp;
use dsp_node;
use event;
use std::sync::{Arc, RwLock};

pub type Graph = dsp::Graph<defs::Frame, dsp_node::DspNode<f32>>;

pub struct AudioThreadContext {
    pub graph: Graph,
    pub selected_node: dsp::NodeIndex,
    pub comms: comms::AudioThreadComms,
    // Events is refcounted because audio nodes also need to hold references to it
    pub events: Arc<RwLock<event::Buffer>>,
}
