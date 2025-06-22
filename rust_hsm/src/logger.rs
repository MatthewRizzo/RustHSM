//! Encapsulates how information should be logged!
use log::LevelFilter;

#[derive(Clone)]
/// Logger for the hsm!
pub struct HSMLogger {
    pub(crate) log_level_allowed: log::LevelFilter,
}

impl Default for HSMLogger {
    fn default() -> Self {
        Self {
            log_level_allowed: log::LevelFilter::Info,
        }
    }
}

impl HSMLogger {
    /// # Params
    /// level_allowed - The level of logs that will actually be printed
    pub fn new(level_allowed: log::LevelFilter) -> Self {
        Self {
            log_level_allowed: level_allowed,
        }
    }

    fn log_msg(&self, log_requested: &log::LevelFilter, function_logging: String, msg: &str) {
        if log_requested <= &self.log_level_allowed {
            println!("[{}][{}] {}", log_requested.as_str(), function_logging, msg);
        }
    }

    /// Attempt to log an info msg. It will get printed conditionally based on
    /// how you init the logger
    #[allow(dead_code)]
    pub(crate) fn log_info(&self, function_logging: String, msg: &str) {
        self.log_msg(&log::LevelFilter::Info, function_logging, msg)
    }

    /// Attempt to log an error msg. It will get printed conditionally based on
    /// how you init the logger
    pub(crate) fn log_error(&self, function_logging: String, msg: &str) {
        self.log_msg(&log::LevelFilter::Error, function_logging, msg)
    }

    /// Attempt to log debug msg. It will get printed conditionally based on
    /// how you init the logger.
    pub(crate) fn log_debug(&self, function_logging: String, msg: &str) {
        self.log_msg(&log::LevelFilter::Debug, function_logging, msg)
    }

    pub(crate) fn log_trace(&self, function_logging: String, msg: &str) {
        self.log_msg(&log::LevelFilter::Trace, function_logging, msg)
    }
}

impl From<LevelFilter> for HSMLogger {
    fn from(level: LevelFilter) -> Self {
        HSMLogger::new(level)
    }
}
