use std::{
    ops::DerefMut,
    process::exit,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use arrayvec::ArrayString;
use clap::{builder::Str, Parser, Subcommand};
use components::uart_module::UartModule;
use lua::{run_test, TestResult};
use parser::load;
use vcd::VcdEvent;

pub mod clock;
pub mod components;
pub mod events;
mod lua;
pub mod module;
pub mod module_holder;
pub mod module_id;
pub mod multiplexer;
mod parser;
pub mod pin_state;
pub mod system;
mod system_tables;
mod vcd;
pub mod wiring;

#[derive(Parser, Debug)]
struct Args {
    /// Path to the config YAML file
    #[arg(short, long)]
    config: Option<String>,

    /// Verbose mode
    #[arg(short, long)]
    verbose: bool,

    /// Enable VCD output
    #[arg(long)]
    vcd: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run Lua tests
    Test { tests: Vec<String> },
    /// Run a simulation
    Run {
        /// Simulation duration in seconds. No value means running forever in realtime.
        duration: Option<i64>,

        /// UART console to connect to
        #[arg(long)]
        uart: Option<String>,
    },
}

fn main() {
    let args: Args = Args::parse();
    let config = args.config.unwrap_or("input.yaml".to_string());
    match args.command {
        Commands::Test { tests } => {
            if tests.len() == 0 {
                println!("No tests specified");
                return;
            }

            for test in tests {
                match run_test(&config, &test, args.vcd) {
                    TestResult::Success(simulation_time) => {
                        println!("Test {} passed in {} ms", test, simulation_time.as_millis());
                    }
                    TestResult::Error(err, messages) => {
                        println!("Test {} failed", test);
                        println!("Error: {}", err);
                        if args.verbose {
                            for message in messages.iter() {
                                println!("{}", message);
                            }
                        }
                    }
                    TestResult::Failure(messages) => {
                        println!("Test {} failed", test);
                        if args.verbose {
                            for message in messages.iter() {
                                println!("{}", message);
                            }
                        }
                    }
                }
            }
        }
        Commands::Run { duration, uart } => {
            let mut sys = load(&config, args.vcd);
            let uart_module: Option<&mut UartModule> =
                uart.and_then(|id| sys.find_module_mut(&id).as_any_mut().downcast_mut());
            if let Some(u) = uart_module {
                u.connect();
            }

            const FREQ: i64 = 16_000_000;

            let vcd = Arc::new(Mutex::new(Some(sys.vcd.take().unwrap().deploy())));
            let vcd_clone = vcd.clone();

            ctrlc::set_handler(move || {
                println!("Terminating simulation...");
                drop(vcd_clone.lock().unwrap().take());
                exit(0);
            })
            .unwrap();

            if let Some(duration) = duration {
                let start = Instant::now();
                let model_time = duration * FREQ;
                sys.run_for(model_time);

                let simulation_time = start.elapsed();
                let model_time =
                    Duration::from_micros((model_time as f64 / FREQ as f64 * 1e6) as u64);

                if args.verbose {
                    let messages = sys.system_tables.messages.read().unwrap();
                    for message in messages.iter() {
                        println!("{}", message);
                    }
                }

                println!(
                    "Model Time: {} ms, Simulation Time: {} ms, Speed: {:.2}%",
                    model_time.as_millis(),
                    simulation_time.as_millis(),
                    model_time.as_nanos() as f64 / simulation_time.as_nanos() as f64 * 100.0
                );
            } else {
                sys.run_realtime(FREQ);
            }

            drop(vcd.lock().unwrap().take());
        }
    }
}
