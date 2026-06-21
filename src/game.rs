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
    pub toggle_diagonal_mode: bool,
    pub toggle_background_mode: bool,
    pub increase_player_speed: bool,
    pub decrease_player_speed: bool,
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
        Self {
            axis,
            slow: is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift),
            toggle_scroll: is_key_pressed(KeyCode::Space),
            toggle_timing_mode: is_key_pressed(KeyCode::Tab),
            toggle_diagonal_mode: is_key_pressed(KeyCode::V),
            toggle_background_mode: is_key_pressed(KeyCode::G),
            increase_player_speed: is_key_pressed(KeyCode::X),
            decrease_player_speed: is_key_pressed(KeyCode::Z),
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagonalMode {
    Normalized,
    Raw,
    AxisLock,
}

impl DiagonalMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Normalized => "NORM",
            Self::Raw => "RAW",
            Self::AxisLock => "LOCK",
        }
    }

    fn toggled(self) -> Self {
        match self {
            Self::Normalized => Self::Raw,
            Self::Raw => Self::AxisLock,
            Self::AxisLock => Self::Normalized,
        }
    }

    fn apply(self, axis: Vec2) -> Vec2 {
        if axis.length_squared() <= 1.0 {
            return axis;
        }

        match self {
            Self::Normalized => axis.normalize(),
            Self::Raw => axis,
            Self::AxisLock => {
                if axis.y != 0.0 {
                    vec2(0.0, axis.y)
                } else {
                    axis
                }
            }
        }
    }
}

pub struct Game {
    player: Player,
    background: ScrollingBackground,
    timing_mode: TimingMode,
    diagonal_mode: DiagonalMode,
    background_mode: BackgroundMode,
    target_refresh_hz: u32,
    player_speed_scale: f32,
}

impl Game {
    pub fn new(assets: &Assets, target_refresh_hz: u32) -> Self {
        let source_size = player_frame_size(&assets.player);
        let draw_size = source_size * PLAYER_DRAW_SCALE;
        let start = vec2(
            screen_width() * 0.5 - draw_size.x * 0.5,
            screen_height() * 0.75 - draw_size.y * 0.5,
        );

        Self {
            player: Player::new(start, source_size),
            background: ScrollingBackground::new(),
            timing_mode: TimingMode::FrameStep,
            diagonal_mode: DiagonalMode::Normalized,
            background_mode: BackgroundMode::Texture,
            target_refresh_hz,
            player_speed_scale: 1.0,
        }
    }

    pub fn set_timing_mode(&mut self, timing_mode: TimingMode) {
        self.timing_mode = timing_mode;
    }

    pub fn set_background_mode(&mut self, background_mode: BackgroundMode) {
        self.background_mode = background_mode;
    }

    pub fn set_background_frame_step(&mut self, frame_step: f32) {
        self.background.set_frame_step(frame_step);
    }

    pub fn update(&mut self, input: InputState, dt: f32) {
        if input.toggle_timing_mode {
            self.timing_mode = self.timing_mode.toggled();
        }
        if input.toggle_diagonal_mode {
            self.diagonal_mode = self.diagonal_mode.toggled();
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
        if input.decrease_player_speed {
            self.player_speed_scale =
                (self.player_speed_scale - PLAYER_SPEED_SCALE_STEP).max(PLAYER_SPEED_SCALE_MIN);
        }
        if input.increase_player_speed {
            self.player_speed_scale =
                (self.player_speed_scale + PLAYER_SPEED_SCALE_STEP).min(PLAYER_SPEED_SCALE_MAX);
        }
        let frame_scale = frame_step_scale(self.target_refresh_hz);
        self.background.update(self.timing_mode, dt, frame_scale);
        self.player.update(
            input,
            self.timing_mode,
            dt,
            frame_scale,
            self.player_speed_scale,
            self.diagonal_mode,
        );
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

    pub fn diagonal_mode(&self) -> DiagonalMode {
        self.diagonal_mode
    }

    pub fn background_mode(&self) -> BackgroundMode {
        self.background_mode
    }

    pub fn background_frame_step(&self) -> f32 {
        self.background.frame_step()
    }

    pub fn background_last_delta(&self) -> f32 {
        self.background.last_delta()
    }

    pub fn player_speed_scale(&self) -> f32 {
        self.player_speed_scale
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
    lean_frame_elapsed: f32,
}

impl Player {
    fn new(position: Vec2, source_size: Vec2) -> Self {
        Self {
            position,
            source_size,
            draw_size: source_size * PLAYER_DRAW_SCALE,
            current_frame: PLAYER_CENTER_FRAME,
            lean_frame_elapsed: 0.0,
        }
    }

    fn update(
        &mut self,
        input: InputState,
        timing_mode: TimingMode,
        dt: f32,
        frame_scale: f32,
        speed_scale: f32,
        diagonal_mode: DiagonalMode,
    ) {
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
            TimingMode::DeltaTime => speed * speed_scale * dt,
            TimingMode::FrameStep => frame_step * speed_scale * frame_scale,
        };
        self.position += diagonal_mode.apply(input.axis) * distance;
        self.position.x = self
            .position
            .x
            .clamp(0.0, screen_width() - self.draw_size.x);
        self.position.y = self
            .position
            .y
            .clamp(0.0, screen_height() - self.draw_size.y);
        self.update_lean_frame(target_frame_for_axis(input.axis.x), dt);
    }

    fn update_lean_frame(&mut self, target_frame: usize, dt: f32) {
        if target_frame == self.current_frame {
            self.lean_frame_elapsed = 0.0;
            return;
        }

        self.lean_frame_elapsed += dt;
        while self.lean_frame_elapsed >= PLAYER_LEAN_FRAME_SECONDS
            && self.current_frame != target_frame
        {
            self.lean_frame_elapsed -= PLAYER_LEAN_FRAME_SECONDS;
            self.current_frame = if self.current_frame < target_frame {
                self.current_frame + 1
            } else {
                self.current_frame - 1
            };
        }
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
    last_delta: f32,
    enabled: bool,
    frame_step: f32,
}

impl ScrollingBackground {
    fn new() -> Self {
        Self {
            offset: 0.0,
            last_delta: 0.0,
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

    fn last_delta(&self) -> f32 {
        self.last_delta
    }

    fn update(&mut self, timing_mode: TimingMode, dt: f32, frame_scale: f32) {
        self.last_delta = 0.0;
        if self.enabled {
            let distance = match timing_mode {
                TimingMode::DeltaTime => BACKGROUND_SCROLL_SPEED * dt,
                TimingMode::FrameStep => self.frame_step * frame_scale,
            };
            self.offset += distance;
            self.last_delta = distance;
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
        let tile_height = texture.height() * BACKGROUND_DRAW_SCALE;
        let tile_size = vec2(tile_width, tile_height);
        let offset = self.offset.rem_euclid(tile_height);
        if tile_width >= screen_width() && tile_height >= screen_height() {
            self.draw_large_texture_wrap(texture, tile_width, tile_height, offset);
            return;
        }

        let columns = (screen_width() / tile_width).ceil() as i32;
        let rows = (screen_height() / tile_height).ceil() as i32 + 1;

        for row in -1..rows {
            for column in 0..columns {
                draw_texture_ex(
                    texture,
                    column as f32 * tile_width,
                    row as f32 * tile_height + offset,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(tile_size),
                        ..Default::default()
                    },
                );
            }
        }
    }

    fn draw_large_texture_wrap(
        &self,
        texture: &Texture2D,
        tile_width: f32,
        tile_height: f32,
        offset: f32,
    ) {
        let tile_size = vec2(tile_width, tile_height);
        for row in -1..1 {
            draw_texture_ex(
                texture,
                0.0,
                row as f32 * tile_height + offset,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(tile_size),
                    ..Default::default()
                },
            );
        }
    }

    fn draw_stripes(&self) {
        self.draw_fixed_guides();

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

        draw_rectangle(
            0.0,
            offset,
            screen_width(),
            PROBE_SCROLL_LINE_THICKNESS,
            PROBE_SCROLL_LINE_COLOR,
        );
    }

    fn draw_fixed_guides(&self) {
        let guide_count = (screen_height() / PROBE_GUIDE_INTERVAL).ceil() as i32 + 1;
        for guide in 0..guide_count {
            let y = guide as f32 * PROBE_GUIDE_INTERVAL;
            draw_rectangle(
                0.0,
                y,
                screen_width(),
                PROBE_GUIDE_THICKNESS,
                PROBE_GUIDE_COLOR,
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

fn frame_step_scale(target_refresh_hz: u32) -> f32 {
    REFERENCE_GAME_HZ / target_refresh_hz.max(1) as f32
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
