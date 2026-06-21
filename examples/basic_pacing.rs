use macroquad::prelude::*;
use macroquad_smooth2d_lab::config::{
    DEFAULT_TARGET_REFRESH_HZ_U32, PACER_BALANCED_SPIN_MARGIN_SECS,
};
use macroquad_smooth2d_lab::prelude::{FramePacer, FrameStats};

fn window_conf() -> Conf {
    Conf {
        window_title: "Basic Pacing Example".to_string(),
        window_width: 960,
        window_height: 540,
        high_dpi: false,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let target_hz = DEFAULT_TARGET_REFRESH_HZ_U32;
    let target_dt = 1.0 / target_hz as f32;
    let pacer = FramePacer::new();
    let mut stats = FrameStats::new();
    let mut previous_loop_time = get_time() - f64::from(target_dt);
    let mut x = 80.0;

    loop {
        let frame_start = get_time();
        let dt = (frame_start - previous_loop_time).max(0.0) as f32;
        previous_loop_time = frame_start;
        stats.record(dt, fps_from_dt(dt), target_dt);

        x += 180.0 * dt;
        if x > screen_width() {
            x = -48.0;
        }

        clear_background(Color::new(0.02, 0.025, 0.035, 1.0));
        draw_rectangle(x, screen_height() * 0.5 - 24.0, 48.0, 48.0, WHITE);
        let label = format!(
            "{} Hz  avg={:.2}ms  p99={:.2}ms  spikes={}",
            target_hz, stats.snapshot.avg_ms, stats.snapshot.p99_ms, stats.snapshot.spike_count
        );
        draw_text(&label, 24.0, 40.0, 28.0, WHITE);

        next_frame().await;
        let _sample =
            pacer.mach_wait_spin_until(frame_start, target_hz, PACER_BALANCED_SPIN_MARGIN_SECS);
    }
}

fn fps_from_dt(dt: f32) -> i32 {
    (1.0 / dt.max(f32::EPSILON)) as i32
}
