use defs;
use dsp;
use dsp_node;

pub type Graph = dsp::Graph<defs::Frame, dsp_node::DspNode<f32>>;

pub struct Context {
    pub graph: Graph,
    pub selected_node: dsp::NodeIndex,
}
