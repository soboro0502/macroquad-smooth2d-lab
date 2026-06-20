use macroquad::prelude::*;

use crate::app_options::PacerMode;
use crate::config::*;
use crate::frame_log::FrameLog;
use crate::frame_pacer::PacerSample;
use crate::frame_stats::{FrameStats, FrameStatsSnapshot};
use crate::game::{Assets, BackgroundMode, TimingMode};

pub fn fps_from_dt(dt: f32) -> i32 {
    (1.0 / dt.max(f32::EPSILON)) as i32
}

pub fn diagnostic_verdict(snapshot: FrameStatsSnapshot, target_dt: f32) -> &'static str {
    if snapshot.avg_ms <= 0.0 {
        return "WAIT";
    }

    let target_ms = target_dt * 1000.0;
    if snapshot.max_ms <= target_ms + DIAG_PASS_MAX_OVER_TARGET_MS
        && snapshot.p99_ms <= target_ms + DIAG_PASS_P99_OVER_TARGET_MS
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

pub fn frame_marker(dt: f32, target_dt: f32) -> FrameMarker {
    if dt >= target_dt * FRAME_SLOW_MARKER_FACTOR {
        FrameMarker::Slow
    } else if dt <= target_dt * FRAME_FAST_MARKER_FACTOR {
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

#[derive(Clone, Copy)]
pub struct HudState {
    pub scroll_enabled: bool,
    pub timing_mode: TimingMode,
    pub background_mode: BackgroundMode,
    pub background_frame_step: f32,
    pub background_last_delta: f32,
    pub clear_only: bool,
    pub manual_pacer_enabled: bool,
    pub pacer_mode: PacerMode,
    pub pacer_spin_margin_secs: f64,
    pub pacer_sleep_margin_secs: f64,
    pub pacer_sleep_threshold_secs: f64,
    pub pacer_sample: PacerSample,
    pub next_frame_wait_secs: f64,
    pub target_refresh_hz: u32,
    pub cpu_percent: f32,
    pub frame_log_enabled: bool,
}

pub struct HudTextCache {
    lines: [String; HUD_LINE_COUNT],
    refresh_timer: f32,
}

impl HudTextCache {
    pub fn new() -> Self {
        Self {
            lines: std::array::from_fn(|_| String::new()),
            refresh_timer: HUD_SAMPLE_SECONDS,
        }
    }

    fn update(&mut self, stats: &FrameStats, state: HudState) {
        self.refresh_timer = 0.0;
        let snapshot = stats.snapshot;
        let scroll = if state.scroll_enabled { "ON" } else { "OFF" };
        let load = if state.clear_only { "CLEAR" } else { "FULL" };
        let pace = if state.manual_pacer_enabled {
            state.pacer_mode.label()
        } else {
            "AUTO"
        };
        let margin_secs = match state.pacer_mode {
            PacerMode::SleepSpin => state.pacer_sleep_margin_secs,
            PacerMode::MachSpin | PacerMode::Spin => state.pacer_spin_margin_secs,
        };
        let log = if state.frame_log_enabled { "ON" } else { "OFF" };
        let quality = diagnostic_verdict(snapshot, 1.0 / state.target_refresh_hz as f32);

        self.lines[0] = format!(
            "STATUS Q {} | LOAD {} | PACE {} S {:.2} T {:.2} | CPU {:>5.1}% | LOG {}",
            quality,
            load,
            pace,
            margin_secs * 1000.0,
            state.pacer_sleep_threshold_secs * 1000.0,
            state.cpu_percent,
            log,
        );
        self.lines[1] = format!(
            "SCENE  MODE {} | DRAW {} | BG {} | STEP {:.0}px | BGD {:>5.2}",
            state.timing_mode.label(),
            state.background_mode.label(),
            scroll,
            state.background_frame_step,
            state.background_last_delta,
        );
        self.lines[2] = format!(
            "SYNC   next {:>5.2}ms | os {:>5.2}ms | spin {:>5.2}ms | total {:>5.2}ms",
            state.next_frame_wait_secs * 1000.0,
            state.pacer_sample.os_wait_ms(),
            state.pacer_sample.spin_ms(),
            state.pacer_sample.total_wait_ms(),
        );
        self.lines[3] = format!(
            "FRAME  hz {:>3} | fps {:>3}/{:>5.1} | ms last {:>5.2} avg {:>5.2} | p95 {:>5.2} p99 {:>5.2}",
            state.target_refresh_hz,
            snapshot.fps,
            snapshot.avg_fps,
            snapshot.last_ms,
            snapshot.avg_ms,
            snapshot.p95_ms,
            snapshot.p99_ms,
        );
        self.lines[4] = format!(
            "STABLE range {:>5.2}-{:>5.2} | sd {:>4.2} | slow {:>4.1}% | spk {:>2}",
            snapshot.min_ms,
            snapshot.max_ms,
            snapshot.stdev_ms,
            snapshot.slow_percent,
            snapshot.spike_count,
        );
    }
}

const HUD_LINE_COUNT: usize = 5;

pub fn draw_hud(
    assets: &Assets,
    stats: &FrameStats,
    state: HudState,
    cache: &mut HudTextCache,
    dt: f32,
) {
    cache.refresh_timer += dt;
    if cache.refresh_timer >= HUD_SAMPLE_SECONDS {
        cache.update(stats, state);
    }

    let snapshot = stats.snapshot;
    if snapshot.avg_ms <= 0.0 {
        cache.update(stats, state);
    }

    draw_rectangle(
        HUD_MARGIN,
        HUD_MARGIN,
        screen_width() - HUD_MARGIN * 2.0,
        HUD_BACKGROUND_HEIGHT,
        Color::new(0.0, 0.0, 0.0, 0.55),
    );
    for (index, line) in cache.lines.iter().enumerate() {
        draw_text_ex(
            line,
            HUD_TEXT_X,
            HUD_TEXT_FIRST_BASELINE + HUD_LINE_HEIGHT * index as f32,
            TextParams {
                font: Some(&assets.font),
                font_size: HUD_FONT_SIZE,
                color: WHITE,
                ..Default::default()
            },
        );
    }
}

pub fn warm_hud_font_cache(assets: &Assets) {
    draw_text_ex(
        "STATUS SCENE SYNC FRAME STABLE Q PASS WARN WAIT LOAD CLEAR FULL PACE MACH SLEEP SPIN AUTO LOG ON OFF CPU MODE DRAW TEX PROBE BANDS STEP BGD next total ms slow spk BG 0123456789.-/%|",
        0.0,
        0.0,
        TextParams {
            font: Some(&assets.font),
            font_size: HUD_FONT_SIZE,
            color: CLEAR_COLOR,
            ..Default::default()
        },
    );
}
