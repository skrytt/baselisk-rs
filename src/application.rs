use defs;
use dsp;
use dsp_node;
use midi;
use std::cell::RefCell;
use std::sync::Arc;

pub type MidiInputBuffer = Arc<RefCell<midi::InputBuffer>>;
pub type Graph = Arc<RefCell<dsp::Graph<defs::Frame, dsp_node::DspNode<f32>>>>;

pub struct Context {
    pub midi_input_buffer: MidiInputBuffer,
    pub graph: Graph,
    pub master_node: dsp::NodeIndex,
}
