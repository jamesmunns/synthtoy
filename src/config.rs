use serde::Deserialize;

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
    op_kind: OperatorKind
}

impl Operator {
    fn into_dyn_sample(&self) -> Box<dyn Sample> {
        match self.op_kind {
            OperatorKind::Sine => {
                let o = objects::SineWave { freq: self.frequency };
                Box::new(o)
            }
            OperatorKind::Square => {
                // TODO(AJM): `as usize`?
                let o = objects::SquareWave { freq: self.frequency as usize };
                Box::new(o)
            }
            OperatorKind::Saw => {
                let o = objects::SawWave { freq: self.frequency };
                Box::new(o)
            }
            OperatorKind::Triangle => todo!()
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

pub struct Bidness {
    pub b_groups: Vec<BGroup>,
}

use crate::objects::{
    Sample,
    self,
};

pub struct BGroup {
    pub source: Box<dyn Sample>,
    pub operators: Vec<Box<dyn Sample>>,
}

impl Bidness {
    pub fn from_config(cfg: &Config) -> Self {
        let mut ret = vec![];
        for g in cfg.group.iter() {
            let src = g.source.into_dyn_sample();
            let ops = g.operators.iter().map(Operator::into_dyn_sample).collect();
            ret.push(BGroup {
                source: src,
                operators: ops,
            });
        }
        Bidness {
            b_groups: ret
        }
    }
}
