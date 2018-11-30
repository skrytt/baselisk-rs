use audio;
use defs;
use dsp;
use dsp_node;
use midi;
use std::cell::RefCell;
use std::sync::Arc;

type MidiInputBuffer = Arc<RefCell<midi::InputBuffer>>;
type Graph = Arc<RefCell<dsp::Graph<defs::Frame, dsp_node::DspNode<f32>>>>;

pub struct Context<'a> {
    pub midi_input_buffer: MidiInputBuffer,
    pub graph: Graph,
    pub master_node: dsp::NodeIndex,
    pub audio_interface: &'a mut audio::Interface,
}

impl<'a> Context<'a> {
    /// Run a closure while the audio stream is paused, passing
    /// a mutable reference to this Context as an argument.
    /// Afterwards, restore the original state of the audio stream.
    pub fn exec_while_paused<F>(&mut self, f: F)
    where
        F: Fn(&mut Context),
    {
        let was_active = self.audio_interface.is_active();
        if was_active {
            self.audio_interface.pause().unwrap();
        }

        // Give a temporary mutable borrow of this Context to the closure
        f(self);

        if was_active {
            self.audio_interface.resume().unwrap();
        }
    }
}
