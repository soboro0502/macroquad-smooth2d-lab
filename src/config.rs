use macroquad::prelude::*;

pub const WINDOW_TITLE: &str = "Rust STG Smoothness Test";
pub const WINDOW_WIDTH: i32 = 960;
pub const WINDOW_HEIGHT: i32 = 720;

pub const FIXED_LOGIC_HZ: f32 = 60.0;
pub const FIXED_DT: f32 = 1.0 / FIXED_LOGIC_HZ;
pub const MAX_FRAME_DT: f32 = 0.1;
pub const MAX_FIXED_STEPS_PER_FRAME: usize = 5;

pub const PLAYER_SPEED: f32 = 360.0;
pub const PLAYER_SLOW_SPEED: f32 = 180.0;
pub const BACKGROUND_SCROLL_SPEED: f32 = 90.0;

pub const PLAYER_FRAME_COUNT: usize = 5;
pub const PLAYER_CENTER_FRAME: usize = 2;
pub const PLAYER_LEAN_STEP_SECONDS: f32 = 0.08;

pub const ASSET_BG_TEST: &str = "assets/bg_test.png";
pub const ASSET_PLAYER: &str = "assets/sprites/player_01_64.png";
pub const ASSET_FONT: &str = "assets/fonts/x14y20pxScoreDozer.ttf";

pub const HUD_SAMPLE_SECONDS: f32 = 1.0;
pub const HUD_RING_SIZE: usize = 240;
pub const HUD_FONT_SIZE: u16 = 20;

pub const CLEAR_COLOR: Color = Color::new(0.02, 0.025, 0.035, 1.0);
