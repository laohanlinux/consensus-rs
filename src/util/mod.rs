// TimerRuntime - simplified for tokio (no actix)
use std::time::Duration;

pub struct TimerRuntime {
    pub timeout: Duration,
}

impl TimerRuntime {
    pub fn new(timeout: Duration) -> Self {
        TimerRuntime { timeout }
    }
}
