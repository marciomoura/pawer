/// Parsed CLI command.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Run simulation for a duration (seconds) or a number of steps.
    Simulate(SimulateArg),
    /// Generate an SVG plot of the named signals.
    Plot(PlotArgs),
    /// Set a scenario parameter.
    Set(String, f64),
    /// Export logged data to a CSV file.
    Save(String),
    /// Reset simulation to t=0 and re-initialize the scenario.
    Reset,
    /// Print current simulation status (time, step count, params).
    Status,
    /// List all logged signal names.
    Signals,
    /// List all parameters and their values.
    Params,
    /// Show the latest value of all or selected signals.
    Snapshot(Vec<String>),
    /// Configure display notation and/or precision.
    Format(FormatArgs),
    /// Print help text.
    Help,
    /// Exit the REPL.
    Quit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SimulateArg {
    /// Simulate for the given duration in seconds.
    Duration(f64),
    /// Simulate for the given number of steps.
    Steps(u64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlotArgs {
    pub signals: Vec<String>,
    pub output: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FormatArgs {
    pub notation: Option<String>,
    pub precision: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Parse a raw input line into a [`Command`].
pub fn parse(input: &str) -> Result<Command, ParseError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(ParseError {
            message: "Empty command. Type /help for available commands.".into(),
        });
    }

    // Commands must start with '/'
    if !input.starts_with('/') {
        return Err(ParseError {
            message: format!(
                "Unknown input \"{}\". Commands start with /. Type /help for help.",
                input
            ),
        });
    }

    let mut parts = input.split_whitespace();
    let cmd = parts.next().unwrap(); // safe: input is non-empty and starts with /
    let args: Vec<&str> = parts.collect();

    match cmd.to_lowercase().as_str() {
        "/simulate" | "/sim" | "/run" => parse_simulate(&args),
        "/plot" => parse_plot(&args),
        "/set" => parse_set(&args),
        "/save" | "/export" => parse_save(&args),
        "/reset" => Ok(Command::Reset),
        "/status" => Ok(Command::Status),
        "/signals" | "/sigs" => Ok(Command::Signals),
        "/params" => Ok(Command::Params),
        "/snapshot" | "/snap" => Ok(Command::Snapshot(args.iter().map(|s| s.to_string()).collect())),
        "/format" | "/fmt" => parse_format(&args),
        "/help" | "/h" | "/?" => Ok(Command::Help),
        "/quit" | "/exit" | "/q" => Ok(Command::Quit),
        other => Err(ParseError {
            message: format!(
                "Unknown command \"{}\". Type /help for available commands.",
                other
            ),
        }),
    }
}

fn parse_simulate(args: &[&str]) -> Result<Command, ParseError> {
    if args.is_empty() {
        return Err(ParseError {
            message: "Usage: /simulate <duration_seconds> or /simulate <N>steps".into(),
        });
    }

    let arg = args[0];

    // Check for "Nsteps" suffix
    if let Some(n_str) = arg
        .strip_suffix("steps")
        .or_else(|| arg.strip_suffix("s").filter(|s| s.ends_with("step")))
    {
        let n_str = n_str.strip_suffix("step").unwrap_or(n_str);
        match n_str.parse::<u64>() {
            Ok(n) if n > 0 => return Ok(Command::Simulate(SimulateArg::Steps(n))),
            _ => {
                return Err(ParseError {
                    message: format!(
                        "Invalid step count \"{}\". Must be a positive integer.",
                        arg
                    ),
                });
            }
        }
    }

    // Otherwise interpret as duration in seconds
    match arg.parse::<f64>() {
        Ok(d) if d > 0.0 => Ok(Command::Simulate(SimulateArg::Duration(d))),
        _ => Err(ParseError {
            message: format!(
                "Invalid duration \"{}\". Provide a positive number (seconds) or e.g. 100steps.",
                arg
            ),
        }),
    }
}

fn parse_plot(args: &[&str]) -> Result<Command, ParseError> {
    if args.is_empty() {
        return Err(ParseError {
            message: "Usage: /plot <signal1> [signal2 ...] [-o output.svg]".into(),
        });
    }

    let mut signals = Vec::new();
    let mut output = None;
    let mut iter = args.iter();

    while let Some(&arg) = iter.next() {
        if arg == "-o" || arg == "--output" {
            match iter.next() {
                Some(&path) => output = Some(path.to_owned()),
                None => {
                    return Err(ParseError {
                        message: "Expected file path after -o flag.".into(),
                    });
                }
            }
        } else {
            signals.push(arg.to_owned());
        }
    }

    if signals.is_empty() {
        return Err(ParseError {
            message: "No signal names provided. Usage: /plot <signal1> [signal2 ...]".into(),
        });
    }

    Ok(Command::Plot(PlotArgs { signals, output }))
}

fn parse_set(args: &[&str]) -> Result<Command, ParseError> {
    if args.len() < 2 {
        return Err(ParseError {
            message: "Usage: /set <name> <value>".into(),
        });
    }

    let name = args[0].to_owned();
    match args[1].parse::<f64>() {
        Ok(value) => Ok(Command::Set(name, value)),
        Err(_) => Err(ParseError {
            message: format!("Invalid value \"{}\". Must be a number.", args[1]),
        }),
    }
}

fn parse_save(args: &[&str]) -> Result<Command, ParseError> {
    if args.is_empty() {
        return Err(ParseError {
            message: "Usage: /save <filename.csv>".into(),
        });
    }
    Ok(Command::Save(args[0].to_owned()))
}

fn parse_format(args: &[&str]) -> Result<Command, ParseError> {
    if args.is_empty() {
        // No args → show current format (handled by command executor)
        return Ok(Command::Format(FormatArgs { notation: None, precision: None }));
    }

    let mut notation = None;
    let mut precision = None;

    for &arg in args {
        match arg {
            "default" | "fixed" | "scientific" | "sci" => {
                notation = Some(arg.to_owned());
            }
            _ => {
                match arg.parse::<usize>() {
                    Ok(p) => precision = Some(p),
                    Err(_) => {
                        return Err(ParseError {
                            message: format!(
                                "Invalid format argument \"{}\". Use: /format [default|fixed|scientific] [precision]",
                                arg
                            ),
                        });
                    }
                }
            }
        }
    }

    Ok(Command::Format(FormatArgs { notation, precision }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simulate_duration() {
        assert_eq!(
            parse("/simulate 0.1").unwrap(),
            Command::Simulate(SimulateArg::Duration(0.1))
        );
    }

    #[test]
    fn parse_simulate_steps() {
        assert_eq!(
            parse("/simulate 100steps").unwrap(),
            Command::Simulate(SimulateArg::Steps(100))
        );
    }

    #[test]
    fn parse_plot_signals() {
        assert_eq!(
            parse("/plot error output").unwrap(),
            Command::Plot(PlotArgs {
                signals: vec!["error".into(), "output".into()],
                output: None,
            })
        );
    }

    #[test]
    fn parse_plot_with_output() {
        assert_eq!(
            parse("/plot error -o my_plot.svg").unwrap(),
            Command::Plot(PlotArgs {
                signals: vec!["error".into()],
                output: Some("my_plot.svg".into()),
            })
        );
    }

    #[test]
    fn parse_set_param() {
        assert_eq!(
            parse("/set setpoint 1.5").unwrap(),
            Command::Set("setpoint".into(), 1.5)
        );
    }

    #[test]
    fn parse_save_file() {
        assert_eq!(
            parse("/save results.csv").unwrap(),
            Command::Save("results.csv".into())
        );
    }

    #[test]
    fn parse_simple_commands() {
        assert_eq!(parse("/reset").unwrap(), Command::Reset);
        assert_eq!(parse("/status").unwrap(), Command::Status);
        assert_eq!(parse("/signals").unwrap(), Command::Signals);
        assert_eq!(parse("/help").unwrap(), Command::Help);
        assert_eq!(parse("/quit").unwrap(), Command::Quit);
        assert_eq!(parse("/snapshot").unwrap(), Command::Snapshot(vec![]));
    }

    #[test]
    fn parse_snapshot_with_signals() {
        assert_eq!(
            parse("/snapshot freq_hz dq_d").unwrap(),
            Command::Snapshot(vec!["freq_hz".into(), "dq_d".into()])
        );
        assert_eq!(
            parse("/snap freq_hz").unwrap(),
            Command::Snapshot(vec!["freq_hz".into()])
        );
    }

    #[test]
    fn parse_aliases() {
        assert_eq!(
            parse("/sim 0.1").unwrap(),
            Command::Simulate(SimulateArg::Duration(0.1))
        );
        assert_eq!(
            parse("/run 0.1").unwrap(),
            Command::Simulate(SimulateArg::Duration(0.1))
        );
        assert_eq!(parse("/sigs").unwrap(), Command::Signals);
        assert_eq!(parse("/exit").unwrap(), Command::Quit);
        assert_eq!(parse("/q").unwrap(), Command::Quit);
    }

    #[test]
    fn parse_empty_input() {
        assert!(parse("").is_err());
        assert!(parse("   ").is_err());
    }

    #[test]
    fn parse_unknown_command() {
        assert!(parse("/foo").is_err());
    }

    #[test]
    fn parse_missing_slash() {
        assert!(parse("simulate 0.1").is_err());
    }
}
