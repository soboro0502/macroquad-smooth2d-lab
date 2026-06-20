use std::thread;
use std::time::Duration;

use macroquad::prelude::get_time;

use crate::config::PACER_SLEEP_THRESHOLD_SECS;

pub struct FramePacer;

impl FramePacer {
    pub fn new() -> Self {
        Self
    }

    pub fn wait_until(&self, frame_start: f64, hz: u32, sleep_margin_secs: f64) {
        let deadline = frame_start + 1.0 / f64::from(hz.max(1));
        loop {
            let now = get_time();
            if now >= deadline {
                return;
            }

            let remaining = deadline - now;
            let sleep_for = remaining - sleep_margin_secs;
            if remaining > PACER_SLEEP_THRESHOLD_SECS && sleep_for > 0.0 {
                thread::sleep(Duration::from_secs_f64(sleep_for));
            } else {
                std::hint::spin_loop();
            }
        }
    }

    pub fn spin_until(&self, frame_start: f64, hz: u32) {
        let deadline = frame_start + 1.0 / f64::from(hz.max(1));
        while get_time() < deadline {
            std::hint::spin_loop();
        }
    }
}
