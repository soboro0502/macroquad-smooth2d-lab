use std::thread;
use std::time::Duration;

use macroquad::prelude::get_time;

#[derive(Clone, Copy, Default)]
pub struct PacerSample {
    pub os_wait_secs: f64,
    pub spin_secs: f64,
    pub total_wait_secs: f64,
}

impl PacerSample {
    pub fn os_wait_ms(self) -> f32 {
        (self.os_wait_secs * 1000.0) as f32
    }

    pub fn spin_ms(self) -> f32 {
        (self.spin_secs * 1000.0) as f32
    }

    pub fn total_wait_ms(self) -> f32 {
        (self.total_wait_secs * 1000.0) as f32
    }
}

pub struct FramePacer;

impl Default for FramePacer {
    fn default() -> Self {
        Self::new()
    }
}

impl FramePacer {
    pub fn new() -> Self {
        Self
    }

    pub fn wait_until(
        &self,
        frame_start: f64,
        hz: u32,
        sleep_margin_secs: f64,
        sleep_threshold_secs: f64,
    ) -> PacerSample {
        let total_start = get_time();
        let mut sample = PacerSample::default();
        let deadline = frame_start + 1.0 / f64::from(hz.max(1));
        loop {
            let now = get_time();
            if now >= deadline {
                sample.total_wait_secs = get_time() - total_start;
                return sample;
            }

            let remaining = deadline - now;
            let sleep_for = remaining - sleep_margin_secs;
            if remaining > sleep_threshold_secs && sleep_for > 0.0 {
                let wait_start = get_time();
                thread::sleep(Duration::from_secs_f64(sleep_for));
                sample.os_wait_secs += get_time() - wait_start;
            } else {
                let spin_start = get_time();
                while get_time() < deadline {
                    std::hint::spin_loop();
                }
                sample.spin_secs += get_time() - spin_start;
                sample.total_wait_secs = get_time() - total_start;
                return sample;
            }
        }
    }

    pub fn spin_until(&self, frame_start: f64, hz: u32) -> PacerSample {
        let spin_start = get_time();
        let deadline = frame_start + 1.0 / f64::from(hz.max(1));
        while get_time() < deadline {
            std::hint::spin_loop();
        }
        let spin_secs = get_time() - spin_start;
        PacerSample {
            os_wait_secs: 0.0,
            spin_secs,
            total_wait_secs: spin_secs,
        }
    }

    pub fn sleep_for(&self, seconds: f64) -> PacerSample {
        let total_start = get_time();
        let wait_start = get_time();
        thread::sleep(Duration::from_secs_f64(seconds.max(0.0)));
        let os_wait_secs = get_time() - wait_start;
        PacerSample {
            os_wait_secs,
            spin_secs: 0.0,
            total_wait_secs: get_time() - total_start,
        }
    }

    pub fn mach_wait_spin_until(
        &self,
        frame_start: f64,
        hz: u32,
        spin_margin_secs: f64,
    ) -> PacerSample {
        imp::mach_wait_spin_until(frame_start, hz, spin_margin_secs)
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use std::sync::OnceLock;

    use macroquad::prelude::get_time;

    use super::PacerSample;

    unsafe extern "C" {
        fn mach_wait_until(deadline: u64) -> libc::kern_return_t;
    }

    #[allow(deprecated)]
    pub fn mach_wait_spin_until(frame_start: f64, hz: u32, spin_margin_secs: f64) -> PacerSample {
        let total_start = get_time();
        let mut sample = PacerSample::default();
        let deadline = frame_start + 1.0 / f64::from(hz.max(1));
        let wait_until = deadline - spin_margin_secs.max(0.0);
        let remaining = wait_until - get_time();
        if remaining > 0.0 {
            let mach_deadline =
                unsafe { libc::mach_absolute_time() }.saturating_add(seconds_to_ticks(remaining));
            let wait_start = get_time();
            let _ = unsafe { mach_wait_until(mach_deadline) };
            sample.os_wait_secs = get_time() - wait_start;
        }

        let spin_start = get_time();
        while get_time() < deadline {
            std::hint::spin_loop();
        }
        sample.spin_secs = get_time() - spin_start;
        sample.total_wait_secs = get_time() - total_start;
        sample
    }

    #[allow(deprecated)]
    fn seconds_to_ticks(seconds: f64) -> u64 {
        let nanos = (seconds * 1_000_000_000.0).max(0.0) as u128;
        let (numer, denom) = *TIMEBASE.get_or_init(load_timebase);
        ((nanos * denom) / numer).min(u128::from(u64::MAX)) as u64
    }

    static TIMEBASE: OnceLock<(u128, u128)> = OnceLock::new();

    #[allow(deprecated)]
    fn load_timebase() -> (u128, u128) {
        let mut info = libc::mach_timebase_info_data_t { numer: 1, denom: 1 };
        let result = unsafe { libc::mach_timebase_info(&mut info) };
        if result != libc::KERN_SUCCESS || info.numer == 0 || info.denom == 0 {
            return (1, 1);
        }
        (u128::from(info.numer), u128::from(info.denom))
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    use std::thread;
    use std::time::Duration;

    use macroquad::prelude::get_time;

    use super::PacerSample;

    pub fn mach_wait_spin_until(frame_start: f64, hz: u32, spin_margin_secs: f64) -> PacerSample {
        let total_start = get_time();
        let mut sample = PacerSample::default();
        let deadline = frame_start + 1.0 / f64::from(hz.max(1));
        let wait_until = deadline - spin_margin_secs.max(0.0);
        let remaining = wait_until - get_time();
        if remaining > 0.0 {
            let wait_start = get_time();
            thread::sleep(Duration::from_secs_f64(remaining));
            sample.os_wait_secs = get_time() - wait_start;
        }

        let spin_start = get_time();
        while get_time() < deadline {
            std::hint::spin_loop();
        }
        sample.spin_secs = get_time() - spin_start;
        sample.total_wait_secs = get_time() - total_start;
        sample
    }
}
