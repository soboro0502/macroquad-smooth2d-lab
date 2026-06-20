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

        background.set_filter(FilterMode::Nearest);
        player.set_filter(FilterMode::Nearest);
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

    pub fn fixed_update(&mut self, input: InputState) {
        self.background.fixed_update();
        self.player.fixed_update(input);
    }

    pub fn draw(&self, assets: &Assets, alpha: f32) {
        self.background.draw(&assets.background, alpha);
        self.player.draw(&assets.player, alpha);
    }
}

struct Player {
    previous_position: Vec2,
    position: Vec2,
    frame_size: Vec2,
    current_frame: usize,
    target_frame: usize,
    lean_timer: f32,
}

impl Player {
    fn new(position: Vec2, frame_size: Vec2) -> Self {
        Self {
            previous_position: position,
            position,
            frame_size,
            current_frame: PLAYER_CENTER_FRAME,
            target_frame: PLAYER_CENTER_FRAME,
            lean_timer: 0.0,
        }
    }

    fn fixed_update(&mut self, input: InputState) {
        self.previous_position = self.position;

        let speed = if input.slow {
            PLAYER_SLOW_SPEED
        } else {
            PLAYER_SPEED
        };
        self.position += input.axis * speed * FIXED_DT;
        self.position.x = self
            .position
            .x
            .clamp(0.0, screen_width() - self.frame_size.x);
        self.position.y = self
            .position
            .y
            .clamp(0.0, screen_height() - self.frame_size.y);

        self.target_frame = target_frame_for_axis(input.axis.x);
        self.update_lean_frame();
    }

    fn update_lean_frame(&mut self) {
        self.lean_timer += FIXED_DT;
        if self.lean_timer < PLAYER_LEAN_STEP_SECONDS {
            return;
        }

        self.lean_timer = 0.0;
        self.current_frame = match self.current_frame.cmp(&self.target_frame) {
            std::cmp::Ordering::Less => self.current_frame + 1,
            std::cmp::Ordering::Greater => self.current_frame - 1,
            std::cmp::Ordering::Equal => self.current_frame,
        };
    }

    fn draw(&self, texture: &Texture2D, alpha: f32) {
        let draw_position = self.previous_position.lerp(self.position, alpha);
        let source = Rect::new(
            self.frame_size.x * self.current_frame as f32,
            0.0,
            self.frame_size.x,
            self.frame_size.y,
        );

        draw_texture_ex(
            texture,
            draw_position.x,
            draw_position.y,
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
    previous_offset: f32,
    offset: f32,
    tile_height: f32,
}

impl ScrollingBackground {
    fn new(tile_height: f32) -> Self {
        Self {
            previous_offset: 0.0,
            offset: 0.0,
            tile_height,
        }
    }

    fn fixed_update(&mut self) {
        self.previous_offset = self.offset;
        self.offset = (self.offset + BACKGROUND_SCROLL_SPEED * FIXED_DT) % self.tile_height;
    }

    fn draw(&self, texture: &Texture2D, alpha: f32) {
        let offset = lerp_wrapped(self.previous_offset, self.offset, alpha, self.tile_height);
        let tile_width = texture.width();
        let columns = (screen_width() / tile_width).ceil() as i32 + 1;
        let rows = (screen_height() / self.tile_height).ceil() as i32 + 2;

        for row in -1..rows {
            for column in 0..columns {
                draw_texture(
                    texture,
                    column as f32 * tile_width,
                    row as f32 * self.tile_height + offset,
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

fn lerp_wrapped(previous: f32, current: f32, alpha: f32, wrap: f32) -> f32 {
    let mut delta = current - previous;
    if delta < -wrap * 0.5 {
        delta += wrap;
    }
    (previous + delta * alpha) % wrap
}
