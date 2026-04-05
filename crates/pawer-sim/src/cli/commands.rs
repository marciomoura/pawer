use crate::cli::parser::{Command, FormatArgs, PlotArgs, SimulateArg};
use crate::cli::{DisplayFormat, Notation};
use crate::engine::Engine;
use crate::export;
use crate::plotter;

/// Execute a parsed command against the simulation engine.
/// Returns `true` if the REPL should continue, `false` to quit.
pub fn execute(cmd: Command, engine: &mut Engine, display: &mut DisplayFormat) -> bool {
    match cmd {
        Command::Simulate(arg) => cmd_simulate(engine, arg),
        Command::Plot(args) => cmd_plot(engine, args),
        Command::Set(name, value) => cmd_set(engine, &name, value),
        Command::Save(path) => cmd_save(engine, &path),
        Command::Reset => cmd_reset(engine),
        Command::Status => cmd_status(engine),
        Command::Signals => cmd_signals(engine),
        Command::Params => cmd_params(engine),
        Command::Snapshot(names) => cmd_snapshot(engine, &names, display),
        Command::Format(args) => cmd_format(display, args),
        Command::Help => cmd_help(),
        Command::Quit => return false,
    }
    true
}

fn cmd_simulate(engine: &mut Engine, arg: SimulateArg) {
    let t_start = engine.time();
    let steps = match arg {
        SimulateArg::Duration(d) => engine.run_duration(d),
        SimulateArg::Steps(n) => engine.run_steps(n),
    };
    let t_end = engine.time();
    println!(
        "  Simulated {} steps (t = {:.6} → {:.6})",
        steps, t_start, t_end
    );
}

fn cmd_plot(engine: &Engine, args: PlotArgs) {
    let output = args.output.unwrap_or_else(|| "plot.svg".to_owned());

    // Validate signal names
    let available = engine.signal_names();
    for name in &args.signals {
        if !available.contains(name) {
            println!(
                "  Error: unknown signal \"{}\". Use /signals to list available signals.",
                name
            );
            return;
        }
    }

    match plotter::plot_signals(engine.logger(), &args.signals, &output) {
        Ok(()) => println!("  Plot saved to {}", output),
        Err(e) => println!("  Error: {}", e),
    }
}

fn cmd_set(engine: &mut Engine, name: &str, value: f64) {
    engine.set_param(name, value);
    println!("  {} = {}", name, value);
}

fn cmd_save(engine: &Engine, path: &str) {
    match export::export_csv(engine.logger(), path) {
        Ok(n) => println!("  Exported {} records to {}", n, path),
        Err(e) => println!("  Error: {}", e),
    }
}

fn cmd_reset(engine: &mut Engine) {
    engine.reset();
    println!("  Simulation reset to t = 0.");
}

fn cmd_status(engine: &Engine) {
    println!("  Time:       {:.6} s", engine.time());
    println!("  Steps:      {}", engine.step_count());
    println!("  Δt:         {:.6e} s", engine.dt());
    println!("  Log records: {}", engine.logger().len());
    let sigs = engine.signal_names();
    if sigs.is_empty() {
        println!("  Signals:    (none)");
    } else {
        println!("  Signals:    {}", sigs.join(", "));
    }
}

fn cmd_signals(engine: &Engine) {
    let names = engine.signal_names();
    if names.is_empty() {
        println!("  No signals logged yet. Run /simulate first.");
    } else {
        println!("  {}", names.join(", "));
    }
}

fn cmd_params(engine: &Engine) {
    let params = engine.params();
    if params.is_empty() {
        println!("  No parameters set.");
    } else {
        for (name, value) in params {
            println!("  {} = {}", name, value);
        }
    }
}

fn cmd_snapshot(engine: &Engine, names: &[String], display: &DisplayFormat) {
    let entries = engine.snapshot(names);
    if entries.is_empty() {
        println!("  No signals available. Run /simulate first.");
        return;
    }
    let max_len = entries.iter().map(|(n, _)| n.len()).max().unwrap_or(0);
    for (name, value) in &entries {
        match value {
            Some(v) => println!("  {:width$} = {}", name, display.fmt(*v), width = max_len),
            None => println!("  {:width$} = (no data)", name, width = max_len),
        }
    }
}

fn cmd_format(display: &mut DisplayFormat, args: FormatArgs) {
    if args.notation.is_none() && args.precision.is_none() {
        println!("  notation  = {}", display.notation);
        println!("  precision = {}", display.precision);
        return;
    }

    if let Some(ref notation) = args.notation {
        display.notation = match notation.as_str() {
            "default" => Notation::Default,
            "fixed" => Notation::Fixed,
            "scientific" | "sci" => Notation::Scientific,
            _ => unreachable!(),
        };
    }
    if let Some(precision) = args.precision {
        display.precision = precision;
    }

    println!("  notation  = {}", display.notation);
    println!("  precision = {}", display.precision);
}

fn cmd_help() {
    println!(
        r#"
  Available commands:

    /simulate <seconds>      Run simulation for the given duration
    /simulate <N>steps       Run simulation for N discrete steps
    /sim, /run               Aliases for /simulate

    /plot <sig1> [sig2 ...]  Generate SVG plot of named signals
         [-o file.svg]       Optional output path (default: plot.svg)

    /set <name> <value>      Set a scenario parameter

    /save <file.csv>         Export logged data to CSV
    /export                  Alias for /save

    /snapshot [sig1 ...]     Show latest value of all or selected signals
    /snap                    Alias for /snapshot

    /format [notation] [N]   Show or set display format for /snapshot
    /fmt                     Alias for /format
                             Notations: default, fixed, scientific (sci)
                             Examples: /format fixed 6, /format sci 3

    /reset                   Reset simulation to t=0
    /status                  Show current simulation state
    /signals, /sigs          List all logged signal names
    /params                  List all parameters and values
    /help, /h, /?            Show this help text
    /quit, /exit, /q         Exit the simulator
"#
    );
}
