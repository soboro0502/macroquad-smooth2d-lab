use macroquad::prelude::*;

// Window and target frame timing.
pub const WINDOW_TITLE: &str = "Rust-STG Frame Pacing Lab";
pub const WINDOW_WIDTH: i32 = 1920;
pub const WINDOW_HEIGHT: i32 = 1080;
pub const TARGET_REFRESH_HZ: f32 = 120.0;
pub const TARGET_REFRESH_HZ_U32: u32 = 120;

// Test-scene movement.
pub const PLAYER_SPEED: f32 = 520.0;
pub const PLAYER_SLOW_SPEED: f32 = 260.0;
pub const BACKGROUND_SCROLL_SPEED: f32 = 160.0;
pub const FRAME_STEP_PLAYER_PIXELS: f32 = 4.0;
pub const FRAME_STEP_PLAYER_SLOW_PIXELS: f32 = 2.0;
pub const BACKGROUND_STEP_1: f32 = 1.0;
pub const BACKGROUND_STEP_2: f32 = 2.0;
pub const BACKGROUND_STEP_3: f32 = 3.0;
pub const BACKGROUND_STEP_4: f32 = 4.0;
pub const DEFAULT_BACKGROUND_STEP: f32 = BACKGROUND_STEP_2;

// Test-scene rendering.
pub const PLAYER_FRAME_COUNT: usize = 5;
pub const PLAYER_CENTER_FRAME: usize = 2;
pub const PLAYER_DRAW_SCALE: f32 = 2.0;
pub const BACKGROUND_DRAW_SCALE: f32 = 2.0;
pub const PROBE_BAND_HEIGHT: f32 = 180.0;
pub const PROBE_BAND_GAP: f32 = 140.0;
pub const PROBE_MARKER_HEIGHT: f32 = 12.0;
pub const PROBE_GUIDE_INTERVAL: f32 = 120.0;
pub const PROBE_GUIDE_THICKNESS: f32 = 2.0;
pub const PROBE_SCROLL_LINE_THICKNESS: f32 = 4.0;

// Assets.
pub const ASSET_BG_TEST: &str = "assets/bg_test.png";
pub const ASSET_BG_PROBE: &str = "assets/bg_probe.png";
pub const ASSET_PLAYER: &str = "assets/sprites/player_01_64.png";
pub const ASSET_FONT: &str = "assets/fonts/Silkscreen-Regular.ttf";

// HUD, logging, and diagnostics.
pub const HUD_SAMPLE_SECONDS: f32 = 1.0;
pub const HUD_RING_SIZE: usize = 240;
pub const HUD_FONT_SIZE: u16 = 20;
pub const FRAME_LOG_ENV: &str = "RUST_STG_FRAME_LOG";
pub const FRAME_LOG_INTERVAL_SECONDS: f64 = 5.0;
pub const DEFAULT_DIAG_SECONDS: f64 = 12.0;
pub const DEFAULT_DIAG_WARMUP_SECONDS: f64 = 2.0;
pub const DIAG_PASS_MAX_MS: f32 = 9.0;
pub const DIAG_PASS_P99_MS: f32 = 9.0;
pub const DIAG_PASS_STDEV_MS: f32 = 0.3;
pub const DIAG_PASS_SLOW_PERCENT: f32 = 0.5;
pub const DIAG_PASS_SPIKES: usize = 0;

// Platform and frame pacing.
pub const TIME_CONSTRAINT_COMPUTATION_SECS: f64 = 0.007;
pub const TIME_CONSTRAINT_CONSTRAINT_SECS: f64 = 0.008;
pub const DEFAULT_TIME_CONSTRAINT_ENABLED: bool = true;
pub const DEFAULT_MANUAL_PACER_ENABLED: bool = true;
pub const FRAME_SPIKE_HZ: f32 = 100.0;
pub const FRAME_FAST_HZ: f32 = 150.0;
pub const FRAME_SPIKE_MARKER_SIZE: f32 = 28.0;
pub const FRAME_MARKER_MARGIN: f32 = 16.0;
pub const FRAME_MARKER_GAP: f32 = 10.0;
pub const PACER_SLEEP_THRESHOLD_SECS: f64 = 0.0062;
pub const PACER_SLEEP_MARGIN_SECS: f64 = 0.0014;

// Colors.
pub const CLEAR_COLOR: Color = Color::new(0.02, 0.025, 0.035, 1.0);
pub const PROBE_GUIDE_COLOR: Color = Color::new(0.75, 0.78, 0.84, 0.22);
pub const PROBE_SCROLL_LINE_COLOR: Color = Color::new(1.0, 0.96, 0.30, 1.0);
