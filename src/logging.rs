use colored::Colorize;
use std::time::{Instant};

/// Log a success message in green
pub fn success(msg: &str) {
    log::info!("{}", msg.green());
}

/// Log info
pub fn info(from: &str, msg: &str)
{
    log::info!("{from}: {msg}");
}

/// Log an error message in red
pub fn error(msg: &str) {
    log::error!("{}", msg);
}

/// Log a debug message
pub fn debug(from : &str, msg: &str) {
    log::debug!("{from}: {}", msg);
}

/// Log a warning in yellow
pub fn warn(from : &str, msg: &str) {
    log::warn!("{from}: {}", msg.yellow());
}

/// Log timing information
pub fn elapsed(what: &str, start: Instant) {
    let ms = start.elapsed().as_millis();
    log::info!("{} completed in {}ms", what.green(), ms);
}

/// Log the start of a step (nice for progress)
pub fn start_step(step: &str) -> Instant {
    log::info!("Starting {}", step.blue());
    Instant::now()
}
