use serde::Deserialize;
use std::collections::VecDeque;

#[derive(Debug, Deserialize)]
pub struct Config {
    group: Vec<Group>,
}

#[derive(Debug, Deserialize)]
pub struct Group {
    source: Operator,
    operators: Vec<Operator>,
}

#[derive(Debug, Deserialize)]
pub struct Operator {
    frequency: f32,
    op_kind: OperatorKind,

    #[serde(default)]
    phase_steps: usize,

    #[serde(default)]
    stepper: Stepper,
}

impl Operator {
    fn into_dyn_sample(&self) -> Box<dyn Sample> {
        match self.op_kind {
            OperatorKind::Sine => {
                let o = objects::SineWave {
                    freq: self.frequency,
                    stepper: self.stepper.clone(),
                    phase_steps: self.phase_steps,
                };
                Box::new(o)
            }
            OperatorKind::Square => {
                // TODO(AJM): `as usize`?
                let o = objects::SquareWave {
                    freq: self.frequency,
                    stepper: self.stepper.clone(),
                    phase_steps: self.phase_steps,
                };
                Box::new(o)
            }
            OperatorKind::Saw => {
                let o = objects::SawWave {
                    freq: self.frequency,
                    stepper: self.stepper.clone(),
                    phase_steps: self.phase_steps,
                };
                Box::new(o)
            }
            OperatorKind::Triangle => todo!(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub enum OperatorKind {
    Sine,
    Square,
    Saw,
    Triangle,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Stepper {
    pub bpm: usize,
    pub steps: VecDeque<StepKind>,
}

impl Default for Stepper {
    fn default() -> Self {
        Stepper {
            bpm: 1,
            steps: VecDeque::new(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "kind", content = "val")]
pub enum StepKind {
    Nop,
    FreqMult(f32),
    FreqSet(f32),
}

pub struct Bidness {
    pub b_groups: Vec<BGroup>,
}

use crate::objects::{self, Sample};

pub struct BGroup {
    pub samps_per_sec: usize,
    pub source: Box<dyn Sample>,
    pub operators: Vec<Box<dyn Sample>>,
}

impl Bidness {
    pub fn from_config(cfg: &Config, samps_per_sec: usize) -> Self {
        let mut ret = vec![];
        for g in cfg.group.iter() {
            let src = g.source.into_dyn_sample();
            let ops = g.operators.iter().map(Operator::into_dyn_sample).collect();
            ret.push(BGroup {
                samps_per_sec,
                source: src,
                operators: ops,
            });
        }
        Bidness { b_groups: ret }
    }
}
