mod app_options;
mod config;
mod cpu_stats;
mod diagnostics;
mod frame_log;
mod frame_pacer;
mod frame_stats;
mod game;
mod platform_tuning;

use app_options::{AppOptions, PacerMode};
use config::*;
use cpu_stats::CpuStats;
use diagnostics::{
    diagnostic_verdict, draw_frame_marker, draw_hud, fps_from_dt, frame_marker, log_frame_marker,
    warm_hud_font_cache, FrameMarker, HudState, HudTextCache,
};
use frame_log::FrameLog;
use frame_pacer::{FramePacer, PacerSample};
use frame_stats::{FrameStats, RunFrameStats, RunValueStats};
use game::{Assets, Game, InputState};
use macroquad::miniquad::conf::{AppleGfxApi, Platform};
use macroquad::prelude::*;
use platform_tuning::ThreadTuningResult;

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
    let target_dt = 1.0 / app_options.target_refresh_hz as f32;
    let assets = Assets::load().await;
    let thread_tuning = platform_tuning::set_latency_sensitive_thread();
    if app_options.diag_seconds.is_some() || std::env::var_os(FRAME_LOG_ENV).is_some() {
        log_thread_tuning("qos=user-interactive", thread_tuning);
    }
    if app_options.time_constraint_enabled {
        let time_constraint_tuning = platform_tuning::set_time_constraint_thread(
            1.0 / f64::from(app_options.target_refresh_hz),
            TIME_CONSTRAINT_COMPUTATION_SECS,
            TIME_CONSTRAINT_CONSTRAINT_SECS,
        );
        if app_options.diag_seconds.is_some() || std::env::var_os(FRAME_LOG_ENV).is_some() {
            log_thread_tuning("time-constraint", time_constraint_tuning);
        }
    }
    let mut game = Game::new(&assets, app_options.target_refresh_hz);
    game.set_timing_mode(app_options.timing_mode);
    game.set_background_mode(app_options.background_mode);
    game.set_background_frame_step(app_options.background_frame_step);
    let mut stats = FrameStats::new();
    let mut run_stats = app_options.diag_seconds.map(|diag_seconds| {
        RunFrameStats::new(diag_sample_capacity(
            diag_seconds,
            app_options.target_refresh_hz,
        ))
    });
    let mut run_bg_delta_stats = app_options.diag_seconds.map(|diag_seconds| {
        RunValueStats::new(diag_sample_capacity(
            diag_seconds,
            app_options.target_refresh_hz,
        ))
    });
    let mut run_pacer_spin_stats = app_options.diag_seconds.map(|diag_seconds| {
        RunValueStats::new(diag_sample_capacity(
            diag_seconds,
            app_options.target_refresh_hz,
        ))
    });
    let mut run_pacer_wait_stats = app_options.diag_seconds.map(|diag_seconds| {
        RunValueStats::new(diag_sample_capacity(
            diag_seconds,
            app_options.target_refresh_hz,
        ))
    });
    let mut run_pacer_total_stats = app_options.diag_seconds.map(|diag_seconds| {
        RunValueStats::new(diag_sample_capacity(
            diag_seconds,
            app_options.target_refresh_hz,
        ))
    });
    let mut run_next_frame_stats = app_options.diag_seconds.map(|diag_seconds| {
        RunValueStats::new(diag_sample_capacity(
            diag_seconds,
            app_options.target_refresh_hz,
        ))
    });
    let mut cpu_stats = CpuStats::new(HUD_SAMPLE_SECONDS);
    let mut frame_log = FrameLog::new(
        app_options.diag_seconds.is_some() || std::env::var_os(FRAME_LOG_ENV).is_some(),
    );
    let frame_pacer = FramePacer::new();
    let mut hud_visible = app_options.hud_visible;
    let mut hud_cache = HudTextCache::new();
    let mut clear_only = app_options.clear_only;
    let mut manual_pacer_enabled = app_options.manual_pacer_enabled;
    let mut last_pacer_sample = PacerSample::default();
    let mut last_next_frame_secs = 0.0;
    let mut previous_loop_time = get_time() - f64::from(target_dt);
    let app_started_at = get_time();
    let mut hud_font_cache_warmed = false;
    let mut diag_measurement_started_at = None;

    loop {
        let frame_start = get_time();
        let startup_warming = frame_start - app_started_at < app_options.startup_warmup_seconds;
        let measured_dt = (frame_start - previous_loop_time) as f32;
        previous_loop_time = frame_start;
        let dt = measured_dt.max(0.0);
        let mut started_diag_measurement_this_frame = false;
        if app_options.diag_seconds.is_some()
            && diag_measurement_started_at.is_none()
            && frame_start - app_started_at
                >= app_options.startup_warmup_seconds + app_options.diag_warmup_seconds
        {
            stats.reset();
            if let Some(run_stats) = run_stats.as_mut() {
                run_stats.reset();
            }
            if let Some(run_bg_delta_stats) = run_bg_delta_stats.as_mut() {
                run_bg_delta_stats.reset();
            }
            if let Some(run_pacer_spin_stats) = run_pacer_spin_stats.as_mut() {
                run_pacer_spin_stats.reset();
            }
            if let Some(run_pacer_wait_stats) = run_pacer_wait_stats.as_mut() {
                run_pacer_wait_stats.reset();
            }
            if let Some(run_pacer_total_stats) = run_pacer_total_stats.as_mut() {
                run_pacer_total_stats.reset();
            }
            if let Some(run_next_frame_stats) = run_next_frame_stats.as_mut() {
                run_next_frame_stats.reset();
            }
            cpu_stats.reset();
            last_pacer_sample = PacerSample::default();
            last_next_frame_secs = 0.0;
            frame_log.reset_clock();
            diag_measurement_started_at = Some(frame_start);
            started_diag_measurement_this_frame = true;
            eprintln!(
                "[frame-diag] measurement started after {:.1}s startup + {:.1}s diag warmup",
                app_options.startup_warmup_seconds, app_options.diag_warmup_seconds,
            );
        }
        let measurement_enabled =
            app_options.diag_seconds.is_some() || hud_visible || frame_log.enabled();
        let record_frame = !startup_warming
            && measurement_enabled
            && (app_options.diag_seconds.is_none()
                || (diag_measurement_started_at.is_some() && !started_diag_measurement_this_frame));
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
        let frame_marker = if record_frame {
            stats.record(dt, fps_from_dt(dt), target_dt);
            if let Some(run_stats) = run_stats.as_mut() {
                run_stats.record(dt);
            }
            cpu_stats.update(dt);
            let frame_marker = frame_marker(dt, target_dt);
            log_frame_marker(
                &frame_log,
                frame_marker,
                dt,
                fps_from_dt(dt),
                clear_only,
                manual_pacer_enabled,
            );
            frame_marker
        } else {
            FrameMarker::None
        };

        clear_background(CLEAR_COLOR);
        if !hud_font_cache_warmed {
            warm_hud_font_cache(&assets);
            hud_font_cache_warmed = true;
        }
        if !clear_only {
            if !startup_warming {
                let input = InputState::read();
                game.update(input, dt);
            }
            game.draw(&assets);
        }
        if record_frame {
            if let Some(run_bg_delta_stats) = run_bg_delta_stats.as_mut() {
                run_bg_delta_stats.record(game.background_last_delta());
            }
            if let Some(run_pacer_spin_stats) = run_pacer_spin_stats.as_mut() {
                run_pacer_spin_stats.record(last_pacer_sample.spin_ms());
            }
            if let Some(run_pacer_wait_stats) = run_pacer_wait_stats.as_mut() {
                run_pacer_wait_stats.record(last_pacer_sample.os_wait_ms());
            }
            if let Some(run_pacer_total_stats) = run_pacer_total_stats.as_mut() {
                run_pacer_total_stats.record(last_pacer_sample.total_wait_ms());
            }
            if let Some(run_next_frame_stats) = run_next_frame_stats.as_mut() {
                run_next_frame_stats.record((last_next_frame_secs * 1000.0) as f32);
            }
        }
        draw_frame_marker(frame_marker);

        if hud_visible {
            draw_hud(
                &assets,
                &stats,
                HudState {
                    profile: app_options.profile,
                    scroll_enabled: game.scroll_enabled(),
                    timing_mode: game.timing_mode(),
                    diagonal_mode: game.diagonal_mode(),
                    background_mode: game.background_mode(),
                    background_frame_step: game.background_frame_step(),
                    background_last_delta: game.background_last_delta(),
                    player_speed_scale: game.player_speed_scale(),
                    clear_only,
                    manual_pacer_enabled,
                    pacer_mode: app_options.pacer_mode,
                    pacer_spin_margin_secs: app_options.pacer_spin_margin_secs,
                    pacer_sleep_margin_secs: app_options.pacer_sleep_margin_secs,
                    pacer_sleep_threshold_secs: app_options.pacer_sleep_threshold_secs,
                    pacer_sample: last_pacer_sample,
                    next_frame_wait_secs: last_next_frame_secs,
                    target_refresh_hz: app_options.target_refresh_hz,
                    cpu_percent: cpu_stats.percent,
                    frame_log_enabled: frame_log.enabled(),
                },
                &mut hud_cache,
                dt,
            );
        }
        if record_frame {
            frame_log.summary(stats.snapshot, cpu_stats.percent);
        }

        if let (Some(diag_seconds), Some(diag_started_at)) =
            (app_options.diag_seconds, diag_measurement_started_at)
        {
            if get_time() - diag_started_at >= diag_seconds {
                let final_snapshot = run_stats
                    .as_ref()
                    .map(|run_stats| run_stats.snapshot(fps_from_dt(dt), target_dt))
                    .unwrap_or(stats.snapshot);
                frame_log.final_summary(
                    final_snapshot,
                    cpu_stats.percent,
                    diagnostic_verdict(final_snapshot, target_dt),
                );
                if let Some(run_bg_delta_stats) = run_bg_delta_stats.as_ref() {
                    frame_log.value_final_summary("bg_delta", run_bg_delta_stats.snapshot());
                }
                if let Some(run_pacer_spin_stats) = run_pacer_spin_stats.as_ref() {
                    frame_log.value_final_summary("pacer_spin_ms", run_pacer_spin_stats.snapshot());
                }
                if let Some(run_pacer_wait_stats) = run_pacer_wait_stats.as_ref() {
                    frame_log
                        .value_final_summary("pacer_os_wait_ms", run_pacer_wait_stats.snapshot());
                }
                if let Some(run_pacer_total_stats) = run_pacer_total_stats.as_ref() {
                    frame_log.value_final_summary(
                        "pacer_total_wait_ms",
                        run_pacer_total_stats.snapshot(),
                    );
                }
                if let Some(run_next_frame_stats) = run_next_frame_stats.as_ref() {
                    frame_log.value_final_summary("next_frame_ms", run_next_frame_stats.snapshot());
                }
                std::process::exit(0);
            }
        }

        let next_frame_started_at = get_time();
        next_frame().await;
        last_next_frame_secs = get_time() - next_frame_started_at;
        last_pacer_sample = if manual_pacer_enabled {
            match app_options.pacer_mode {
                PacerMode::MachSpin => frame_pacer.mach_wait_spin_until(
                    frame_start,
                    app_options.target_refresh_hz,
                    app_options.pacer_spin_margin_secs,
                ),
                PacerMode::SleepSpin => frame_pacer.wait_until(
                    frame_start,
                    app_options.target_refresh_hz,
                    app_options.pacer_sleep_margin_secs,
                    app_options.pacer_sleep_threshold_secs,
                ),
                PacerMode::Spin => {
                    frame_pacer.spin_until(frame_start, app_options.target_refresh_hz)
                }
            }
        } else {
            PacerSample::default()
        };
    }
}

fn diag_sample_capacity(diag_seconds: f64, target_refresh_hz: u32) -> usize {
    (diag_seconds * f64::from(target_refresh_hz) * 1.25).ceil() as usize
}

fn log_thread_tuning(label: &'static str, result: ThreadTuningResult) {
    match result {
        ThreadTuningResult::Applied => eprintln!("[thread-tuning] {label}"),
        ThreadTuningResult::Failed(code) => {
            eprintln!("[thread-tuning] {label} failed code={code}");
        }
        ThreadTuningResult::Unsupported => eprintln!("[thread-tuning] {label} unsupported"),
    }
}
