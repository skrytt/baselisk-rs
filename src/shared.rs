
use parameter;
use modmatrix;
use std::sync::Arc;

pub struct SharedState {
    // Parameters is behind an Arc because in the VST case we need to
    // pass the parameters to the host thread.
    pub parameters: Arc<parameter::BaseliskPluginParameters>,
    pub modmatrix: modmatrix::ModulationMatrix,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            parameters: Arc::new(parameter::BaseliskPluginParameters::default()),
            modmatrix: modmatrix::ModulationMatrix::new(),
        }
    }
}
