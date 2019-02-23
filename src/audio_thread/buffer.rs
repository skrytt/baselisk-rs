
use defs;
use dsp::sample::frame;
use dsp::sample::Frame;

/// Buffer: a container for a buffer needed for processing,
/// and a means to manage its size.
pub struct Buffer {
    data: Vec<frame::Mono<defs::Output>>,
}

// If the frames per callback exceed this value,
// buffers will need to be reallocated, which is expensive
// and not desirable for realtime audio processing.
const BUFFER_DEFAULT_CAPACITY: usize = 4096;

fn resize_buffer<F>(buffer: &mut Vec<F>, frames: usize)
where
    F: Frame,
{
    let current_len = buffer.len();
    if current_len < frames {
        buffer.extend((current_len..frames).map(|_| F::equilibrium()));
    } else if current_len > frames {
        buffer.truncate(frames);
    }
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            data: Vec::with_capacity(BUFFER_DEFAULT_CAPACITY),
        }
    }

    /// Check the buffer matches the expected size.
    /// Otherwise, resize it.
    fn ensure_size(&mut self, frames: usize) {
        if self.data.len() != frames {
            resize_buffer(&mut self.data, frames)
        }
    }

    /// get_mut: Returns a mutable reference to a buffer of size expected_frames.
    /// The buffer will be resized if necessary.
    pub fn get_sized_mut(&mut self, expected_frames: usize) -> &mut Vec<frame::Mono<defs::Output>> {
        self.ensure_size(expected_frames);
        &mut self.data
    }
}
