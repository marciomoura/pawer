pub mod commands;
pub mod parser;

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use crate::engine::Engine;
use crate::scenario::Scenario;

/// Interactive simulation CLI.
///
/// Wraps a [`Scenario`] in an [`Engine`] and provides a REPL for
/// interactively controlling the simulation.
///
/// # Example
///
/// ```ignore
/// let scenario = MyScenario::new();
/// let mut cli = SimCli::new(Box::new(scenario), 0.001);
/// cli.run();
/// ```
pub struct SimCli {
    engine: Engine,
}

impl SimCli {
    /// Create a new CLI with the given scenario and sampling period (seconds).
    pub fn new(scenario: Box<dyn Scenario>, dt: f64) -> Self {
        Self {
            engine: Engine::new(scenario, dt),
        }
    }

    /// Enter the interactive REPL. Blocks until the user types `/quit`.
    pub fn run(&mut self) {
        self.engine.init();
        print_banner(self.engine.dt());

        let mut rl = match DefaultEditor::new() {
            Ok(editor) => editor,
            Err(e) => {
                eprintln!("Failed to initialize readline: {}", e);
                return;
            }
        };

        loop {
            match rl.readline("pawer-sim> ") {
                Ok(line) => {
                    let line = line.trim().to_owned();
                    if line.is_empty() {
                        continue;
                    }
                    let _ = rl.add_history_entry(&line);

                    match parser::parse(&line) {
                        Ok(cmd) => {
                            if !commands::execute(cmd, &mut self.engine) {
                                println!("  Goodbye.");
                                break;
                            }
                        }
                        Err(e) => println!("  {}", e),
                    }
                }
                Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                    println!("  Goodbye.");
                    break;
                }
                Err(e) => {
                    eprintln!("  Readline error: {}", e);
                    break;
                }
            }
        }
    }
}

fn print_banner(dt: f64) {
    println!();
    println!("  ╔══════════════════════════════════════════╗");
    println!("  ║            pawer-sim  v{}            ║", env!("CARGO_PKG_VERSION"));
    println!("  ║   Interactive Simulation Environment     ║");
    println!("  ╚══════════════════════════════════════════╝");
    println!();
    println!("  Sampling period: {:.6e} s", dt);
    println!("  Type /help for available commands.");
    println!();
}
