use defs;
use dsp;
use dsp_node;
use event;
use std::sync::{Arc, RwLock};

pub type EventBuffer = Arc<RwLock<event::Buffer>>;
pub type Graph = Arc<RwLock<dsp::Graph<defs::Frame, dsp_node::DspNode<f32>>>>;

pub struct Context {
    pub event_buffer: EventBuffer,
    pub graph: Graph,
    pub master_node: dsp::NodeIndex,
}
