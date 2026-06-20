use macroquad::prelude::*;

pub const WINDOW_TITLE: &str = "Arcade Quality Smooth Engine";
pub const WINDOW_WIDTH: i32 = 1920;
pub const WINDOW_HEIGHT: i32 = 1080;
pub const PLAYER_SPEED: f32 = 520.0;
pub const PLAYER_SLOW_SPEED: f32 = 260.0;
pub const BACKGROUND_SCROLL_SPEED: f32 = 160.0;
pub const FRAME_STEP_PLAYER_PIXELS: f32 = 4.0;
pub const FRAME_STEP_PLAYER_SLOW_PIXELS: f32 = 2.0;
pub const FRAME_STEP_BACKGROUND_PIXELS: f32 = 1.0;

pub const PLAYER_FRAME_COUNT: usize = 5;
pub const PLAYER_CENTER_FRAME: usize = 2;

pub const ASSET_BG_TEST: &str = "assets/bg_test.png";
pub const ASSET_PLAYER: &str = "assets/sprites/player_01_64.png";
pub const ASSET_FONT: &str = "assets/fonts/x14y20pxScoreDozer.ttf";

pub const HUD_SAMPLE_SECONDS: f32 = 1.0;
pub const HUD_RING_SIZE: usize = 240;
pub const HUD_FONT_SIZE: u16 = 20;
pub const TARGET_REFRESH_HZ: f32 = 120.0;

pub const CLEAR_COLOR: Color = Color::new(0.02, 0.025, 0.035, 1.0);
