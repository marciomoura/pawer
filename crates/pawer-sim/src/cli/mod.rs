pub mod commands;
pub mod parser;

use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use crate::engine::Engine;
use crate::scenario::Scenario;

/// Controls how numeric values are printed in the CLI.
#[derive(Clone, Debug)]
pub struct DisplayFormat {
    pub notation: Notation,
    pub precision: usize,
}

/// Notation style for numeric display.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Notation {
    /// Rust default `Display` (`50.123`).
    Default,
    /// Fixed-point (`50.123000`).
    Fixed,
    /// Scientific (`5.012300e1`).
    Scientific,
}

impl Default for DisplayFormat {
    fn default() -> Self {
        Self {
            notation: Notation::Fixed,
            precision: 4,
        }
    }
}

impl DisplayFormat {
    /// Format a value according to the current settings.
    pub fn fmt(&self, v: f32) -> String {
        match self.notation {
            Notation::Default => format!("{}", v),
            Notation::Fixed => format!("{:.prec$}", v, prec = self.precision),
            Notation::Scientific => format!("{:.prec$e}", v, prec = self.precision),
        }
    }
}

impl std::fmt::Display for Notation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Notation::Default => write!(f, "default"),
            Notation::Fixed => write!(f, "fixed"),
            Notation::Scientific => write!(f, "scientific"),
        }
    }
}

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
    display: DisplayFormat,
}

impl SimCli {
    /// Create a new CLI with the given scenario and sampling period (seconds).
    pub fn new(scenario: Box<dyn Scenario>, dt: f64) -> Self {
        Self {
            engine: Engine::new(scenario, dt),
            display: DisplayFormat::default(),
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
                            if !commands::execute(cmd, &mut self.engine, &mut self.display) {
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
    println!(
        "  ║            pawer-sim  v{}            ║",
        env!("CARGO_PKG_VERSION")
    );
    println!("  ║   Interactive Simulation Environment     ║");
    println!("  ╚══════════════════════════════════════════╝");
    println!();
    println!("  Sampling period: {:.6e} s", dt);
    println!("  Type /help for available commands.");
    println!();
}
