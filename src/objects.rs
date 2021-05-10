use crate::config::{StepKind, Stepper};

pub struct SineWave {
    pub freq: f32,
    pub stepper: Stepper,
    pub phase_steps: usize,
}

impl Sample for SineWave {
    fn next(&mut self, num_samples: usize) -> f32 {
        let num_samples = num_samples.wrapping_add(self.phase_steps);
        if !self.stepper.steps.is_empty() {
            // TODO(AJM): this is just bad for a lot of reasons
            let samps_per_beat = (self.stepper.bpm * 48000) / 60;
            if num_samples % samps_per_beat == 0 {
                println!("step!");
                let step = self.stepper.steps.pop_front().unwrap();
                match step {
                    StepKind::Nop => {}
                    StepKind::FreqMult(fm) => {
                        self.freq *= fm;
                    }
                    StepKind::FreqSet(fs) => {
                        self.freq = fs;
                    }
                }
                self.stepper.steps.push_back(step);
            }
        }
        let value = 2.0 * core::f32::consts::PI * self.freq * num_samples as f32 / 48000.0;
        value.sin()
    }
}

pub struct SquareWave {
    pub freq: f32,
    pub stepper: Stepper,
    pub phase_steps: usize,
}

impl Sample for SquareWave {
    fn next(&mut self, num_samples: usize) -> f32 {
        let num_samples = num_samples.wrapping_add(self.phase_steps);
        if !self.stepper.steps.is_empty() {
            // TODO(AJM): this is just bad for a lot of reasons
            let samps_per_beat = (self.stepper.bpm * 48000) / 60;
            if num_samples % samps_per_beat == 0 {
                println!("step!");
                let step = self.stepper.steps.pop_front().unwrap();
                match step {
                    StepKind::Nop => {}
                    StepKind::FreqMult(fm) => {
                        self.freq *= fm;
                    }
                    StepKind::FreqSet(fs) => {
                        self.freq = fs;
                    }
                }
                self.stepper.steps.push_back(step);
            }
        }
        let div = num_samples / (24000 / (self.freq as usize));
        if (div % 2) == 0 {
            -1.0
        } else {
            1.0
        }
    }
}

pub struct SawWave {
    pub freq: f32,
    pub stepper: Stepper,
    pub phase_steps: usize,
}

impl Sample for SawWave {
    fn next(&mut self, num_samples: usize) -> f32 {
        let num_samples = num_samples.wrapping_add(self.phase_steps);
        if !self.stepper.steps.is_empty() {
            // TODO(AJM): this is just bad for a lot of reasons
            let samps_per_beat = (self.stepper.bpm * 48000) / 60;
            if num_samples % samps_per_beat == 0 {
                println!("step!");
                let step = self.stepper.steps.pop_front().unwrap();
                match step {
                    StepKind::Nop => {}
                    StepKind::FreqMult(fm) => {
                        self.freq *= fm;
                    }
                    StepKind::FreqSet(fs) => {
                        self.freq = fs;
                    }
                }
                self.stepper.steps.push_back(step);
            }
        }
        let samp_per = (48000.0 / self.freq) as usize;
        let idx = num_samples % samp_per;
        let norm = (idx as f32) / (samp_per as f32);
        (norm * 2.0) - norm
    }
}

pub trait Sample {
    fn next(&mut self, num_samples: usize) -> f32;
}
