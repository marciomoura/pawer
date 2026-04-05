use crate::context::SimContext;

/// Trait that users implement to define a simulation scenario.
///
/// A scenario combines `pawer` control blocks (or any custom models) into
/// a test case that can be driven interactively through the CLI.
///
/// # Example
///
/// ```ignore
/// use pawer_sim::context::SignalId;
///
/// struct MyScenario {
///     sig_output: SignalId,
/// }
///
/// impl Scenario for MyScenario {
///     fn init(&mut self, ctx: &mut SimContext) {
///         ctx.set_param("setpoint", 0.0);
///         self.sig_output = ctx.register_signal("output");
///     }
///
///     fn step(&mut self, ctx: &mut SimContext) {
///         let sp = ctx.param("setpoint");
///         ctx.log_id(self.sig_output, sp * 2.0);
///     }
///
///     fn on_param_change(&mut self, name: &str, value: f64, _ctx: &SimContext) {
///         if name == "setpoint" {
///             println!("Setpoint changed to {}", value);
///         }
///     }
/// }
/// ```
pub trait Scenario {
    /// Called once before simulation starts and on every reset.
    ///
    /// Use this to register signals with [`SimContext::register_signal`],
    /// set initial parameter values, and configure control blocks.
    fn init(&mut self, ctx: &mut SimContext);

    /// Called every simulation step.
    ///
    /// Read parameters with [`SimContext::param`], compute outputs using
    /// your blocks, and log signals with [`SimContext::log_id`].
    fn step(&mut self, ctx: &mut SimContext);

    /// Called when a parameter is changed via the CLI `/set` command.
    ///
    /// Override this to react to parameter changes in an event-driven
    /// style, instead of polling parameters with if-else logic inside
    /// [`step`](Self::step).
    ///
    /// The parameter has already been set on `ctx` when this is called,
    /// so [`SimContext::param`] / [`SimContext::param_f64`] return the
    /// new value.
    fn on_param_change(&mut self, _name: &str, _value: f64, _ctx: &SimContext) {}
}
