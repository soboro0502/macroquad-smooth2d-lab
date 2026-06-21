mod app_options;
mod diagnostics;
mod game;

use app_options::{AppOptions, LoopMode, PacerMode};
use diagnostics::{
    diagnostic_verdict, draw_frame_marker, draw_hud, fps_from_dt, frame_marker, log_frame_marker,
    warm_hud_font_cache, FrameMarker, HudState, HudTextCache,
};
use game::{Assets, Game, InputState};
use macroquad::miniquad::conf::{AppleGfxApi, Platform};
use macroquad::prelude::*;
use macroquad_smooth2d_lab::config::*;
use macroquad_smooth2d_lab::cpu_stats::CpuStats;
use macroquad_smooth2d_lab::frame_log::FrameLog;
use macroquad_smooth2d_lab::frame_pacer::{FramePacer, PacerSample};
use macroquad_smooth2d_lab::frame_stats::{FrameStats, RunFrameStats, RunValueStats};
use macroquad_smooth2d_lab::platform_tuning::{self, ThreadTuningResult};

struct DiagSession {
    diag_seconds: f64,
    frames: RunFrameStats,
    bg_delta: RunValueStats,
    pacer_spin: RunValueStats,
    pacer_wait: RunValueStats,
    pacer_total: RunValueStats,
    next_frame: RunValueStats,
    started_at: Option<f64>,
}

impl DiagSession {
    fn new(diag_seconds: f64, target_refresh_hz: u32) -> Self {
        let capacity = diag_sample_capacity(diag_seconds, target_refresh_hz);
        Self {
            diag_seconds,
            frames: RunFrameStats::new(capacity),
            bg_delta: RunValueStats::new(capacity),
            pacer_spin: RunValueStats::new(capacity),
            pacer_wait: RunValueStats::new(capacity),
            pacer_total: RunValueStats::new(capacity),
            next_frame: RunValueStats::new(capacity),
            started_at: None,
        }
    }

    fn reset(&mut self) {
        self.frames.reset();
        self.bg_delta.reset();
        self.pacer_spin.reset();
        self.pacer_wait.reset();
        self.pacer_total.reset();
        self.next_frame.reset();
    }

    fn record_values(
        &mut self,
        bg_delta: f32,
        pacer_spin_ms: f32,
        pacer_os_wait_ms: f32,
        pacer_total_ms: f32,
        next_frame_ms: f32,
    ) {
        self.bg_delta.record(bg_delta);
        self.pacer_spin.record(pacer_spin_ms);
        self.pacer_wait.record(pacer_os_wait_ms);
        self.pacer_total.record(pacer_total_ms);
        self.next_frame.record(next_frame_ms);
    }

    fn finalize(&self, frame_log: &FrameLog, cpu_percent: f32, dt: f32, target_dt: f32) {
        let final_snapshot = self.frames.snapshot(fps_from_dt(dt), target_dt);
        frame_log.final_summary(
            final_snapshot,
            cpu_percent,
            diagnostic_verdict(final_snapshot, target_dt),
        );
        frame_log.value_final_summary("bg_delta", self.bg_delta.snapshot());
        frame_log.value_final_summary("pacer_spin_ms", self.pacer_spin.snapshot());
        frame_log.value_final_summary("pacer_os_wait_ms", self.pacer_wait.snapshot());
        frame_log.value_final_summary("pacer_total_wait_ms", self.pacer_total.snapshot());
        frame_log.value_final_summary("next_frame_ms", self.next_frame.snapshot());
    }
}

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
    game.set_stress_sprite_count(app_options.stress_sprite_count);
    let mut stats = FrameStats::new();
    let mut diag_session = app_options
        .diag_seconds
        .map(|s| DiagSession::new(s, app_options.target_refresh_hz));
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
    let mut fixed_accumulator = 0.0_f32;
    let mut pending_fixed_input = InputState::default();
    let app_started_at = get_time();
    let mut hud_font_cache_warmed = false;

    loop {
        let frame_start = get_time();
        let startup_warming = frame_start - app_started_at < app_options.startup_warmup_seconds;
        let measured_dt = (frame_start - previous_loop_time) as f32;
        previous_loop_time = frame_start;
        let dt = measured_dt.max(0.0);
        let mut started_diag_measurement_this_frame = false;
        if let Some(session) = diag_session.as_mut() {
            if session.started_at.is_none()
                && frame_start - app_started_at
                    >= app_options.startup_warmup_seconds + app_options.diag_warmup_seconds
            {
                stats.reset();
                session.reset();
                cpu_stats.reset();
                last_pacer_sample = PacerSample::default();
                last_next_frame_secs = 0.0;
                frame_log.reset_clock();
                session.started_at = Some(frame_start);
                started_diag_measurement_this_frame = true;
                eprintln!(
                    "[frame-diag] measurement started after {:.1}s startup + {:.1}s diag warmup",
                    app_options.startup_warmup_seconds, app_options.diag_warmup_seconds,
                );
            }
        }
        let measurement_enabled = diag_session.is_some() || hud_visible || frame_log.enabled();
        let record_frame = !startup_warming
            && measurement_enabled
            && match diag_session.as_ref() {
                Some(session) => {
                    session.started_at.is_some() && !started_diag_measurement_this_frame
                }
                None => true,
            };
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
            if let Some(session) = diag_session.as_mut() {
                session.frames.record(dt);
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
                match app_options.loop_mode {
                    LoopMode::RenderStep => {
                        let input = InputState::read();
                        game.update(input, dt);
                    }
                    LoopMode::Fixed60Draw => {
                        pending_fixed_input.accumulate(InputState::read());
                        fixed_accumulator =
                            (fixed_accumulator + dt).min(MAX_ACCUMULATED_FIXED_TIME_SECS);

                        let mut fixed_steps = 0;
                        while fixed_accumulator >= FIXED_LOGIC_DT
                            && fixed_steps < MAX_FIXED_STEPS_PER_FRAME
                        {
                            game.fixed60_update(pending_fixed_input);
                            pending_fixed_input.clear_edges();
                            fixed_accumulator -= FIXED_LOGIC_DT;
                            fixed_steps += 1;
                        }

                        if fixed_accumulator >= FIXED_LOGIC_DT {
                            fixed_accumulator = 0.0;
                        }
                    }
                }
            } else {
                fixed_accumulator = 0.0;
                pending_fixed_input = InputState::default();
            }
            match app_options.loop_mode {
                LoopMode::RenderStep => game.draw(&assets),
                LoopMode::Fixed60Draw => {
                    let alpha = (fixed_accumulator / FIXED_LOGIC_DT).clamp(0.0, 1.0);
                    game.draw_interpolated(&assets, alpha);
                }
            }
        }
        if record_frame {
            if let Some(session) = diag_session.as_mut() {
                session.record_values(
                    game.background_last_delta(),
                    last_pacer_sample.spin_ms(),
                    last_pacer_sample.os_wait_ms(),
                    last_pacer_sample.total_wait_ms(),
                    (last_next_frame_secs * 1000.0) as f32,
                );
            }
        }
        draw_frame_marker(frame_marker);

        if hud_visible {
            draw_hud(
                &assets,
                &stats,
                HudState {
                    profile: app_options.profile,
                    loop_mode: app_options.loop_mode,
                    scroll_enabled: game.scroll_enabled(),
                    timing_mode: game.timing_mode(),
                    diagonal_mode: game.diagonal_mode(),
                    background_mode: game.background_mode(),
                    background_frame_step: game.background_frame_step(),
                    background_last_delta: game.background_last_delta(),
                    player_speed_scale: game.player_speed_scale(),
                    stress_sprite_count: game.stress_sprite_count(),
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

        if let Some(session) = diag_session.as_ref() {
            if let Some(diag_started_at) = session.started_at {
                if get_time() - diag_started_at >= session.diag_seconds {
                    session.finalize(&frame_log, cpu_stats.percent, dt, target_dt);
                    std::process::exit(0);
                }
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
                PacerMode::PresentSleep => frame_pacer.sleep_for(PACER_PRESENT_SLEEP_SECS),
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
