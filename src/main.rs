use bbqueue::{consts::*, BBBuffer, ConstBBBuffer, Consumer};
use rodio::play_raw;
use rodio::Source;
use std::thread::{sleep, yield_now};
use std::time::{Duration, Instant};

static BB: BBBuffer<U16384> = BBBuffer(ConstBBBuffer::new());

pub mod config;
pub mod objects;

use toml;

fn main() {
    let config_str = std::fs::read_to_string("op.toml").unwrap();
    let config: config::Config = toml::from_str(&config_str).unwrap();
    let mut ops = config::Bidness::from_config(&config, 48_000);
    println!("{:?}", config);

    let (mut prod, cons) = BB.try_split().unwrap();
    let device = rodio::default_output_device().unwrap();
    let mut num_samples = 0usize;

    let src = BBSource::new(cons);
    play_raw(&device, src);

    loop {
        match prod.grant_max_remaining(10000000) {
            Ok(mut wgr) if wgr.len() >= 4 => {
                let mut rel = 0;
                for ch in wgr.chunks_exact_mut(4) {
                    // modulate each
                    let mut samps = vec![];
                    for g in ops.b_groups.iter_mut() {
                        let mut src = g.source.next(num_samples);
                        g.operators
                            .iter_mut()
                            .map(|op| op.next(num_samples))
                            .for_each(|samp| {
                                src *= samp;
                            });
                        samps.push(src);
                    }

                    // combine
                    let len = samps.len();
                    let samp: f32 = samps.into_iter().sum::<f32>() / (len as f32);

                    num_samples = num_samples.wrapping_add(1);
                    ch.copy_from_slice(&samp.to_le_bytes());
                    rel += 4;
                }

                wgr.commit(rel);
            }
            _ => {
                sleep(Duration::from_millis(10));
                yield_now();
            }
        }
    }
    // Do something!
}

pub struct BBSource {
    cons: Consumer<'static, U16384>,
    start: Instant,
}

impl BBSource {
    pub fn new(cons: Consumer<'static, U16384>) -> Self {
        Self {
            cons,
            start: Instant::now(),
        }
    }
}

impl Iterator for BBSource {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        // TODO: Should we block on missing samples?
        // If you starve it for too long, you get:
        // `ALSA lib pcm.c:8526:(snd_pcm_recover) underrun occurred`

        match self.cons.read() {
            Ok(rgr) if rgr.len() >= 4 => {
                let mut slice = [0u8; 4];
                slice.copy_from_slice(&rgr[..4]);
                let sample = f32::from_le_bytes(slice);
                rgr.release(4);
                Some(sample)
            }

            // TODO: Is it acceptable to just zero-fill on missing samples?
            // Throw up some kind of underrun warning?
            _ => {
                let elapsed = self.start.elapsed();

                if elapsed >= Duration::from_millis(100) {
                    println!("BRAP {:?}", elapsed);
                }
                Some(0.0)
            }
        }
    }
}

impl Source for BBSource {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    #[inline]
    fn channels(&self) -> u16 {
        1
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        48000
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
