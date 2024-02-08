use crate::common::{TimeDiff, Timestamp};

#[derive(Debug, Clone, Copy)]
pub struct Clock {
    current_time: Timestamp,
    time_per_tick: TimeDiff,
}

impl Clock {
    pub const fn new(time_per_tick: TimeDiff) -> Self {
        Clock {
            current_time: 0,
            time_per_tick,
        }
    }

    #[inline]
    pub fn current_time(&self) -> Timestamp {
        self.current_time
    }

    #[inline]
    pub fn next_tick(&self) -> Timestamp {
        self.current_time + self.time_per_tick
    }

    #[inline]
    pub fn after_ticks(&self, ticks: i64) -> Timestamp {
        self.current_time + self.time_per_tick * ticks
    }

    #[inline]
    pub fn advance(&mut self, ticks: i64) {
        self.current_time += self.time_per_tick * ticks;
    }
}
