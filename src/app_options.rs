use crate::game::{BackgroundMode, TimingMode};
use macroquad_smooth2d_lab::config::*;

#[derive(Clone, Copy)]
pub struct AppOptions {
    pub profile: RuntimeProfile,
    pub target_refresh_hz: u32,
    pub startup_warmup_seconds: f64,
    pub diag_seconds: Option<f64>,
    pub diag_warmup_seconds: f64,
    pub clear_only: bool,
    pub manual_pacer_enabled: bool,
    pub pacer_mode: PacerMode,
    pub pacer_spin_margin_secs: f64,
    pub pacer_sleep_margin_secs: f64,
    pub pacer_sleep_threshold_secs: f64,
    pub time_constraint_enabled: bool,
    pub hud_visible: bool,
    pub timing_mode: TimingMode,
    pub background_mode: BackgroundMode,
    pub background_frame_step: f32,
    pub stress_sprite_count: usize,
}

#[derive(Clone, Copy)]
pub enum RuntimeProfile {
    Stable60,
    Smooth120,
    Custom,
}

impl RuntimeProfile {
    pub fn target_refresh_hz(self) -> u32 {
        match self {
            Self::Stable60 => 60,
            Self::Smooth120 | Self::Custom => DEFAULT_TARGET_REFRESH_HZ_U32,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Stable60 => "Stable60",
            Self::Smooth120 => "Smooth120",
            Self::Custom => "Custom",
        }
    }

    fn from_label(label: &str) -> Option<Self> {
        match label {
            "stable60" | "stable-60" | "60" => Some(Self::Stable60),
            "smooth120" | "smooth-120" | "120" => Some(Self::Smooth120),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub enum PacerMode {
    MachSpin,
    SleepSpin,
    Spin,
}

impl PacerMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::MachSpin => "MACH",
            Self::SleepSpin => "SLEEP",
            Self::Spin => "SPIN",
        }
    }
}

impl AppOptions {
    pub fn from_args(mut args: impl Iterator<Item = String>) -> Self {
        let default_profile = RuntimeProfile::Smooth120;
        let mut options = Self {
            profile: default_profile,
            target_refresh_hz: default_profile.target_refresh_hz(),
            startup_warmup_seconds: DEFAULT_STARTUP_WARMUP_SECONDS,
            diag_seconds: None,
            diag_warmup_seconds: DEFAULT_DIAG_WARMUP_SECONDS,
            clear_only: false,
            manual_pacer_enabled: DEFAULT_MANUAL_PACER_ENABLED,
            pacer_mode: PacerMode::MachSpin,
            pacer_spin_margin_secs: PACER_BALANCED_SPIN_MARGIN_SECS,
            pacer_sleep_margin_secs: PACER_SLEEP_MARGIN_SECS,
            pacer_sleep_threshold_secs: PACER_SLEEP_THRESHOLD_SECS,
            time_constraint_enabled: DEFAULT_TIME_CONSTRAINT_ENABLED,
            hud_visible: false,
            timing_mode: TimingMode::FrameStep,
            background_mode: BackgroundMode::Texture,
            background_frame_step: DEFAULT_BACKGROUND_STEP,
            stress_sprite_count: DEFAULT_STRESS_SPRITE_COUNT,
        };

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--diag" => {
                    options.diag_seconds = Some(DEFAULT_DIAG_SECONDS);
                }
                "--target-hz" => {
                    options.target_refresh_hz = args
                        .next()
                        .and_then(|hz| hz.parse::<u32>().ok())
                        .filter(|hz| *hz > 0)
                        .unwrap_or(options.target_refresh_hz);
                    options.profile = match options.target_refresh_hz {
                        60 => RuntimeProfile::Stable60,
                        120 => RuntimeProfile::Smooth120,
                        _ => RuntimeProfile::Custom,
                    };
                }
                "--profile" => {
                    if let Some(profile) = args
                        .next()
                        .and_then(|profile| RuntimeProfile::from_label(&profile))
                    {
                        options.profile = profile;
                        options.target_refresh_hz = profile.target_refresh_hz();
                    }
                }
                "--diag-seconds" => {
                    options.diag_seconds = args
                        .next()
                        .and_then(|seconds| seconds.parse::<f64>().ok())
                        .filter(|seconds| *seconds > 0.0);
                }
                "--startup-warmup-seconds" => {
                    options.startup_warmup_seconds = args
                        .next()
                        .and_then(|seconds| seconds.parse::<f64>().ok())
                        .filter(|seconds| *seconds >= 0.0)
                        .unwrap_or(options.startup_warmup_seconds);
                }
                "--no-startup-warmup" => {
                    options.startup_warmup_seconds = 0.0;
                }
                "--diag-warmup-seconds" => {
                    options.diag_warmup_seconds = args
                        .next()
                        .and_then(|seconds| seconds.parse::<f64>().ok())
                        .filter(|seconds| *seconds >= 0.0)
                        .unwrap_or(options.diag_warmup_seconds);
                }
                "--diag-no-warmup" => {
                    options.diag_warmup_seconds = 0.0;
                }
                "--diag-clear" => {
                    options.clear_only = true;
                }
                "--diag-manual" => {
                    options.manual_pacer_enabled = true;
                }
                "--diag-auto" => {
                    options.manual_pacer_enabled = false;
                }
                "--spin-pacer" => {
                    options.manual_pacer_enabled = true;
                    options.pacer_mode = PacerMode::Spin;
                }
                "--mach-pacer" => {
                    options.manual_pacer_enabled = true;
                    options.pacer_mode = PacerMode::MachSpin;
                }
                "--balanced-pacer" => {
                    options.manual_pacer_enabled = true;
                    options.pacer_mode = PacerMode::MachSpin;
                    options.pacer_spin_margin_secs = PACER_BALANCED_SPIN_MARGIN_SECS;
                }
                "--eco-pacer" => {
                    options.manual_pacer_enabled = true;
                    options.pacer_mode = PacerMode::MachSpin;
                    options.pacer_spin_margin_secs = PACER_ECO_SPIN_MARGIN_SECS;
                }
                "--precision-pacer" => {
                    options.manual_pacer_enabled = true;
                    options.pacer_mode = PacerMode::MachSpin;
                    options.pacer_spin_margin_secs = PACER_PRECISION_SPIN_MARGIN_SECS;
                }
                "--sleep-pacer" => {
                    options.manual_pacer_enabled = true;
                    options.pacer_mode = PacerMode::SleepSpin;
                }
                "--pacer-margin-ms" => {
                    let margin_secs = args
                        .next()
                        .and_then(|margin| margin.parse::<f64>().ok())
                        .filter(|margin| *margin >= 0.0)
                        .map(|margin| margin / 1000.0)
                        .unwrap_or(options.pacer_spin_margin_secs);
                    options.pacer_spin_margin_secs = margin_secs;
                    options.pacer_sleep_margin_secs = margin_secs;
                }
                "--pacer-sleep-threshold-ms" => {
                    options.pacer_sleep_threshold_secs = args
                        .next()
                        .and_then(|threshold| threshold.parse::<f64>().ok())
                        .filter(|threshold| *threshold >= 0.0)
                        .map(|threshold| threshold / 1000.0)
                        .unwrap_or(options.pacer_sleep_threshold_secs);
                }
                "--time-constraint" => {
                    options.time_constraint_enabled = true;
                }
                "--no-time-constraint" => {
                    options.time_constraint_enabled = false;
                }
                "--hud" => {
                    options.hud_visible = true;
                }
                "--visual-check" => {
                    options.timing_mode = TimingMode::FrameStep;
                    options.background_mode = BackgroundMode::Texture;
                    options.background_frame_step = DEFAULT_BACKGROUND_STEP;
                }
                "--texture" => {
                    options.background_mode = BackgroundMode::Texture;
                }
                "--probe" => {
                    options.background_mode = BackgroundMode::ProbeTexture;
                }
                "--bands" => {
                    options.background_mode = BackgroundMode::Stripes;
                }
                "--dt" => {
                    options.timing_mode = TimingMode::DeltaTime;
                }
                "--frame" => {
                    options.timing_mode = TimingMode::FrameStep;
                }
                "--bg-step" => {
                    options.background_frame_step = args
                        .next()
                        .and_then(|step| step.parse::<f32>().ok())
                        .filter(|step| *step > 0.0)
                        .unwrap_or(options.background_frame_step);
                }
                "--stress-sprites" | "--stress" => {
                    options.stress_sprite_count = args
                        .next()
                        .and_then(|count| count.parse::<usize>().ok())
                        .unwrap_or(options.stress_sprite_count);
                }
                _ => {}
            }
        }

        options
    }
}
