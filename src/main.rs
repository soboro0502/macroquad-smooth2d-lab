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
    let mut spike_flash = 0.0;

    loop {
        let dt = get_frame_time();
        if is_key_pressed(KeyCode::H) {
            hud_visible = !hud_visible;
        }
        stats.record(dt, get_fps(), 1.0 / TARGET_REFRESH_HZ);
        spike_flash = update_spike_flash(spike_flash, dt);

        let input = InputState::read();
        game.update(input, dt);

        clear_background(CLEAR_COLOR);
        game.draw(&assets);
        draw_spike_marker(spike_flash);

        if hud_visible {
            draw_hud(
                &assets,
                &stats,
                game.scroll_enabled(),
                game.timing_mode(),
                game.background_mode(),
                game.background_frame_step(),
            );
        }

        next_frame().await;
    }
}

fn update_spike_flash(spike_flash: f32, dt: f32) -> f32 {
    let spike_dt = 1.0 / FRAME_SPIKE_HZ;
    if dt >= spike_dt {
        FRAME_SPIKE_FLASH_SECONDS
    } else {
        (spike_flash - dt).max(0.0)
    }
}

fn draw_spike_marker(spike_flash: f32) {
    if spike_flash <= 0.0 {
        return;
    }

    draw_rectangle(
        8.0,
        8.0,
        FRAME_SPIKE_MARKER_SIZE,
        FRAME_SPIKE_MARKER_SIZE,
        Color::new(1.0, 0.08, 0.08, 0.9),
    );
}

fn draw_hud(
    assets: &Assets,
    stats: &FrameStats,
    scroll_enabled: bool,
    timing_mode: TimingMode,
    background_mode: game::BackgroundMode,
    background_frame_step: f32,
) {
    let snapshot = stats.snapshot;
    let scroll = if scroll_enabled { "ON" } else { "OFF" };
    let text = format!(
        "MODE {}  DRAW {}  BGSTEP {:.0}px  fps {:>3}  avg {:>5.2}ms  range {:>5.2}-{:>5.2}  sd {:>4.2}  slow {:>4.1}%  spikes {:>2}  BG {}",
        timing_mode.label(),
        background_mode.label(),
        background_frame_step,
        snapshot.fps,
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
