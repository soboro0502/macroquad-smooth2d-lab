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

    loop {
        let dt = get_frame_time();
        let input = InputState::read();
        game.update(input, dt);

        clear_background(CLEAR_COLOR);
        game.draw(&assets);
        draw_hud(&assets, &stats, game.scroll_enabled(), game.timing_mode());

        stats.record(dt, get_fps(), 1.0 / TARGET_REFRESH_HZ);

        next_frame().await;
    }
}

fn draw_hud(assets: &Assets, stats: &FrameStats, scroll_enabled: bool, timing_mode: TimingMode) {
    let snapshot = stats.snapshot;
    let scroll = if scroll_enabled { "ON" } else { "OFF" };
    let text = format!(
        "MODE {}  fps {:>3}  avg {:>5.2}ms  range {:>5.2}-{:>5.2}  sd {:>4.2}  slow {:>4.1}%  spikes {:>2}  BG {}",
        timing_mode.label(),
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
