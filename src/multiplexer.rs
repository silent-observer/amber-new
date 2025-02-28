use std::collections::HashMap;

use itertools::Either;

use crate::{
    module_id::PinAddress,
    system_tables::{self, SystemTables},
    wiring::WiringTable,
};

#[derive(Debug, Clone)]
struct Multiplexer {
    wireable_pin: PinAddress,
    connections: Vec<PinAddress>,
    flags: Vec<bool>,
    active_position: usize,
}

#[derive(Debug, Clone)]
pub struct MultiplexingTable {
    incoming_event_table: HashMap<PinAddress, Vec<PinAddress>>,
    multiplexer_table: HashMap<PinAddress, usize>,
    multiplexers: Vec<Multiplexer>,
}

impl MultiplexingTable {
    pub fn new() -> Self {
        Self {
            incoming_event_table: HashMap::new(),
            multiplexer_table: HashMap::new(),
            multiplexers: Vec::new(),
        }
    }

    pub fn register(&mut self, wireable_pin: PinAddress, connections: &[PinAddress]) {
        self.incoming_event_table
            .insert(wireable_pin, connections.to_vec());
        let multiplexer_id = self.multiplexers.len();
        self.multiplexer_table.insert(wireable_pin, multiplexer_id);
        for &addr in connections {
            self.multiplexer_table.insert(addr, multiplexer_id);
        }

        let mut flags: Vec<bool> = vec![false; connections.len()];
        flags[connections.len() - 1] = true;
        self.multiplexers.push(Multiplexer {
            connections: connections.to_vec(),
            flags,
            active_position: connections.len() - 1,
            wireable_pin,
        })
    }

    pub fn set_flag(&mut self, pin: PinAddress, flag: bool) {
        if let Some(&multiplexer_id) = self.multiplexer_table.get(&pin) {
            let m = &mut self.multiplexers[multiplexer_id];
            let position = m.connections.iter().position(|&a| a == pin).unwrap();
            m.flags[position] = flag;
            for (i, &f) in m.flags.iter().enumerate() {
                if f {
                    m.active_position = i;
                    break;
                }
            }
        }
    }

    pub fn incoming_event_listeners<'b>(
        &'b self,
        addr: PinAddress,
    ) -> impl Iterator<Item = PinAddress> + 'b {
        if let Some(connections) = self.incoming_event_table.get(&addr) {
            Either::Left(connections.iter().copied())
        } else {
            Either::Right(std::iter::once(addr))
        }
    }

    pub fn outgoing_event_listeners<'b>(
        &'b self,
        wt: &'b WiringTable,
        addr: PinAddress,
    ) -> impl Iterator<Item = PinAddress> + 'b {
        if let Some(&multiplexer_id) = self.multiplexer_table.get(&addr) {
            let m = &self.multiplexers[multiplexer_id];
            let position = m.connections.iter().position(|&a| a == addr).unwrap();
            if m.active_position == position {
                Either::Left(
                    m.connections
                        .iter()
                        .copied()
                        .filter(move |&a| a != addr)
                        .chain(wt.get_connected(m.wireable_pin).map_or_else(
                            || Either::Right(std::iter::empty()),
                            |v| Either::Left(v.iter().copied()),
                        )),
                )
            } else {
                Either::Right(std::iter::empty())
            }
        } else {
            Either::Right(std::iter::empty())
        }
    }

    pub fn read_pin_addr(&self, addr: PinAddress) -> PinAddress {
        if let Some(&multiplexer_id) = self.multiplexer_table.get(&addr) {
            let m = &self.multiplexers[multiplexer_id];
            m.connections[m.active_position]
        } else {
            addr
        }
    }
}
