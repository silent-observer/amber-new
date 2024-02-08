use std::time::{Duration, Instant};

use components::led::Led;
use events::EventQueue;
use module::{ActiveModule, Module, WireableModule};
use module_id::ModuleId;
use wiring::{InboxTable, WiringTable};

use crate::components::avr::mcu;

pub mod clock;
pub mod common;
pub mod components;
pub mod events;
pub mod module;
pub mod module_holder;
pub mod module_id;
pub mod pin_state;
pub mod wiring;

fn main() {
    let mut it = InboxTable::new();
    let mut wt = WiringTable::new();

    let event_queue = EventQueue::new(1, 0, it.add_listener(0));
    let mut mcu = mcu::Mcu::new(event_queue).with_flash_hex("./hex/blink.hex");
    mcu.set_module_id(ModuleId::root().child_id(0).child_id(0));
    let led = mcu
        .module_store()
        .add_module(Box::new(Led::new()))
        .get_pin_module(0)
        .unwrap();

    wt.add_wire(mcu.get_pin_module(15).unwrap(), vec![led]);

    it.save();
    wt.save();

    let start = Instant::now();
    const SIMULATION_SECONDS: i64 = 10;
    const FREQ: i64 = 16_000_000;
    const CYCLES: i64 = SIMULATION_SECONDS * FREQ;
    mcu.run_until_time(CYCLES);
    let simulation_time = start.elapsed();
    let model_time = Duration::from_secs(SIMULATION_SECONDS as u64);
    println!(
        "Model Time: {} ms, Simulation Time: {} ms, Speed: {:.2}%",
        model_time.as_millis(),
        simulation_time.as_millis(),
        model_time.as_nanos() as f64 / simulation_time.as_nanos() as f64 * 100.0
    )
}
