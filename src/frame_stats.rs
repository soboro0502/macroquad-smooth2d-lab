use crate::config::{HUD_RING_SIZE, HUD_SAMPLE_SECONDS};

pub struct FrameStats {
    samples: [f32; HUD_RING_SIZE],
    next: usize,
    len: usize,
    publish_timer: f32,
    pub snapshot: FrameStatsSnapshot,
}

pub struct RunFrameStats {
    samples: Vec<f32>,
}

pub struct RunValueStats {
    samples: Vec<f32>,
}

#[derive(Clone, Copy, Default)]
pub struct FrameStatsSnapshot {
    pub fps: i32,
    pub avg_fps: f32,
    pub last_ms: f32,
    pub avg_ms: f32,
    pub min_ms: f32,
    pub max_ms: f32,
    pub p95_ms: f32,
    pub p99_ms: f32,
    pub stdev_ms: f32,
    pub slow_percent: f32,
    pub spike_count: usize,
}

#[derive(Clone, Copy, Default)]
pub struct ValueStatsSnapshot {
    pub last: f32,
    pub avg: f32,
    pub min: f32,
    pub max: f32,
    pub stdev: f32,
}

impl RunFrameStats {
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: Vec::with_capacity(capacity),
        }
    }

    pub fn reset(&mut self) {
        self.samples.clear();
    }

    pub fn record(&mut self, dt: f32) {
        self.samples.push(dt);
    }

    pub fn snapshot(&self, fps: i32, target_dt: f32) -> FrameStatsSnapshot {
        build_snapshot(&self.samples, fps, target_dt)
    }
}

impl RunValueStats {
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: Vec::with_capacity(capacity),
        }
    }

    pub fn reset(&mut self) {
        self.samples.clear();
    }

    pub fn record(&mut self, value: f32) {
        self.samples.push(value);
    }

    pub fn snapshot(&self) -> ValueStatsSnapshot {
        build_value_snapshot(&self.samples)
    }
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

    pub fn reset(&mut self) {
        *self = Self::new();
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
        build_snapshot(&self.samples[..self.len], fps, target_dt)
    }
}

fn build_value_snapshot(samples: &[f32]) -> ValueStatsSnapshot {
    if samples.is_empty() {
        return ValueStatsSnapshot::default();
    }

    let sum: f32 = samples.iter().sum();
    let avg = sum / samples.len() as f32;
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
        / samples.len() as f32;

    ValueStatsSnapshot {
        last: samples[samples.len() - 1],
        avg,
        min,
        max,
        stdev: variance.sqrt(),
    }
}

fn build_snapshot(samples: &[f32], fps: i32, target_dt: f32) -> FrameStatsSnapshot {
    if samples.is_empty() {
        return FrameStatsSnapshot::default();
    }

    let sum: f32 = samples.iter().sum();
    let avg = sum / samples.len() as f32;
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
        / samples.len() as f32;
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
    let mut sorted_samples = samples.to_vec();
    sorted_samples.sort_by(|a, b| a.total_cmp(b));
    let p95 = percentile(&sorted_samples, 0.95);
    let p99 = percentile(&sorted_samples, 0.99);

    FrameStatsSnapshot {
        fps,
        avg_fps: 1.0 / avg.max(f32::EPSILON),
        last_ms: samples[samples.len() - 1] * 1000.0,
        avg_ms: avg * 1000.0,
        min_ms: min * 1000.0,
        max_ms: max * 1000.0,
        p95_ms: p95 * 1000.0,
        p99_ms: p99 * 1000.0,
        stdev_ms: variance.sqrt() * 1000.0,
        slow_percent: slow_count as f32 / samples.len() as f32 * 100.0,
        spike_count,
    }
}

fn percentile(sorted_samples: &[f32], percentile: f32) -> f32 {
    if sorted_samples.is_empty() {
        return 0.0;
    }

    let index = ((sorted_samples.len() - 1) as f32 * percentile)
        .round()
        .clamp(0.0, (sorted_samples.len() - 1) as f32) as usize;
    sorted_samples[index]
}
