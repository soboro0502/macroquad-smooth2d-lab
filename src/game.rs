use macroquad::prelude::*;

use crate::config::*;

pub struct Assets {
    pub background: Texture2D,
    pub probe_background: Texture2D,
    pub player: Texture2D,
    pub font: Font,
}

impl Assets {
    pub async fn load() -> Self {
        let background = load_texture(ASSET_BG_TEST)
            .await
            .expect("failed to load test background");
        let probe_background = load_texture(ASSET_BG_PROBE)
            .await
            .expect("failed to load probe background");
        let player = load_texture(ASSET_PLAYER)
            .await
            .expect("failed to load player sprite sheet");
        let mut font = load_ttf_font(ASSET_FONT)
            .await
            .expect("failed to load HUD font");

        background.set_filter(FilterMode::Nearest);
        probe_background.set_filter(FilterMode::Nearest);
        player.set_filter(FilterMode::Nearest);
        font.set_filter(FilterMode::Nearest);

        Self {
            background,
            probe_background,
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
    pub toggle_timing_mode: bool,
    pub toggle_background_mode: bool,
    pub selected_background_step: Option<f32>,
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
            toggle_timing_mode: is_key_pressed(KeyCode::Tab),
            toggle_background_mode: is_key_pressed(KeyCode::G),
            selected_background_step: selected_background_step(),
        }
    }
}

fn selected_background_step() -> Option<f32> {
    if is_key_pressed(KeyCode::Key1) {
        Some(BACKGROUND_STEP_1)
    } else if is_key_pressed(KeyCode::Key2) {
        Some(BACKGROUND_STEP_2)
    } else if is_key_pressed(KeyCode::Key3) {
        Some(BACKGROUND_STEP_3)
    } else if is_key_pressed(KeyCode::Key4) {
        Some(BACKGROUND_STEP_4)
    } else {
        None
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimingMode {
    DeltaTime,
    FrameStep,
}

impl TimingMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::DeltaTime => "DT",
            Self::FrameStep => "FRAME",
        }
    }

    fn toggled(self) -> Self {
        match self {
            Self::DeltaTime => Self::FrameStep,
            Self::FrameStep => Self::DeltaTime,
        }
    }
}

pub struct Game {
    player: Player,
    background: ScrollingBackground,
    timing_mode: TimingMode,
    background_mode: BackgroundMode,
}

impl Game {
    pub fn new(assets: &Assets) -> Self {
        let source_size = player_frame_size(&assets.player);
        let draw_size = source_size * PLAYER_DRAW_SCALE;
        let start = vec2(
            screen_width() * 0.5 - draw_size.x * 0.5,
            screen_height() * 0.75 - draw_size.y * 0.5,
        );

        Self {
            player: Player::new(start, source_size),
            background: ScrollingBackground::new(
                assets.background.height() * BACKGROUND_DRAW_SCALE,
            ),
            timing_mode: TimingMode::FrameStep,
            background_mode: BackgroundMode::Texture,
        }
    }

    pub fn update(&mut self, input: InputState, dt: f32) {
        if input.toggle_timing_mode {
            self.timing_mode = self.timing_mode.toggled();
        }
        if input.toggle_background_mode {
            self.background_mode = self.background_mode.toggled();
        }
        if input.toggle_scroll {
            self.background.toggle();
        }
        if let Some(frame_step) = input.selected_background_step {
            self.background.set_frame_step(frame_step);
        }
        self.background.update(self.timing_mode, dt);
        self.player.update(input, self.timing_mode, dt);
    }

    pub fn draw(&self, assets: &Assets) {
        self.background.draw(
            &assets.background,
            &assets.probe_background,
            self.background_mode,
        );
        self.player.draw(&assets.player);
    }

    pub fn scroll_enabled(&self) -> bool {
        self.background.enabled
    }

    pub fn timing_mode(&self) -> TimingMode {
        self.timing_mode
    }

    pub fn background_mode(&self) -> BackgroundMode {
        self.background_mode
    }

    pub fn background_frame_step(&self) -> f32 {
        self.background.frame_step()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackgroundMode {
    Texture,
    ProbeTexture,
    Stripes,
}

impl BackgroundMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Texture => "TEX",
            Self::ProbeTexture => "PROBE",
            Self::Stripes => "BANDS",
        }
    }

    fn toggled(self) -> Self {
        match self {
            Self::Texture => Self::ProbeTexture,
            Self::ProbeTexture => Self::Stripes,
            Self::Stripes => Self::Texture,
        }
    }
}

struct Player {
    position: Vec2,
    source_size: Vec2,
    draw_size: Vec2,
    current_frame: usize,
}

impl Player {
    fn new(position: Vec2, source_size: Vec2) -> Self {
        Self {
            position,
            source_size,
            draw_size: source_size * PLAYER_DRAW_SCALE,
            current_frame: PLAYER_CENTER_FRAME,
        }
    }

    fn update(&mut self, input: InputState, timing_mode: TimingMode, dt: f32) {
        let speed = if input.slow {
            PLAYER_SLOW_SPEED
        } else {
            PLAYER_SPEED
        };
        let frame_step = if input.slow {
            FRAME_STEP_PLAYER_SLOW_PIXELS
        } else {
            FRAME_STEP_PLAYER_PIXELS
        };
        let distance = match timing_mode {
            TimingMode::DeltaTime => speed * dt,
            TimingMode::FrameStep => frame_step,
        };
        self.position += input.axis * distance;
        self.position.x = self
            .position
            .x
            .clamp(0.0, screen_width() - self.draw_size.x);
        self.position.y = self
            .position
            .y
            .clamp(0.0, screen_height() - self.draw_size.y);
        self.current_frame = target_frame_for_axis(input.axis.x);
    }

    fn draw(&self, texture: &Texture2D) {
        let source = Rect::new(
            self.source_size.x * self.current_frame as f32,
            0.0,
            self.source_size.x,
            self.source_size.y,
        );

        draw_texture_ex(
            texture,
            self.position.x,
            self.position.y,
            WHITE,
            DrawTextureParams {
                source: Some(source),
                dest_size: Some(self.draw_size),
                ..Default::default()
            },
        );
    }
}

struct ScrollingBackground {
    offset: f32,
    tile_height: f32,
    enabled: bool,
    frame_step: f32,
}

impl ScrollingBackground {
    fn new(tile_height: f32) -> Self {
        Self {
            offset: 0.0,
            tile_height,
            enabled: true,
            frame_step: DEFAULT_BACKGROUND_STEP,
        }
    }

    fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    fn set_frame_step(&mut self, frame_step: f32) {
        self.frame_step = frame_step;
    }

    fn frame_step(&self) -> f32 {
        self.frame_step
    }

    fn update(&mut self, timing_mode: TimingMode, dt: f32) {
        if self.enabled {
            let distance = match timing_mode {
                TimingMode::DeltaTime => BACKGROUND_SCROLL_SPEED * dt,
                TimingMode::FrameStep => self.frame_step,
            };
            self.offset += distance;
        }
    }

    fn draw(&self, texture: &Texture2D, probe_texture: &Texture2D, mode: BackgroundMode) {
        match mode {
            BackgroundMode::Texture => self.draw_texture_tiles(texture),
            BackgroundMode::ProbeTexture => self.draw_texture_tiles(probe_texture),
            BackgroundMode::Stripes => self.draw_stripes(),
        }
    }

    fn draw_texture_tiles(&self, texture: &Texture2D) {
        let tile_width = texture.width() * BACKGROUND_DRAW_SCALE;
        let tile_size = vec2(tile_width, self.tile_height);
        let columns = (screen_width() / tile_width).ceil() as i32 + 1;
        let rows = (screen_height() / self.tile_height).ceil() as i32 + 2;
        let offset = self.offset.rem_euclid(self.tile_height);

        for row in -1..rows {
            for column in 0..columns {
                draw_texture_ex(
                    texture,
                    column as f32 * tile_width,
                    row as f32 * self.tile_height + offset,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(tile_size),
                        ..Default::default()
                    },
                );
            }
        }
    }

    fn draw_stripes(&self) {
        let cycle = PROBE_BAND_HEIGHT + PROBE_BAND_GAP;
        let rows = (screen_height() / cycle).ceil() as i32 + 2;
        let offset = self.offset.rem_euclid(cycle);

        for row in -1..rows {
            let y = row as f32 * cycle + offset;
            draw_rectangle(
                0.0,
                y,
                screen_width(),
                PROBE_BAND_HEIGHT,
                Color::new(0.10, 0.13, 0.20, 1.0),
            );
            draw_rectangle(
                0.0,
                y,
                screen_width(),
                PROBE_MARKER_HEIGHT,
                Color::new(0.35, 0.72, 0.95, 1.0),
            );
            draw_rectangle(
                screen_width() * 0.5 - 12.0,
                y,
                24.0,
                PROBE_BAND_HEIGHT,
                Color::new(0.90, 0.55, 0.20, 1.0),
            );
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
