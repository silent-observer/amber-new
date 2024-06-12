use std::time::{Duration, Instant};

use components::led::Led;
use events::EventQueue;
use module::ActiveModule;

use parser::load;
use wiring::{InboxTable, WiringTable};

use crate::{components::avr::mcu, module_id::PinAddress};

pub mod clock;
pub mod components;
pub mod events;
pub mod module;
pub mod module_holder;
pub mod module_id;
pub mod multiplexer;
mod parser;
pub mod pin_state;
pub mod wiring;

fn main() {
    let mut modules = load("input.yaml");
    let mcu = modules[0].as_mut();

    let start = Instant::now();
    const SIMULATION_SECONDS: i64 = 10;
    const FREQ: i64 = 16_000_000;
    const CYCLES: i64 = SIMULATION_SECONDS * FREQ;
    let model_time = mcu.run_until_time(CYCLES);
    let simulation_time = start.elapsed();
    let model_time = Duration::from_micros((model_time as f64 / FREQ as f64 * 1e6) as u64);
    println!(
        "Model Time: {} ms, Simulation Time: {} ms, Speed: {:.2}%",
        model_time.as_millis(),
        simulation_time.as_millis(),
        model_time.as_nanos() as f64 / simulation_time.as_nanos() as f64 * 100.0
    )
}
