use macroquad::prelude::*;

pub const WINDOW_TITLE: &str = "Arcade Quality Smooth Engine";
pub const WINDOW_WIDTH: i32 = 1920;
pub const WINDOW_HEIGHT: i32 = 1080;
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

pub const PLAYER_FRAME_COUNT: usize = 5;
pub const PLAYER_CENTER_FRAME: usize = 2;
pub const PLAYER_DRAW_SCALE: f32 = 2.0;
pub const BACKGROUND_DRAW_SCALE: f32 = 2.0;
pub const PROBE_BAND_HEIGHT: f32 = 180.0;
pub const PROBE_BAND_GAP: f32 = 140.0;
pub const PROBE_MARKER_HEIGHT: f32 = 12.0;

pub const ASSET_BG_TEST: &str = "assets/bg_test.png";
pub const ASSET_BG_PROBE: &str = "assets/bg_probe.png";
pub const ASSET_PLAYER: &str = "assets/sprites/player_01_64.png";
pub const ASSET_FONT: &str = "assets/fonts/x14y20pxScoreDozer.ttf";

pub const HUD_SAMPLE_SECONDS: f32 = 1.0;
pub const HUD_RING_SIZE: usize = 240;
pub const HUD_FONT_SIZE: u16 = 20;
pub const TARGET_REFRESH_HZ: f32 = 120.0;
pub const TARGET_REFRESH_HZ_U32: u32 = 120;
pub const FRAME_SPIKE_HZ: f32 = 100.0;
pub const FRAME_FAST_HZ: f32 = 150.0;
pub const FRAME_SPIKE_MARKER_SIZE: f32 = 28.0;
pub const FRAME_MARKER_MARGIN: f32 = 16.0;
pub const FRAME_MARKER_GAP: f32 = 10.0;
pub const PACER_SLEEP_THRESHOLD_SECS: f64 = 0.002;
pub const PACER_SLEEP_MARGIN_SECS: f64 = 0.0005;
pub const FRAME_LOG_ENV: &str = "RUST_STG_FRAME_LOG";
pub const FRAME_LOG_INTERVAL_SECONDS: f64 = 5.0;

pub const CLEAR_COLOR: Color = Color::new(0.02, 0.025, 0.035, 1.0);
