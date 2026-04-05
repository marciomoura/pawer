/// Example: PI controller step response.
///
/// Demonstrates how to set up a simple closed-loop scenario using a
/// `pawer::PiController` and drive it interactively through the CLI.
///
/// Run with:
/// ```bash
/// cargo run -p pawer-sim --example pi_step_response
/// ```
///
/// Then try:
/// ```text
/// /set setpoint 1.0
/// /simulate 0.05
/// /plot setpoint output command
/// /save results.csv
/// /quit
/// ```
use pawer::pi_controller::PiController;
use pawer::types::Real;
use pawer_sim::prelude::*;

#[derive(Default)]
struct PiSignals {
    setpoint: SignalId,
    error: SignalId,
    command: SignalId,
    output: SignalId,
}

struct PiStepResponse {
    pi: PiController,
    plant_output: Real,
    sigs: PiSignals,
}

impl Scenario for PiStepResponse {
    fn init(&mut self, ctx: &mut SimContext) {
        ctx.set_param("setpoint", 0.0);
        ctx.set_param("tau", 0.01);

        self.sigs.setpoint = ctx.register_signal("setpoint");
        self.sigs.error = ctx.register_signal("error");
        self.sigs.command = ctx.register_signal("command");
        self.sigs.output = ctx.register_signal("output");

        self.pi.configure(2.0, 100.0);
        self.plant_output = 0.0;
    }

    fn step(&mut self, ctx: &mut SimContext) {
        let setpoint = ctx.param("setpoint");
        let tau: Real = ctx.param("tau");

        let error = setpoint - self.plant_output;
        let command = self.pi.update(error);

        // Simple first-order plant: y += (u - y) * dt / tau
        if tau.abs() > 1e-12 {
            self.plant_output += (command - self.plant_output) * ctx.dt() / tau;
        }

        ctx.log_id(self.sigs.setpoint, setpoint);
        ctx.log_id(self.sigs.error, error);
        ctx.log_id(self.sigs.command, command);
        ctx.log_id(self.sigs.output, self.plant_output);
    }
}

fn main() {
    let scenario = PiStepResponse {
        pi: PiController::new(0.001),
        plant_output: 0.0,
        sigs: PiSignals::default(),
    };
    let mut cli = SimCli::new(Box::new(scenario), 0.001);
    cli.run();
}
