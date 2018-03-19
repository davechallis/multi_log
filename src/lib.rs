//! Provides a `MultiLogger` that allows for logging to any number of loggers
//! that have the [`log::Log`](https://docs.rs/log/0.4.1/log/trait.Log.html) trait.
//!
//! This enables log messages to be sent to multiple loggers from a single
//! log macro ([`debug!`](https://docs.rs/log/0.4.1/log/macro.debug.html),
//! [`info!`](https://docs.rs/log/0.4.1/log/macro.info.html), etc.).
//!
//! # Example
//! ```
//! extern crate log;
//! extern crate env_logger;
//! extern crate simplelog;
//! extern crate multi_logger;
//!
//! let logger_a = Box::new(env_logger::Builder::from_default_env().build());
//! let logger_b = simplelog::SimpleLogger::new(log::LevelFilter::Warn, simplelog::Config::default());
//! multi_logger::MultiLogger::init(vec![logger_a, logger_b], log::Level::Info);
//! ```

#[cfg_attr(test, macro_use)]
extern crate log;

/// Logger that writes log messages to all the loggers it encapsulates.
pub struct MultiLogger {
    loggers: Vec<Box<log::Log>>,
}

impl MultiLogger {
    /// Creates a MultiLogger from any number of other loggers.
    ///
    /// Once initialised, this will need setting as the
    /// [`log`](https://docs.rs/log/0.4.1/log/) crate's global logger using
    /// [`log::set_boxed_logger`](https://docs.rs/log/0.4.1/log/fn.set_boxed_logger.html).
    pub fn new(loggers: Vec<Box<log::Log>>) -> Self {
        MultiLogger { loggers }
    }

    /// Initialises the [`log`](https://docs.rs/log/0.4.1/log/) crate's global logging facility
    /// with a MultiLogger built from any number of given loggers.
    ///
    /// The log level threshold of individual loggers can't always be determined, so a `level`
    /// parameter is provided as an optimisation to avoid sending unnecessary messages to
    /// loggers that will discard them.
    ///
    /// # Arguments
    /// * `loggers` - one more more boxed loggers
    /// * `level` - minimum log level to send to all loggers
    pub fn init(loggers: Vec<Box<log::Log>>, level: log::Level) -> Result<(), log::SetLoggerError> {
        log::set_max_level(level.to_level_filter());
        log::set_boxed_logger(Box::new(MultiLogger::new(loggers)))
    }
}

impl log::Log for MultiLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.loggers.iter().any(|logger| logger.enabled(metadata))
    }

    fn log(&self, record: &log::Record) {
        self.loggers.iter().for_each(|logger| logger.log(record));
    }

    fn flush(&self) {
        self.loggers.iter().for_each(|logger| logger.flush());
    }
}

#[cfg(test)]
mod tests {
    extern crate log;

    use std::sync::{Arc, Mutex};
    use std::ops::Deref;

    use super::MultiLogger;

    struct VecLogger {
        messages: Arc<Mutex<Vec<String>>>,
        level: log::Level,
    }

    impl VecLogger {
        fn new(messages: Arc<Mutex<Vec<String>>>, level: log::Level) -> Self {
            VecLogger { messages, level }
        }
    }

    impl log::Log for VecLogger {
        fn enabled(&self, metadata: &log::Metadata) -> bool {
            metadata.level() <= self.level
        }

        fn log(&self, record: &log::Record) {
            if self.enabled(record.metadata()) {
                let mut messages = self.messages.lock().unwrap();
                messages.push(format!("{}", record.args()));
            }
        }

        fn flush(&self) {}
    }

    #[test]
    fn multiple_vec_loggers() {
        let mutex_a = Arc::new(Mutex::new(Vec::new()));
        let mutex_b = Arc::new(Mutex::new(Vec::new()));
        let mutex_c = Arc::new(Mutex::new(Vec::new()));

        let logger = MultiLogger::new(vec![Box::new(VecLogger::new(mutex_a.clone(), log::Level::Debug)),
                                           Box::new(VecLogger::new(mutex_b.clone(), log::Level::Info)),
                                           Box::new(VecLogger::new(mutex_c.clone(), log::Level::Error))]);

        log::set_max_level(log::Level::Trace.to_level_filter());
        log::set_boxed_logger(Box::new(logger)).unwrap(); // can only be initialised once globally

        debug!("debug");
        info!("info");
        warn!("warn");
        error!("error");

        assert_eq!(get_messages(mutex_a.clone()), vec!["debug", "info", "warn", "error"]);
        assert_eq!(get_messages(mutex_b.clone()), vec!["info", "warn", "error"]);
        assert_eq!(get_messages(mutex_c.clone()), vec!["error"]);
    }

    fn get_messages(mutex: Arc<Mutex<Vec<String>>>) -> Vec<String> {
        let lock = mutex.lock().unwrap();
        lock.deref().clone()
    }
}
