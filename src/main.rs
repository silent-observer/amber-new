use std::time::{Duration, Instant};

use arrayvec::ArrayString;
use clap::{Parser, Subcommand};
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
    Run { duration: i64 },
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
        Commands::Run { duration } => {
            let mut sys = load(&config, args.vcd);
            let mcu = sys.modules[0].as_mut();
            const FREQ: i64 = 16_000_000;

            let thread = std::thread::spawn(move || {
                sys.vcd.run();
            });

            let start = Instant::now();
            let model_time = mcu.run_until_time(duration * FREQ);
            let simulation_time = start.elapsed();
            let model_time = Duration::from_micros((model_time as f64 / FREQ as f64 * 1e6) as u64);

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

            sys.vcd_sender
                .send(VcdEvent {
                    t: 0,
                    signal_id: -1,
                    new_value: ArrayString::new(),
                })
                .unwrap();
            thread.join().unwrap();
        }
    }
}
