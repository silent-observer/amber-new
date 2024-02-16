pub type Timestamp = i64;
pub type TickTimestamp = i64;
pub type TimeDiff = i64;

#[derive(Debug, Clone, Copy)]
pub struct Clock {
    current_time: Timestamp,
    current_tick: TickTimestamp,
    time_per_tick: TimeDiff,
}

impl Clock {
    pub const fn new(time_per_tick: TimeDiff) -> Self {
        Clock {
            current_time: 0,
            current_tick: 0,
            time_per_tick,
        }
    }

    #[inline]
    pub fn current_time(&self) -> Timestamp {
        self.current_time
    }

    #[inline]
    pub fn current_tick(&self) -> TickTimestamp {
        self.current_tick
    }

    #[inline]
    pub fn ticks_to_time(&self, t: TickTimestamp) -> Timestamp {
        t * self.time_per_tick
    }

    #[inline]
    pub fn time_to_ticks(&self, t: Timestamp) -> TickTimestamp {
        t / self.time_per_tick
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
        self.current_tick += ticks;
    }
}
