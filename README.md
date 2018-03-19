# multi\_log #

A Rust library providing a logger which passes messages on to multiple loggers
from a single log call.

* [`multi_log` documentation](https://docs.rs/multi_log/)

## Usage ##

Create a `multi_log::MultiLogger` that wraps any number of loggers that
implement the [`log::Log`](https://docs.rs/log/0.4.1/log/trait.Log.html) trait.

Logging to all these can then be performed using the `log` crate's macros
([`debug!`](https://docs.rs/log/0.4.1/log/macro.debug.html),
[`info!`](https://docs.rs/log/0.4.1/log/macro.info.html), etc.).

## Example ##

    #[macro_use] extern crate log;
    extern crate env_logger;
    extern crate simplelog;
    extern crate multi_log;

    fn main() {
        // create a new logger from the `env_logger` crate
        let logger_a = Box::new(env_logger::Builder::new().filter(None, log::LevelFilter::Info).build());

        // create a new logger from the `simplelog` crate
        let logger_b = simplelog::SimpleLogger::new(log::LevelFilter::Warn, simplelog::Config::default());

        // wrap them both in a MultiLogger, and initialise as global logger
        multi_log::MultiLogger::init(vec![logger_a, logger_b], log::Level::Info).unwrap();

        warn!("This message should be logged with each logger.");
    }
