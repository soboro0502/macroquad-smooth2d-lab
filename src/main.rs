mod config;
mod frame_stats;
mod game;

use macroquad::prelude::*;

use config::*;
use frame_stats::FrameStats;
use game::{Assets, Game, InputState};

fn window_conf() -> Conf {
    Conf {
        window_title: WINDOW_TITLE.to_string(),
        window_width: WINDOW_WIDTH,
        window_height: WINDOW_HEIGHT,
        high_dpi: false,
        window_resizable: false,
        platform: miniquad::conf::Platform {
            swap_interval: Some(1),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let assets = Assets::load().await;
    let mut game = Game::new(&assets);
    let mut stats = FrameStats::new();
    let mut accumulator = 0.0;

    loop {
        let frame_dt = get_frame_time().min(MAX_FRAME_DT);
        accumulator += frame_dt;

        let input = InputState::read();
        let mut fixed_steps = 0;
        while accumulator >= FIXED_DT && fixed_steps < MAX_FIXED_STEPS_PER_FRAME {
            game.fixed_update(input);
            accumulator -= FIXED_DT;
            fixed_steps += 1;
        }
        if fixed_steps == MAX_FIXED_STEPS_PER_FRAME {
            accumulator = accumulator.min(FIXED_DT);
        }

        let alpha = (accumulator / FIXED_DT).clamp(0.0, 1.0);

        clear_background(CLEAR_COLOR);
        game.draw(&assets, alpha);
        draw_hud(&assets, &stats, alpha, fixed_steps);

        stats.record(get_frame_time(), get_fps(), 1.0 / 120.0);
        next_frame().await;
    }
}

fn draw_hud(assets: &Assets, stats: &FrameStats, alpha: f32, fixed_steps: usize) {
    let snapshot = stats.snapshot;
    let text = format!(
        "fps {:>3}  avg {:>5.2}ms  range {:>5.2}-{:>5.2}  sd {:>4.2}  slow {:>4.1}%  spikes {:>2}  alpha {:>4.2}  steps {}",
        snapshot.fps,
        snapshot.avg_ms,
        snapshot.min_ms,
        snapshot.max_ms,
        snapshot.stdev_ms,
        snapshot.slow_percent,
        snapshot.spike_count,
        alpha,
        fixed_steps
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
