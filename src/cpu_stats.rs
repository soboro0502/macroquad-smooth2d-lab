use macroquad::prelude::get_time;

pub struct CpuStats {
    last_wall_secs: f64,
    last_cpu_secs: f64,
    publish_timer_secs: f32,
    sample_interval_secs: f32,
    pub percent: f32,
}

impl CpuStats {
    pub fn new(sample_interval_secs: f32) -> Self {
        Self {
            last_wall_secs: get_time(),
            last_cpu_secs: process_cpu_secs(),
            publish_timer_secs: 0.0,
            sample_interval_secs,
            percent: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.publish_timer_secs += dt;
        if self.publish_timer_secs < self.sample_interval_secs {
            return;
        }

        self.publish_timer_secs = 0.0;
        let wall_secs = get_time();
        let cpu_secs = process_cpu_secs();
        let wall_delta = wall_secs - self.last_wall_secs;
        let cpu_delta = cpu_secs - self.last_cpu_secs;
        if wall_delta > 0.0 && cpu_delta >= 0.0 {
            self.percent = (cpu_delta / wall_delta * 100.0) as f32;
        }
        self.last_wall_secs = wall_secs;
        self.last_cpu_secs = cpu_secs;
    }
}

fn process_cpu_secs() -> f64 {
    let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
    let result = unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) };
    if result != 0 {
        return 0.0;
    }

    let usage = unsafe { usage.assume_init() };
    timeval_secs(usage.ru_utime) + timeval_secs(usage.ru_stime)
}

fn timeval_secs(timeval: libc::timeval) -> f64 {
    timeval.tv_sec as f64 + timeval.tv_usec as f64 / 1_000_000.0
}
