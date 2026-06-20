mod config;
mod cpu_stats;
mod frame_log;
mod frame_pacer;
mod frame_stats;
mod game;

use config::*;
use cpu_stats::CpuStats;
use frame_log::FrameLog;
use frame_pacer::FramePacer;
use frame_stats::FrameStats;
use game::{Assets, Game, InputState, TimingMode};
use macroquad::miniquad::conf::{AppleGfxApi, Platform};
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: WINDOW_TITLE.to_string(),
        window_width: WINDOW_WIDTH,
        window_height: WINDOW_HEIGHT,
        high_dpi: false,
        window_resizable: false,
        platform: Platform {
            apple_gfx_api: AppleGfxApi::Metal,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let app_options = AppOptions::from_args(std::env::args().skip(1));
    let assets = Assets::load().await;
    let mut game = Game::new(&assets);
    let mut stats = FrameStats::new();
    let mut cpu_stats = CpuStats::new(HUD_SAMPLE_SECONDS);
    let mut frame_log = FrameLog::new(
        app_options.diag_seconds.is_some() || std::env::var_os(FRAME_LOG_ENV).is_some(),
    );
    let frame_pacer = FramePacer::new();
    let mut hud_visible = false;
    let mut clear_only = app_options.clear_only;
    let mut manual_pacer_enabled = app_options.manual_pacer_enabled;
    let mut previous_loop_time = get_time() - 1.0 / f64::from(TARGET_REFRESH_HZ_U32);
    let started_at = get_time();

    loop {
        let frame_start = get_time();
        let measured_dt = (frame_start - previous_loop_time) as f32;
        previous_loop_time = frame_start;
        let dt = measured_dt.max(0.0);
        if is_key_pressed(KeyCode::H) {
            hud_visible = !hud_visible;
        }
        if is_key_pressed(KeyCode::C) {
            clear_only = !clear_only;
        }
        if is_key_pressed(KeyCode::P) {
            manual_pacer_enabled = !manual_pacer_enabled;
        }
        if is_key_pressed(KeyCode::L) {
            frame_log.toggle();
        }
        stats.record(dt, fps_from_dt(dt), 1.0 / TARGET_REFRESH_HZ);
        cpu_stats.update(dt);
        let frame_marker = frame_marker(dt);
        log_frame_marker(
            &frame_log,
            frame_marker,
            dt,
            fps_from_dt(dt),
            clear_only,
            manual_pacer_enabled,
        );

        clear_background(CLEAR_COLOR);
        if !clear_only {
            let input = InputState::read();
            game.update(input, dt);
            game.draw(&assets);
        }
        draw_frame_marker(frame_marker);

        if hud_visible {
            draw_hud(
                &assets,
                &stats,
                game.scroll_enabled(),
                game.timing_mode(),
                game.background_mode(),
                game.background_frame_step(),
                clear_only,
                manual_pacer_enabled,
                cpu_stats.percent,
                frame_log.enabled(),
            );
        }
        frame_log.summary(stats.snapshot, cpu_stats.percent);

        if let Some(diag_seconds) = app_options.diag_seconds {
            if get_time() - started_at >= diag_seconds {
                frame_log.final_summary(stats.snapshot, cpu_stats.percent);
                std::process::exit(0);
            }
        }

        next_frame().await;
        if manual_pacer_enabled {
            frame_pacer.wait_until(frame_start, TARGET_REFRESH_HZ_U32);
        }
    }
}

#[derive(Clone, Copy)]
struct AppOptions {
    diag_seconds: Option<f64>,
    clear_only: bool,
    manual_pacer_enabled: bool,
}

impl AppOptions {
    fn from_args(mut args: impl Iterator<Item = String>) -> Self {
        let mut options = Self {
            diag_seconds: None,
            clear_only: false,
            manual_pacer_enabled: DEFAULT_MANUAL_PACER_ENABLED,
        };

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--diag" => {
                    options.diag_seconds = Some(DEFAULT_DIAG_SECONDS);
                }
                "--diag-seconds" => {
                    options.diag_seconds = args
                        .next()
                        .and_then(|seconds| seconds.parse::<f64>().ok())
                        .filter(|seconds| *seconds > 0.0);
                }
                "--diag-clear" => {
                    options.clear_only = true;
                }
                "--diag-manual" => {
                    options.manual_pacer_enabled = true;
                }
                "--diag-auto" => {
                    options.manual_pacer_enabled = false;
                }
                _ => {}
            }
        }

        options
    }
}

fn fps_from_dt(dt: f32) -> i32 {
    (1.0 / dt.max(f32::EPSILON)) as i32
}

fn log_frame_marker(
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

#[derive(Clone, Copy, Eq, PartialEq)]
enum FrameMarker {
    None,
    Slow,
    Fast,
}

fn frame_marker(dt: f32) -> FrameMarker {
    if dt >= 1.0 / FRAME_SPIKE_HZ {
        FrameMarker::Slow
    } else if dt <= 1.0 / FRAME_FAST_HZ {
        FrameMarker::Fast
    } else {
        FrameMarker::None
    }
}

fn draw_frame_marker(frame_marker: FrameMarker) {
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

fn draw_hud(
    assets: &Assets,
    stats: &FrameStats,
    scroll_enabled: bool,
    timing_mode: TimingMode,
    background_mode: game::BackgroundMode,
    background_frame_step: f32,
    clear_only: bool,
    manual_pacer_enabled: bool,
    cpu_percent: f32,
    frame_log_enabled: bool,
) {
    let snapshot = stats.snapshot;
    let scroll = if scroll_enabled { "ON" } else { "OFF" };
    let load = if clear_only { "CLEAR" } else { "FULL" };
    let pace = if manual_pacer_enabled {
        "MANUAL"
    } else {
        "AUTO"
    };
    let log = if frame_log_enabled { "ON" } else { "OFF" };
    let text = format!(
        "LOAD {}  PACE {}  LOG {}  MODE {}  DRAW {}  BGSTEP {:.0}px  CPU {:>5.1}%  fps {:>3}/{:>5.1}  ms last {:>5.2} avg {:>5.2} p95 {:>5.2} p99 {:>5.2} range {:>5.2}-{:>5.2} sd {:>4.2} slow {:>4.1}% spk {:>2} BG {}",
        load,
        pace,
        log,
        timing_mode.label(),
        background_mode.label(),
        background_frame_step,
        cpu_percent,
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
