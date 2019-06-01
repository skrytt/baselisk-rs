
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
        let parameters = Arc::new(parameter::BaseliskPluginParameters::default());
        let parameters_clone = Arc::clone(&parameters);
        Self {
            parameters,
            modmatrix: modmatrix::ModulationMatrix::new(parameters_clone),
        }
    }
}
