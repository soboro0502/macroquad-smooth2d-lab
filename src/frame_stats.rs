use crate::config::{HUD_RING_SIZE, HUD_SAMPLE_SECONDS};

pub struct FrameStats {
    samples: [f32; HUD_RING_SIZE],
    next: usize,
    len: usize,
    publish_timer: f32,
    pub snapshot: FrameStatsSnapshot,
}

#[derive(Clone, Copy, Default)]
pub struct FrameStatsSnapshot {
    pub fps: i32,
    pub avg_ms: f32,
    pub min_ms: f32,
    pub max_ms: f32,
    pub stdev_ms: f32,
    pub slow_percent: f32,
    pub spike_count: usize,
}

impl FrameStats {
    pub fn new() -> Self {
        Self {
            samples: [0.0; HUD_RING_SIZE],
            next: 0,
            len: 0,
            publish_timer: 0.0,
            snapshot: FrameStatsSnapshot::default(),
        }
    }

    pub fn record(&mut self, dt: f32, fps: i32, target_dt: f32) {
        self.samples[self.next] = dt;
        self.next = (self.next + 1) % self.samples.len();
        self.len = (self.len + 1).min(self.samples.len());
        self.publish_timer += dt;

        if self.publish_timer >= HUD_SAMPLE_SECONDS {
            self.publish_timer = 0.0;
            self.snapshot = self.build_snapshot(fps, target_dt);
        }
    }

    fn build_snapshot(&self, fps: i32, target_dt: f32) -> FrameStatsSnapshot {
        if self.len == 0 {
            return FrameStatsSnapshot::default();
        }

        let samples = &self.samples[..self.len];
        let sum: f32 = samples.iter().sum();
        let avg = sum / self.len as f32;
        let min = samples
            .iter()
            .fold(f32::INFINITY, |acc, sample| acc.min(*sample));
        let max = samples.iter().fold(0.0_f32, |acc, sample| acc.max(*sample));
        let variance = samples
            .iter()
            .map(|sample| {
                let delta = sample - avg;
                delta * delta
            })
            .sum::<f32>()
            / self.len as f32;
        let slow_limit = target_dt * 1.25;
        let spike_limit = target_dt * 2.0;
        let slow_count = samples
            .iter()
            .filter(|sample| **sample > slow_limit)
            .count();
        let spike_count = samples
            .iter()
            .filter(|sample| **sample > spike_limit)
            .count();

        FrameStatsSnapshot {
            fps,
            avg_ms: avg * 1000.0,
            min_ms: min * 1000.0,
            max_ms: max * 1000.0,
            stdev_ms: variance.sqrt() * 1000.0,
            slow_percent: slow_count as f32 / self.len as f32 * 100.0,
            spike_count,
        }
    }
}
