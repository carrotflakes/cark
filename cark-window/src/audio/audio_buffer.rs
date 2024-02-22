use std::sync::Arc;

pub type AudioBufferRef = Arc<AudioBuffer>;

pub struct AudioBuffer {
    pub sample_rate: f32,
    pub buffer: Vec<[f32; 2]>,
}

impl AudioBuffer {
    pub fn new(sample_rate: f32, buffer: Vec<[f32; 2]>) -> Self {
        Self {
            sample_rate,
            buffer,
        }
    }
}
