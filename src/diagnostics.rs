use macroquad::prelude::*;

use crate::app_options::PacerMode;
use crate::config::*;
use crate::frame_log::FrameLog;
use crate::frame_stats::{FrameStats, FrameStatsSnapshot};
use crate::game::{Assets, BackgroundMode, TimingMode};

pub fn fps_from_dt(dt: f32) -> i32 {
    (1.0 / dt.max(f32::EPSILON)) as i32
}

pub fn diagnostic_verdict(snapshot: FrameStatsSnapshot) -> &'static str {
    if snapshot.avg_ms <= 0.0 {
        return "WAIT";
    }

    if snapshot.max_ms <= DIAG_PASS_MAX_MS
        && snapshot.p99_ms <= DIAG_PASS_P99_MS
        && snapshot.stdev_ms <= DIAG_PASS_STDEV_MS
        && snapshot.slow_percent <= DIAG_PASS_SLOW_PERCENT
        && snapshot.spike_count == DIAG_PASS_SPIKES
    {
        "PASS"
    } else {
        "WARN"
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum FrameMarker {
    None,
    Slow,
    Fast,
}

pub fn frame_marker(dt: f32) -> FrameMarker {
    if dt >= 1.0 / FRAME_SPIKE_HZ {
        FrameMarker::Slow
    } else if dt <= 1.0 / FRAME_FAST_HZ {
        FrameMarker::Fast
    } else {
        FrameMarker::None
    }
}

pub fn log_frame_marker(
    frame_log: &FrameLog,
    frame_marker: FrameMarker,
    dt: f32,
    fps: i32,
    clear_only: bool,
    manual_pacer_enabled: bool,
) {
    match frame_marker {
        FrameMarker::None => {}
        FrameMarker::Slow => {
            frame_log.event("slow", dt, fps, clear_only, manual_pacer_enabled);
        }
        FrameMarker::Fast => {
            frame_log.event("fast", dt, fps, clear_only, manual_pacer_enabled);
        }
    }
}

pub fn draw_frame_marker(frame_marker: FrameMarker) {
    let (x, color) = match frame_marker {
        FrameMarker::None => return,
        FrameMarker::Slow => (FRAME_MARKER_MARGIN, Color::new(1.0, 0.08, 0.08, 0.9)),
        FrameMarker::Fast => (
            FRAME_MARKER_MARGIN + FRAME_SPIKE_MARKER_SIZE + FRAME_MARKER_GAP,
            Color::new(0.08, 0.45, 1.0, 0.9),
        ),
    };
    let y = screen_height() - FRAME_MARKER_MARGIN - FRAME_SPIKE_MARKER_SIZE;

    draw_rectangle(
        x,
        y,
        FRAME_SPIKE_MARKER_SIZE,
        FRAME_SPIKE_MARKER_SIZE,
        color,
    );
}

pub struct HudState {
    pub scroll_enabled: bool,
    pub timing_mode: TimingMode,
    pub background_mode: BackgroundMode,
    pub background_frame_step: f32,
    pub background_last_delta: f32,
    pub clear_only: bool,
    pub manual_pacer_enabled: bool,
    pub pacer_mode: PacerMode,
    pub pacer_sleep_margin_secs: f64,
    pub pacer_sleep_threshold_secs: f64,
    pub cpu_percent: f32,
    pub frame_log_enabled: bool,
}

pub fn draw_hud(assets: &Assets, stats: &FrameStats, state: HudState) {
    let snapshot = stats.snapshot;
    let scroll = if state.scroll_enabled { "ON" } else { "OFF" };
    let load = if state.clear_only { "CLEAR" } else { "FULL" };
    let pace = if state.manual_pacer_enabled {
        state.pacer_mode.label()
    } else {
        "AUTO"
    };
    let log = if state.frame_log_enabled { "ON" } else { "OFF" };
    let quality = diagnostic_verdict(snapshot);
    let text = format!(
        "Q {}  LOAD {}  PACE {} M {:.2} T {:.2}  LOG {}  MODE {}  DRAW {}  BGSTEP {:.0}px BGD {:>5.2}  CPU {:>5.1}%  fps {:>3}/{:>5.1}  ms last {:>5.2} avg {:>5.2} p95 {:>5.2} p99 {:>5.2} range {:>5.2}-{:>5.2} sd {:>4.2} slow {:>4.1}% spk {:>2} BG {}",
        quality,
        load,
        pace,
        state.pacer_sleep_margin_secs * 1000.0,
        state.pacer_sleep_threshold_secs * 1000.0,
        log,
        state.timing_mode.label(),
        state.background_mode.label(),
        state.background_frame_step,
        state.background_last_delta,
        state.cpu_percent,
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
        scroll
    );

    draw_rectangle(
        8.0,
        8.0,
        screen_width() - 16.0,
        30.0,
        Color::new(0.0, 0.0, 0.0, 0.55),
    );
    draw_text_ex(
        &text,
        14.0,
        30.0,
        TextParams {
            font: Some(&assets.font),
            font_size: HUD_FONT_SIZE,
            color: WHITE,
            ..Default::default()
        },
    );
}
