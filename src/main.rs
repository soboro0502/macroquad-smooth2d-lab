mod config;
mod frame_stats;
mod game;

use config::*;
use frame_stats::FrameStats;
use game::{Assets, Game, InputState, TimingMode};
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: WINDOW_TITLE.to_string(),
        window_width: WINDOW_WIDTH,
        window_height: WINDOW_HEIGHT,
        high_dpi: false,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let assets = Assets::load().await;
    let mut game = Game::new(&assets);
    let mut stats = FrameStats::new();
    let mut hud_visible = false;
    let mut clear_only = false;

    loop {
        let dt = get_frame_time();
        if is_key_pressed(KeyCode::H) {
            hud_visible = !hud_visible;
        }
        if is_key_pressed(KeyCode::C) {
            clear_only = !clear_only;
        }
        stats.record(dt, get_fps(), 1.0 / TARGET_REFRESH_HZ);
        let frame_marker = frame_marker(dt);

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
            );
        }

        next_frame().await;
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
) {
    let snapshot = stats.snapshot;
    let scroll = if scroll_enabled { "ON" } else { "OFF" };
    let load = if clear_only { "CLEAR" } else { "FULL" };
    let text = format!(
        "LOAD {}  MODE {}  DRAW {}  BGSTEP {:.0}px  fps {:>3}  last {:>5.2}ms  avg {:>5.2}ms  range {:>5.2}-{:>5.2}  sd {:>4.2}  slow {:>4.1}%  spikes {:>2}  BG {}",
        load,
        timing_mode.label(),
        background_mode.label(),
        background_frame_step,
        snapshot.fps,
        snapshot.last_ms,
        snapshot.avg_ms,
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
