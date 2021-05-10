pub struct SineWave {
    pub freq: f32,
}

impl Sample for SineWave {
    fn next(&mut self, num_samples: usize) -> f32 {
        let value = 2.0 * core::f32::consts::PI * self.freq * num_samples as f32 / 48000.0;
        value.sin()
    }
}

pub struct SquareWave {
    pub freq: usize,
}

impl Sample for SquareWave {
    fn next(&mut self, num_samples: usize) -> f32 {
        let div = num_samples / (24000 / self.freq);
        if (div % 2) == 0 {
            -1.0
        } else {
            1.0
        }
    }
}

pub struct SawWave {
    pub freq: f32,
}

impl Sample for SawWave {
    fn next(&mut self, num_samples: usize) -> f32 {
        let samp_per = (48000.0 / self.freq) as usize;
        let idx = num_samples % samp_per;
        let norm = (idx as f32) / (samp_per as f32);
        (norm * 2.0) - norm
    }
}

pub trait Sample {
    fn next(&mut self, num_samples: usize) -> f32;
}

// 1hz

// 00000-24000 off
// 24000-48000 on

// 2hz

// 00000-12000 off
// 12000-24000 on
