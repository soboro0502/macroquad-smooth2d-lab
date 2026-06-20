use macroquad::prelude::get_time;

use crate::config::FRAME_LOG_INTERVAL_SECONDS;
use crate::frame_stats::FrameStatsSnapshot;

pub struct FrameLog {
    enabled: bool,
    started_at: f64,
    next_summary_at: f64,
}

impl FrameLog {
    pub fn new(enabled: bool) -> Self {
        let now = get_time();
        Self {
            enabled,
            started_at: now,
            next_summary_at: now + FRAME_LOG_INTERVAL_SECONDS,
        }
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
        eprintln!(
            "[frame-log] {}",
            if self.enabled { "enabled" } else { "disabled" }
        );
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn reset_clock(&mut self) {
        let now = get_time();
        self.started_at = now;
        self.next_summary_at = now + FRAME_LOG_INTERVAL_SECONDS;
    }

    pub fn event(
        &self,
        label: &'static str,
        dt: f32,
        fps: i32,
        clear_only: bool,
        manual_pacer_enabled: bool,
    ) {
        if !self.enabled {
            return;
        }

        eprintln!(
            "[frame-event t={:.3}s] kind={} dt_ms={:.3} fps={} load={} pace={}",
            get_time() - self.started_at,
            label,
            dt * 1000.0,
            fps,
            if clear_only { "CLEAR" } else { "FULL" },
            if manual_pacer_enabled {
                "MANUAL"
            } else {
                "AUTO"
            },
        );
    }

    pub fn summary(&mut self, snapshot: FrameStatsSnapshot, cpu_percent: f32) {
        if !self.enabled {
            return;
        }

        let now = get_time();
        if now < self.next_summary_at {
            return;
        }

        self.next_summary_at = now + FRAME_LOG_INTERVAL_SECONDS;
        eprintln!(
            "[frame-summary t={:.3}s] fps={} avg_fps={:.1} last_ms={:.3} avg_ms={:.3} p95_ms={:.3} p99_ms={:.3} range_ms={:.3}-{:.3} sd_ms={:.3} slow_pct={:.1} spikes={} cpu={:.1}%",
            now - self.started_at,
            snapshot.fps,
            snapshot.avg_fps,
            snapshot.last_ms,
            snapshot.avg_ms,
            snapshot.p95_ms,
            snapshot.p99_ms,
            snapshot.min_ms,
            snapshot.max_ms,
            snapshot.stdev_ms,
            snapshot.slow_percent,
            snapshot.spike_count,
            cpu_percent,
        );
    }

    pub fn final_summary(&self, snapshot: FrameStatsSnapshot, cpu_percent: f32) {
        if !self.enabled {
            return;
        }

        eprintln!(
            "[frame-final t={:.3}s] fps={} avg_fps={:.1} last_ms={:.3} avg_ms={:.3} p95_ms={:.3} p99_ms={:.3} range_ms={:.3}-{:.3} sd_ms={:.3} slow_pct={:.1} spikes={} cpu={:.1}%",
            get_time() - self.started_at,
            snapshot.fps,
            snapshot.avg_fps,
            snapshot.last_ms,
            snapshot.avg_ms,
            snapshot.p95_ms,
            snapshot.p99_ms,
            snapshot.min_ms,
            snapshot.max_ms,
            snapshot.stdev_ms,
            snapshot.slow_percent,
            snapshot.spike_count,
            cpu_percent,
        );
    }
}
