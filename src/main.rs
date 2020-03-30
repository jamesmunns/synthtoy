use rodio::{play_raw};
use std::time::{Duration, Instant};
use std::thread::{sleep, yield_now};
use rodio::Source;
use bbqueue::{consts::*, BBBuffer, ConstBBBuffer, Consumer};

static BB: BBBuffer<U16384> = BBBuffer( ConstBBBuffer::new() );

pub mod objects;

fn main() {
    let (mut prod, cons) = BB.try_split().unwrap();
    let device = rodio::default_output_device().unwrap();
    let mut num_samples = 0usize;

    let src = BBSource::new(cons);
    play_raw(&device, src);

    let mut out_sq = objects::SquareWave { freq: 220 };
    let mut out_si = objects::SineWave { freq: 4.0 };
    let mut out_si2 = objects::SineWave { freq: 2.0 };

    let mut out2_sq = objects::SquareWave { freq: 440 };
    let mut out2_si = objects::SineWave { freq: 8.0 };
    let mut out2_si2 = objects::SawWave { freq: 0.5 };

    let mut out3_sq = objects::SawWave { freq: 110.0 };
    let mut out3_si = objects::SineWave { freq: 1.0 };
    let mut out3_si2 = objects::SineWave { freq: 0.25 };

    loop {
        match prod.grant_max_remaining(10000000) {
            Ok(mut wgr) if wgr.len() >= 4 => {
                let mut rel = 0;
                for ch in wgr.chunks_exact_mut(4) {
                    let smpl_si = out_si.next(num_samples);
                    let smpl_sq = out_sq.next(num_samples);
                    let smpl_si2 = out_si2.next(num_samples);

                    // Modulate
                    let smpl = smpl_si * smpl_sq * smpl_si2;

                    let smpl2_si = out2_si.next(num_samples);
                    let smpl2_sq = out2_sq.next(num_samples);
                    let smpl2_si2 = out2_si2.next(num_samples + 12000);

                    // Modulate
                    let smpl2 = smpl2_si * smpl2_sq * smpl2_si2;

                    let smpl3_si = out3_si.next(num_samples);
                    let smpl3_sq = out3_sq.next(num_samples);
                    let smpl3_si2 = out3_si2.next(num_samples);

                    // Modulate
                    let smpl3 = smpl3_si * smpl3_sq * smpl3_si2;

                    // combine
                    let smpl = (smpl + smpl2 + smpl3) / 3.0;

                    // let smpl = smpl3_sq;

                    num_samples = num_samples.wrapping_add(1);
                    ch.copy_from_slice(&smpl.to_le_bytes());
                    rel += 4;

                    if num_samples % (48000 * 2) == 0 {
                        out_si.freq *= 1.5;
                    }
                    if num_samples % (48000 * 16) == 0 {
                        out_si.freq = 4.0;
                    }

                    match (num_samples / 48000) % 4 {
                        0 => {
                            out3_sq.freq = 110.0;
                        }
                        1 => {

                        }
                        2 => {
                            out3_sq.freq = 220.0;
                        }
                        3 => {
                            out3_sq.freq = 440.0;
                        }
                        _ => unreachable!(),
                    }
                }

                wgr.commit(rel);
            }
            _ => {
                sleep(Duration::from_millis(10));
                yield_now();
            },
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
        Self { cons, start: Instant::now() }
    }
}

impl Iterator for BBSource {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        // TODO: Should we block on missing samples?
        // If you starve it for too long, you get:
        // `ALSA lib pcm.c:8526:(snd_pcm_recover) underrun occurred`
        //
        // let mut latch = false;
        // loop {
        //     match (self.cons.read(), &mut latch) {
        //         (Ok(rgr), _) if rgr.len() >= 4 => {
        //             let mut slice = [0u8; 4];
        //             slice.copy_from_slice(&rgr[..4]);
        //             let sample = f32::from_le_bytes(slice);
        //             rgr.release(4);
        //             break Some(sample);
        //         }
        //         (_, l @ false) => {
        //             *l = true;
        //             println!("SKIP - {:?}", self.start.elapsed());
        //         }
        //         _ => {},
        //     }
        //     yield_now();
        // }

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

/// An infinite source that produces a sine.
///
/// Always has a rate of 48kHz and one channel.
#[derive(Clone, Debug)]
pub struct SineWave {
    freq: f32,
    num_sample: usize,
    start: Instant,
    last: Instant,
}

impl SineWave {
    /// The frequency of the sine.
    #[inline]
    pub fn new(freq: u32) -> SineWave {
        SineWave {
            freq: freq as f32,
            num_sample: 0,
            start: Instant::now(),
            last: Instant::now(),
        }
    }
}

impl Iterator for SineWave {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        if self.num_sample == 0 {
            self.start = Instant::now();
        }

        let theo_duration = (Duration::from_secs(1) / 48000) * self.num_sample as u32;

        // let interval = self.last.elapsed();
        self.last = Instant::now();

        let elapsed = self.start.elapsed();

        let rpt = if elapsed > theo_duration {
            // Elapsed is MORE than the theoretical duration
            // We're behind schedule
            format!("BEHIND {:?}", elapsed - theo_duration)
        } else {
            // Elapsed is LESS than the theoretical duration
            // We're ahead of schedule
            format!("AHEAD {:?}", theo_duration - elapsed)
        };

        // println!("{} - {} - {:?}", self.freq, rpt, self.start.elapsed());
        self.num_sample = self.num_sample.wrapping_add(1);

        if self.num_sample >= 12000 {
            return None;
        }

        let value = 2.0 * 3.14159265 * self.freq * self.num_sample as f32 / 48000.0;
        Some(value.sin())
    }
}

impl Source for SineWave {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        Some(128)
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
