use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a (possibly ongoing) period of time tracking, with its associated metadata.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Event {
    /// The start of a time-tracking period.
    pub start: DateTime<Utc>,
    /// The end of a time-tracking period.
    pub stop: Option<DateTime<Utc>>,
}

impl Event {
    /// Create a new event starting at the given time.
    pub fn new(start: DateTime<Utc>) -> Self {
        Event { start, stop: None }
    }
}
