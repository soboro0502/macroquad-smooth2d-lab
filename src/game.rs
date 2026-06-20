use macroquad::prelude::*;

use crate::config::*;

pub struct Assets {
    pub background: Texture2D,
    pub player: Texture2D,
    pub font: Font,
}

impl Assets {
    pub async fn load() -> Self {
        let background = load_texture(ASSET_BG_TEST)
            .await
            .expect("failed to load test background");
        let player = load_texture(ASSET_PLAYER)
            .await
            .expect("failed to load player sprite sheet");
        let mut font = load_ttf_font(ASSET_FONT)
            .await
            .expect("failed to load HUD font");

        background.set_filter(FilterMode::Linear);
        player.set_filter(FilterMode::Linear);
        font.set_filter(FilterMode::Nearest);

        Self {
            background,
            player,
            font,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct InputState {
    pub axis: Vec2,
    pub slow: bool,
    pub toggle_scroll: bool,
}

impl InputState {
    pub fn read() -> Self {
        let mut axis = Vec2::ZERO;

        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            axis.x -= 1.0;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            axis.x += 1.0;
        }
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
            axis.y -= 1.0;
        }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
            axis.y += 1.0;
        }
        if axis.length_squared() > 1.0 {
            axis = axis.normalize();
        }

        Self {
            axis,
            slow: is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift),
            toggle_scroll: is_key_pressed(KeyCode::Space),
        }
    }
}

pub struct Game {
    player: Player,
    background: ScrollingBackground,
}

impl Game {
    pub fn new(assets: &Assets) -> Self {
        let frame_size = player_frame_size(&assets.player);
        let start = vec2(
            screen_width() * 0.5 - frame_size.x * 0.5,
            screen_height() * 0.75 - frame_size.y * 0.5,
        );

        Self {
            player: Player::new(start, frame_size),
            background: ScrollingBackground::new(assets.background.height()),
        }
    }

    pub fn update(&mut self, input: InputState, dt: f32) {
        if input.toggle_scroll {
            self.background.toggle();
        }
        self.background.update(dt);
        self.player.update(input, dt);
    }

    pub fn draw(&self, assets: &Assets) {
        self.background.draw(&assets.background);
        self.player.draw(&assets.player);
    }

    pub fn scroll_enabled(&self) -> bool {
        self.background.enabled
    }
}

struct Player {
    position: Vec2,
    frame_size: Vec2,
    current_frame: usize,
}

impl Player {
    fn new(position: Vec2, frame_size: Vec2) -> Self {
        Self {
            position,
            frame_size,
            current_frame: PLAYER_CENTER_FRAME,
        }
    }

    fn update(&mut self, input: InputState, dt: f32) {
        let speed = if input.slow {
            PLAYER_SLOW_SPEED
        } else {
            PLAYER_SPEED
        };
        self.position += input.axis * speed * dt;
        self.position.x = self
            .position
            .x
            .clamp(0.0, screen_width() - self.frame_size.x);
        self.position.y = self
            .position
            .y
            .clamp(0.0, screen_height() - self.frame_size.y);
        self.current_frame = target_frame_for_axis(input.axis.x);
    }

    fn draw(&self, texture: &Texture2D) {
        let source = Rect::new(
            self.frame_size.x * self.current_frame as f32,
            0.0,
            self.frame_size.x,
            self.frame_size.y,
        );

        draw_texture_ex(
            texture,
            self.position.x,
            self.position.y,
            WHITE,
            DrawTextureParams {
                source: Some(source),
                dest_size: Some(self.frame_size),
                ..Default::default()
            },
        );
    }
}

struct ScrollingBackground {
    offset: f32,
    tile_height: f32,
    enabled: bool,
}

impl ScrollingBackground {
    fn new(tile_height: f32) -> Self {
        Self {
            offset: 0.0,
            tile_height,
            enabled: true,
        }
    }

    fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    fn update(&mut self, dt: f32) {
        if self.enabled {
            self.offset = (self.offset + BACKGROUND_SCROLL_SPEED * dt) % self.tile_height;
        }
    }

    fn draw(&self, texture: &Texture2D) {
        let tile_width = texture.width();
        let columns = (screen_width() / tile_width).ceil() as i32 + 1;
        let rows = (screen_height() / self.tile_height).ceil() as i32 + 2;

        for row in -1..rows {
            for column in 0..columns {
                draw_texture(
                    texture,
                    column as f32 * tile_width,
                    row as f32 * self.tile_height + self.offset,
                    WHITE,
                );
            }
        }
    }
}

fn player_frame_size(texture: &Texture2D) -> Vec2 {
    vec2(
        texture.width() / PLAYER_FRAME_COUNT as f32,
        texture.height(),
    )
}

fn target_frame_for_axis(axis_x: f32) -> usize {
    if axis_x < -0.5 {
        0
    } else if axis_x < 0.0 {
        1
    } else if axis_x > 0.5 {
        4
    } else if axis_x > 0.0 {
        3
    } else {
        PLAYER_CENTER_FRAME
    }
}
