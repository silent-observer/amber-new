use kanal::Receiver;
use priority_queue::PriorityQueue;

use crate::{
    clock::{Clock, TimeDiff, Timestamp},
    module::Module,
    module_id::ModuleId,
    pin_state::WireState,
    wiring::{InboxTable, WiringTable},
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum EventData {
    None,
    WireState(WireState),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Event {
    pub receiver_id: ModuleId,
    pub data: EventData,
}

pub trait EventReceiver: Module {
    fn receive_event(&mut self, event: Event, queue: &mut EventQueue);
}

#[derive(Debug, Clone)]
pub struct EventQueue {
    pub clock: Clock,
    events: PriorityQueue<Event, Timestamp>,
    root_prefix: u8,
    receiver: Receiver<Event>,
}

impl EventQueue {
    pub fn new(ticks_per_cycle: TimeDiff, root_prefix: u8, receiver: Receiver<Event>) -> Self {
        Self {
            clock: Clock::new(ticks_per_cycle),
            events: PriorityQueue::new(),
            root_prefix,
            receiver,
        }
    }

    #[inline]
    pub fn fire_event(&mut self, event: Event, t: Timestamp) {
        self.events.push(event, t);
    }

    #[inline]
    pub fn fire_event_now(&mut self, event: Event) {
        self.fire_event(event, self.clock.current_time());
    }

    #[inline]
    pub fn fire_event_next_tick(&mut self, event: Event) {
        self.fire_event(event, self.clock.next_tick());
    }

    #[inline]
    pub fn fire_event_after_ticks(&mut self, event: Event, ticks: i64) {
        self.fire_event(event, self.clock.after_ticks(ticks));
    }

    #[inline]
    pub fn set_wire(&mut self, writer_id: ModuleId, state: WireState) {
        if let Some(readers) = WiringTable::get_connected(writer_id) {
            for &reader_id in readers {
                let e = Event {
                    receiver_id: reader_id,
                    data: EventData::WireState(state),
                };
                if reader_id.current() == self.root_prefix {
                    self.fire_event_now(e);
                } else {
                    InboxTable::send(e);
                }
            }
        }
    }

    pub fn update(&mut self, root: &mut impl EventReceiver) {
        loop {
            if let Ok(Some(event)) = self.receiver.try_recv() {
                root.receive_event(event, self);
                continue;
            }
            if let Some((&event, &t)) = self.events.peek() {
                if t <= self.clock.current_time() {
                    self.events.pop();
                    root.receive_event(event, self);
                    continue;
                }
            }
            break;
        }
    }
}
