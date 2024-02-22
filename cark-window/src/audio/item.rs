use super::audio_buffer::AudioBufferRef;

pub struct AudioItem {
    volume: f32,
    pan: f32,
    pitch: f32,
    buffer: AudioBufferRef,
    repeat: bool,
    pointer: f32,
}

impl AudioItem {
    pub fn new_bgm(buffer: AudioBufferRef) -> Self {
        Self {
            volume: 1.0,
            pan: 0.0,
            pitch: 1.0,
            buffer,
            repeat: true,
            pointer: 0.0,
        }
    }

    pub fn new_se(buffer: AudioBufferRef) -> Self {
        Self {
            volume: 1.0,
            pan: 0.0,
            pitch: 1.0,
            buffer,
            repeat: false,
            pointer: 0.0,
        }
    }

    pub fn volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    pub fn pan(mut self, pan: f32) -> Self {
        self.pan = pan;
        self
    }

    pub fn pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch;
        self
    }

    pub fn is_end(&self) -> bool {
        !self.repeat && self.pointer >= self.buffer.buffer.len() as f32
    }

    // data: [L, R, L, R, ...]
    pub fn add_to_buffer(&mut self, sample_rate: f32, data: &mut [f32]) {
        let ab = &self.buffer;
        let len = data.len() / 2;
        let step = self.pitch * ab.sample_rate / sample_rate;
        let panner = panner(self.pan);
        if self.repeat {
            for i in 0..len {
                let p = self.pointer + i as f32 * step;
                let p_usize = p as usize % ab.buffer.len();
                let l = ab.buffer[p_usize][0]
                    + (ab.buffer[(p_usize + 1) % ab.buffer.len()][0] - ab.buffer[p_usize][0])
                        * p.fract();
                let r = ab.buffer[p_usize][1]
                    + (ab.buffer[(p_usize + 1) % ab.buffer.len()][1] - ab.buffer[p_usize][1])
                        * p.fract();
                let [l, r] = panner([l, r]);
                data[i * 2] += l * self.volume;
                data[i * 2 + 1] += r * self.volume;
            }
            self.pointer = (self.pointer + len as f32 * step) % ab.buffer.len() as f32;
        } else {
            for i in 0..len {
                let p = self.pointer + i as f32 * step;
                let p_usize = p as usize;
                if p_usize >= ab.buffer.len() - 1 {
                    self.pointer = ab.buffer.len() as f32;
                    return;
                }
                let l = ab.buffer[p_usize][0]
                    + (ab.buffer[p_usize + 1][0] - ab.buffer[p_usize][0]) * p.fract();
                let r = ab.buffer[p_usize][1]
                    + (ab.buffer[p_usize + 1][1] - ab.buffer[p_usize][1]) * p.fract();
                let [l, r] = panner([l, r]);
                data[i * 2] += l * self.volume;
                data[i * 2 + 1] += r * self.volume;
            }
            self.pointer += len as f32 * step;
        }
    }
}

fn panner(pan: f32) -> impl Fn([f32; 2]) -> [f32; 2] {
    let pan = pan.clamp(-1.0, 1.0);
    let [ll, lr, rl, rr] = if pan <= 0.0 {
        let a = (pan + 1.0) * std::f32::consts::FRAC_PI_2;
        let (gain_r, gain_l) = a.sin_cos();
        [1.0, gain_l, gain_r, 0.0]
    } else {
        let a = pan * std::f32::consts::FRAC_PI_2;
        let (gain_l, gain_r) = a.sin_cos();
        [gain_l, 0.0, gain_r, 1.0]
    };
    move |x: [f32; 2]| [x[0] * ll + x[1] * lr, x[0] * rl + x[1] * rr]
}
