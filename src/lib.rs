//! Reusable frame pacing and diagnostics building blocks for Macroquad projects.
//!
//! This crate is still experimental. The current API exposes the pieces that
//! are already used by the bundled demo application: frame pacing, frame
//! statistics, CPU sampling, frame logging, platform thread tuning, and shared
//! timing constants.

pub mod config;
pub mod cpu_stats;
pub mod frame_log;
pub mod frame_pacer;
pub mod frame_stats;
pub mod platform_tuning;

pub mod prelude {
    //! Common imports for experiments using this crate.

    pub use crate::cpu_stats::CpuStats;
    pub use crate::frame_log::FrameLog;
    pub use crate::frame_pacer::{FramePacer, PacerSample};
    pub use crate::frame_stats::{
        FrameStats, FrameStatsSnapshot, RunFrameStats, RunValueStats, ValueStatsSnapshot,
    };
    pub use crate::platform_tuning::{
        set_latency_sensitive_thread, set_time_constraint_thread, ThreadTuningResult,
    };
}
