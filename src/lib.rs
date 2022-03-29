//! The `punch-clock` crate is a lightweight tool for tracking time.
//!
//! This library exposes an API for performing all the same tasks as through the command-line
//! interface (e.g. punching in or out, checking time tracking status, counting totals).

mod event;
mod period;
pub mod sheet;

pub use event::Event;
pub use period::Period;
pub use sheet::Sheet;
