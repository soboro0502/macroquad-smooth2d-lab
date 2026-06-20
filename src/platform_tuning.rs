pub fn set_latency_sensitive_thread() -> ThreadTuningResult {
    imp::set_latency_sensitive_thread()
}

pub fn set_time_constraint_thread(
    period_secs: f64,
    computation_secs: f64,
    constraint_secs: f64,
) -> ThreadTuningResult {
    imp::set_time_constraint_thread(period_secs, computation_secs, constraint_secs)
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum ThreadTuningResult {
    Applied,
    Failed(i32),
    Unsupported,
}

#[cfg(target_os = "macos")]
mod imp {
    use super::ThreadTuningResult;

    pub fn set_latency_sensitive_thread() -> ThreadTuningResult {
        // Tell macOS this thread is latency-sensitive UI/game work.
        let result = unsafe {
            libc::pthread_set_qos_class_self_np(libc::qos_class_t::QOS_CLASS_USER_INTERACTIVE, 0)
        };
        if result == 0 {
            ThreadTuningResult::Applied
        } else {
            ThreadTuningResult::Failed(result)
        }
    }

    #[allow(deprecated)]
    pub fn set_time_constraint_thread(
        period_secs: f64,
        computation_secs: f64,
        constraint_secs: f64,
    ) -> ThreadTuningResult {
        // Mach time-constraint policy is the macOS-specific part that removed
        // rare 10-11 ms presentation spikes in this test project.
        let mut policy = libc::thread_time_constraint_policy {
            period: seconds_to_ticks(period_secs),
            computation: seconds_to_ticks(computation_secs),
            constraint: seconds_to_ticks(constraint_secs),
            preemptible: 0,
        };
        let thread = unsafe { libc::pthread_mach_thread_np(libc::pthread_self()) };
        let result = unsafe {
            libc::thread_policy_set(
                thread,
                libc::THREAD_TIME_CONSTRAINT_POLICY as libc::thread_policy_flavor_t,
                (&mut policy as *mut libc::thread_time_constraint_policy).cast::<libc::integer_t>(),
                libc::THREAD_TIME_CONSTRAINT_POLICY_COUNT,
            )
        };

        if result == libc::KERN_SUCCESS {
            ThreadTuningResult::Applied
        } else {
            ThreadTuningResult::Failed(result)
        }
    }

    #[allow(deprecated)]
    fn seconds_to_ticks(seconds: f64) -> u32 {
        let nanos = (seconds * 1_000_000_000.0).max(0.0) as u128;
        let mut info = libc::mach_timebase_info_data_t { numer: 1, denom: 1 };
        let result = unsafe { libc::mach_timebase_info(&mut info) };
        if result != libc::KERN_SUCCESS || info.numer == 0 || info.denom == 0 {
            return nanos.min(u128::from(u32::MAX)) as u32;
        }
        ((nanos * u128::from(info.denom)) / u128::from(info.numer)).min(u128::from(u32::MAX)) as u32
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    use super::ThreadTuningResult;

    pub fn set_latency_sensitive_thread() -> ThreadTuningResult {
        ThreadTuningResult::Unsupported
    }

    pub fn set_time_constraint_thread(
        _period_secs: f64,
        _computation_secs: f64,
        _constraint_secs: f64,
    ) -> ThreadTuningResult {
        ThreadTuningResult::Unsupported
    }
}
