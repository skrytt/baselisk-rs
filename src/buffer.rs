extern crate sample;

use sample::Frame;

/// A container for one or more buffers needed for processing,
/// and a means to manage its size.
pub struct ResizableFrameBuffers<F>
where
    F: Frame,
{
    per_buffer_capacity: usize,
    num_buffers: usize,
    data: Vec<F>,
}

impl<F> ResizableFrameBuffers<F>
where
    F: Frame,
{
    /// Create the buffers.
    /// The capacity defaults to 0, and should be set using self.resize().
    pub fn new(num_buffers: usize) -> ResizableFrameBuffers<F> {
        ResizableFrameBuffers {
            per_buffer_capacity: 0,
            num_buffers,
            data: Vec::with_capacity(0),
        }
    }

    /// Resize the buffers by setting a new capacity per buffer.
    pub fn resize(&mut self, per_buffer_capacity: usize) {
        self.data.resize(self.num_buffers * per_buffer_capacity, F::equilibrium());
        self.per_buffer_capacity = per_buffer_capacity;
    }

    /// get_mut: Returns a mutable reference to a buffer with the given number.
    /// Panics if the requested buffer is out of range.
    pub fn get_mut(&mut self, buffer_number: usize) -> &mut [F] {
        let start = buffer_number * self.per_buffer_capacity;
        let end = start + self.per_buffer_capacity;
        self.data.get_mut(start..end).unwrap()
    }
}
