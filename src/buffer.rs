extern crate sample;

use sample::Frame;

/// Buffer: a container for a buffer needed for processing,
/// and a means to manage its size.
pub struct ResizableFrameBuffer<F>
where
    F: Frame,
{
    data: Vec<F>,
}

// If the frames per callback exceed this value,
// buffers will need to be reallocated, which is expensive
// and not desirable for realtime audio processing.
const BUFFER_DEFAULT_CAPACITY: usize = 4096;

impl<F> ResizableFrameBuffer<F>
where
    F: Frame,
{
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(BUFFER_DEFAULT_CAPACITY),
        }
    }

    /// Check the buffer matches the expected size.
    /// Otherwise, resize it.
    fn ensure_size(&mut self, frames: usize) {
        if self.data.len() != frames {
            let current_len = self.data.len();
            if current_len < frames {
                self.data.extend((current_len..frames).map(|_| F::equilibrium()));
            } else if current_len > frames {
                self.data.truncate(frames);
            }
        }
    }

    /// get_mut: Returns a mutable reference to a buffer of size expected_frames.
    /// The buffer will be resized if necessary.
    pub fn get_sized_mut(&mut self, expected_frames: usize) -> &mut Vec<F> {
        self.ensure_size(expected_frames);
        &mut self.data
    }
}
