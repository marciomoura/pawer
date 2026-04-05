use crate::context::SimContext;
use crate::logger::Logger;
use crate::scenario::Scenario;

/// Simulation engine that drives a [`Scenario`] forward in time.
///
/// The engine owns the simulation context, logger, and scenario. It provides
/// methods to step, run, and reset the simulation, which are called by the CLI
/// command dispatcher.
pub struct Engine {
    scenario: Box<dyn Scenario>,
    pub(crate) ctx: SimContext,
    pub(crate) logger: Logger,
    step_count: u64,
    initialized: bool,
}

impl Engine {
    /// Create a new engine with the given scenario and sampling period.
    pub fn new(scenario: Box<dyn Scenario>, dt: f64) -> Self {
        Self {
            scenario,
            ctx: SimContext::new(dt),
            logger: Logger::new(),
            step_count: 0,
            initialized: false,
        }
    }

    /// Initialize (or re-initialize) the scenario.
    pub fn init(&mut self) {
        self.scenario.init(&mut self.ctx);
        self.initialized = true;
    }

    /// Execute a single simulation step.
    pub fn step(&mut self) {
        if !self.initialized {
            self.init();
        }
        self.scenario.step(&mut self.ctx);
        let buffer = self.ctx.take_step_buffer();
        self.logger.record(self.ctx.time(), buffer);
        self.ctx.advance_time();
        self.step_count += 1;
    }

    /// Run simulation for the given duration in seconds.
    /// Returns the number of steps executed.
    pub fn run_duration(&mut self, duration: f64) -> u64 {
        let dt = self.ctx.dt_f64();
        let steps = (duration / dt).round() as u64;
        self.run_steps(steps)
    }

    /// Run simulation for the given number of steps.
    /// Returns the number of steps executed.
    pub fn run_steps(&mut self, steps: u64) -> u64 {
        for _ in 0..steps {
            self.step();
        }
        steps
    }

    /// Reset simulation to t=0, clear logs, and re-initialize the scenario.
    pub fn reset(&mut self) {
        self.ctx.reset_time();
        self.ctx.clear_params();
        self.logger.reset();
        self.step_count = 0;
        self.initialized = false;
        self.init();
    }

    /// Current simulation time.
    pub fn time(&self) -> f64 {
        self.ctx.time()
    }

    /// Total number of steps executed since last reset.
    pub fn step_count(&self) -> u64 {
        self.step_count
    }

    /// Sampling period.
    pub fn dt(&self) -> f64 {
        self.ctx.dt_f64()
    }

    /// Access the logger for reading recorded data.
    pub fn logger(&self) -> &Logger {
        &self.logger
    }

    /// List all signal names that have been logged.
    pub fn signal_names(&self) -> Vec<String> {
        self.logger.signal_names()
    }

    /// List all parameter names and values.
    pub fn params(&self) -> Vec<(String, f64)> {
        let names = self.ctx.param_names();
        names
            .into_iter()
            .map(|n| {
                let v = self.ctx.param_f64(&n);
                (n, v)
            })
            .collect()
    }

    /// Set a parameter value on the context.
    pub fn set_param(&mut self, name: &str, value: f64) {
        self.ctx.set_param(name, value);
    }
}
