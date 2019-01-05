use event;
use std::sync::mpsc;

/// MainThreadComms contains the pair of mpsc Sender and Receiver instances
/// used by the main thread to talk to the audio thread.
/// This is used as a client: to request changes to the state, then check the responses.
pub struct MainThreadComms {
    pub tx: mpsc::Sender<event::Event>,
    pub rx: mpsc::Receiver<Result<(), String>>,
}

/// AudioThreadComms contains the pair of mpsc Sender and Receiver instances
/// used by the audio thread to talk to the main thread.
/// This is used as a server: to receive state change requests, then respond to indicate
/// whether they were successful.
pub struct AudioThreadComms {
    pub rx: mpsc::Receiver<event::Event>,
    pub tx: mpsc::Sender<Result<(), String>>,
}

/// Create two MPSC channels (for bidirectional communication).
/// structure and return the channels for each thread separately.
pub fn new_bidirectional() -> (MainThreadComms, AudioThreadComms) {
    let (tx_thread1, rx_thread2) = mpsc::channel();
    let (tx_thread2, rx_thread1) = mpsc::channel();

    (
        MainThreadComms {
            rx: rx_thread1,
            tx: tx_thread1,
        },
        AudioThreadComms {
            rx: rx_thread2,
            tx: tx_thread2,
        },
    )
}
