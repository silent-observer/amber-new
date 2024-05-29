use std::ops::{Add, AddAssign, Mul, Sub};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Timestamp(pub i64);

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TickTimestamp(pub i64);

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TimeDiff(pub i64);

impl Add<TimeDiff> for Timestamp {
    type Output = Timestamp;

    fn add(self, rhs: TimeDiff) -> Self::Output {
        Timestamp(self.0 + rhs.0)
    }
}

impl AddAssign<TimeDiff> for Timestamp {
    fn add_assign(&mut self, rhs: TimeDiff) {
        self.0 += rhs.0
    }
}

impl Add<i64> for TickTimestamp {
    type Output = TickTimestamp;

    fn add(self, rhs: i64) -> Self::Output {
        TickTimestamp(self.0 + rhs)
    }
}

impl AddAssign<i64> for TickTimestamp {
    fn add_assign(&mut self, rhs: i64) {
        self.0 += rhs
    }
}

impl Sub<TimeDiff> for Timestamp {
    type Output = Timestamp;

    fn sub(self, rhs: TimeDiff) -> Self::Output {
        Timestamp(self.0 - rhs.0)
    }
}

impl Sub<i64> for TickTimestamp {
    type Output = TickTimestamp;

    fn sub(self, rhs: i64) -> Self::Output {
        TickTimestamp(self.0 - rhs)
    }
}

impl Sub<TickTimestamp> for TickTimestamp {
    type Output = i64;

    fn sub(self, rhs: TickTimestamp) -> Self::Output {
        self.0 - rhs.0
    }
}

impl Sub<Timestamp> for Timestamp {
    type Output = TimeDiff;

    fn sub(self, rhs: Timestamp) -> Self::Output {
        TimeDiff(self.0 - rhs.0)
    }
}

impl Mul<i64> for TimeDiff {
    type Output = TimeDiff;

    fn mul(self, rhs: i64) -> Self::Output {
        TimeDiff(self.0 * rhs)
    }
}

impl Timestamp {
    pub const MAX: Timestamp = Timestamp(i64::MAX);
}

impl TickTimestamp {
    pub const MAX: TickTimestamp = TickTimestamp(i64::MAX);
}

#[derive(Debug, Clone, Copy)]
pub struct Clock {
    current_time: Timestamp,
    pub(super) current_tick: TickTimestamp,
    time_per_tick: TimeDiff,
}

impl Clock {
    pub const fn new(time_per_tick: TimeDiff) -> Self {
        Clock {
            current_time: Timestamp(0),
            current_tick: TickTimestamp(0),
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
        Timestamp(t.0 * self.time_per_tick.0)
    }

    #[inline]
    pub fn time_to_ticks(&self, t: Timestamp) -> TickTimestamp {
        TickTimestamp(t.0 / self.time_per_tick.0)
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
