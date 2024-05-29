use std::cmp::Reverse;

use kanal::Receiver;
use priority_queue::PriorityQueue;
use smallvec::SmallVec;

use crate::{
    clock::{Clock, TickTimestamp, TimeDiff, Timestamp},
    module::{Module, PinId},
    module_id::{EventPortAddress, ModuleAddress, PinAddress},
    multiplexer::MultiplexingTable,
    pin_state::WireState,
    wiring::InboxTable,
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct InternalEvent {
    pub receiver_id: EventPortAddress,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct WireChangeEvent {
    pub receiver_id: PinAddress,
    pub state: WireState,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct PinRedirect {
    pub main_pin: PinAddress,
    pub redirect_pin: PinAddress,
}

#[derive(Debug, Clone)]
pub struct EventQueue {
    pub next_interrupt: TickTimestamp,
    pub clock: Clock,
    internal_events: PriorityQueue<InternalEvent, Reverse<TickTimestamp>>,
    wire_events: PriorityQueue<WireChangeEvent, Reverse<TickTimestamp>>,
    root_prefix: u8,
    receiver: Receiver<(WireChangeEvent, Timestamp)>,

    multiplexing_table: MultiplexingTable,
}

impl EventQueue {
    pub fn new(
        ticks_per_cycle: TimeDiff,
        root_prefix: u8,
        receiver: Receiver<(WireChangeEvent, Timestamp)>,
    ) -> Self {
        Self {
            next_interrupt: TickTimestamp::MAX,
            clock: Clock::new(ticks_per_cycle),
            internal_events: PriorityQueue::new(),
            wire_events: PriorityQueue::new(),
            root_prefix,
            receiver,

            multiplexing_table: MultiplexingTable::new(),
        }
    }

    pub fn root_module_id(&self) -> ModuleAddress {
        ModuleAddress::root().child_id(self.root_prefix)
    }

    #[inline]
    pub fn fire_event(&mut self, mut event: InternalEvent, t: TickTimestamp) {
        assert!(event.receiver_id.module_address.current() == self.root_prefix);
        event.receiver_id.module_address.advance();
        self.internal_events.push(event, Reverse(t));
    }

    #[inline]
    pub fn fire_event_now(&mut self, event: InternalEvent) {
        self.fire_event(event, self.clock.current_tick());
    }

    #[inline]
    pub fn fire_event_next_tick(&mut self, event: InternalEvent) {
        self.fire_event(event, self.clock.current_tick() + 1);
    }

    #[inline]
    pub fn set_wire(&mut self, writer_pin_address: PinAddress, state: WireState) {
        for reader_id in self
            .multiplexing_table
            .outgoing_event_listeners(writer_pin_address)
        {
            let mut e = WireChangeEvent {
                receiver_id: reader_id,
                state,
            };
            if reader_id.module_address.current() == self.root_prefix {
                e.receiver_id.module_address.advance();
                self.wire_events.push(e, Reverse(self.clock.current_tick()));
                if self
                    .wire_events
                    .peek()
                    .map_or(false, |(_, &Reverse(t_queued))| {
                        t_queued < self.next_interrupt
                    })
                {
                    let (_, &Reverse(t_queued)) = self.wire_events.peek().unwrap();
                    self.next_interrupt = t_queued;
                }
            } else {
                InboxTable::send(e, self.clock.current_time());
            }
        }
    }

    pub fn update(&mut self, root: &mut impl Module) {
        while let Ok(Some((e, t))) = self.receiver.try_recv() {
            let readers: SmallVec<[PinAddress; 4]> = self
                .multiplexing_table
                .incoming_event_listeners(e.receiver_id)
                .collect();
            for reader in readers {
                self.wire_events.push(
                    WireChangeEvent {
                        receiver_id: reader,
                        state: e.state,
                    },
                    Reverse(self.clock.time_to_ticks(t)),
                );
            }
        }
        loop {
            let mut min_t = TickTimestamp::MAX;
            if let Some((&e, &Reverse(t))) = self.internal_events.peek() {
                if t <= self.clock.current_tick() {
                    self.internal_events.pop().unwrap();
                    let m = root.find_mut(e.receiver_id.module_address);
                    if let Some(m) = m {
                        m.handle_event(e, self, t);
                    } else {
                        panic!("Module not found: {:?}", e.receiver_id);
                    }
                    continue;
                }
                if t < min_t {
                    min_t = t;
                }
            }
            if let Some((&e, &Reverse(t))) = self.wire_events.peek() {
                if t <= self.clock.current_tick() {
                    self.wire_events.pop().unwrap();

                    let m = root.find_mut(e.receiver_id.module_address);

                    if let Some(m) = m {
                        if let Some(m) = m.to_wireable() {
                            m.set_pin(self, e.receiver_id.pin_id as PinId, e.state);
                        } else {
                            panic!("Module not wireable: {:?}", e.receiver_id);
                        }
                    } else {
                        panic!("Module not found: {:?}", e.receiver_id);
                    }
                    continue;
                }
                if t < min_t {
                    min_t = t;
                }
            }
            self.next_interrupt = min_t;
            break;
        }
    }

    pub fn register_multiplexer(&mut self, main_pin: PinAddress, alternatives: &[PinAddress]) {
        self.multiplexing_table.register(main_pin, alternatives)
    }

    pub fn set_multiplexer_flag(&mut self, pin: PinAddress, flag: bool) {
        self.multiplexing_table.set_flag(pin, flag)
    }

    pub fn is_empty(&self) -> bool {
        self.internal_events.is_empty() && self.wire_events.is_empty()
    }

    pub fn skip_to_event(&mut self) {
        let t1 = self.wire_events.peek().map(|(_, &Reverse(t))| t);
        let t2 = self.internal_events.peek().map(|(_, &Reverse(t))| t);
        let t = t1.min(t2);
        if let Some(t) = t {
            let ticks = t - self.clock.current_tick();
            self.clock.advance(ticks);
        } else {
            self.clock.advance(1000);
        }
    }
}
