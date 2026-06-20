use crate::config::*;
use crate::game::{BackgroundMode, TimingMode};

#[derive(Clone, Copy)]
pub struct AppOptions {
    pub diag_seconds: Option<f64>,
    pub diag_warmup_seconds: f64,
    pub clear_only: bool,
    pub manual_pacer_enabled: bool,
    pub pacer_mode: PacerMode,
    pub pacer_sleep_margin_secs: f64,
    pub pacer_sleep_threshold_secs: f64,
    pub time_constraint_enabled: bool,
    pub hud_visible: bool,
    pub timing_mode: TimingMode,
    pub background_mode: BackgroundMode,
    pub background_frame_step: f32,
}

#[derive(Clone, Copy)]
pub enum PacerMode {
    SleepSpin,
    Spin,
}

impl PacerMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::SleepSpin => "SLEEP",
            Self::Spin => "SPIN",
        }
    }
}

impl AppOptions {
    pub fn from_args(mut args: impl Iterator<Item = String>) -> Self {
        let mut options = Self {
            diag_seconds: None,
            diag_warmup_seconds: DEFAULT_DIAG_WARMUP_SECONDS,
            clear_only: false,
            manual_pacer_enabled: DEFAULT_MANUAL_PACER_ENABLED,
            pacer_mode: PacerMode::Spin,
            pacer_sleep_margin_secs: PACER_SLEEP_MARGIN_SECS,
            pacer_sleep_threshold_secs: PACER_SLEEP_THRESHOLD_SECS,
            time_constraint_enabled: DEFAULT_TIME_CONSTRAINT_ENABLED,
            hud_visible: false,
            timing_mode: TimingMode::FrameStep,
            background_mode: BackgroundMode::Texture,
            background_frame_step: DEFAULT_BACKGROUND_STEP,
        };

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--diag" => {
                    options.diag_seconds = Some(DEFAULT_DIAG_SECONDS);
                }
                "--diag-seconds" => {
                    options.diag_seconds = args
                        .next()
                        .and_then(|seconds| seconds.parse::<f64>().ok())
                        .filter(|seconds| *seconds > 0.0);
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
                "--sleep-pacer" => {
                    options.manual_pacer_enabled = true;
                    options.pacer_mode = PacerMode::SleepSpin;
                }
                "--pacer-margin-ms" => {
                    options.pacer_sleep_margin_secs = args
                        .next()
                        .and_then(|margin| margin.parse::<f64>().ok())
                        .filter(|margin| *margin >= 0.0)
                        .map(|margin| margin / 1000.0)
                        .unwrap_or(options.pacer_sleep_margin_secs);
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
                    options.hud_visible = true;
                    options.timing_mode = TimingMode::FrameStep;
                    options.background_mode = BackgroundMode::Stripes;
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
                _ => {}
            }
        }

        options
    }
}
