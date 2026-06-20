pub fn set_latency_sensitive_thread() -> ThreadTuningResult {
    imp::set_latency_sensitive_thread()
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
        let result = unsafe {
            libc::pthread_set_qos_class_self_np(libc::qos_class_t::QOS_CLASS_USER_INTERACTIVE, 0)
        };
        if result == 0 {
            ThreadTuningResult::Applied
        } else {
            ThreadTuningResult::Failed(result)
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    use super::ThreadTuningResult;

    pub fn set_latency_sensitive_thread() -> ThreadTuningResult {
        ThreadTuningResult::Unsupported
    }
}
