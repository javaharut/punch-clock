use chrono::{DateTime, Local};
use punch_clock::Period;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "punch", about = "Lightweight time-tracking utility.")]
pub enum Opt {
    /// Start tracking time.
    In {
        /// The time to start the tracking period from (default: now). Currently unimplemented;
        /// always defaults to now.
        #[structopt(short = "t", long = "time")]
        time: Option<DateTime<Local>>,
    },
    /// Stop tracking time.
    Out {
        /// The time to end the tracking period at (default: now). Currently unimplemented; always
        /// defaults to now.
        #[structopt(short = "t", long = "time")]
        time: Option<DateTime<Local>>,
    },
    /// Check whether currently punched in, and if so, since when.
    Status,
    /// Count the amount of time worked over a certain period of time.
    Count {
        /// Period of time to count from. Values for <period> include: all, today, yesterday, week,
        /// month, last week, last month. Shortened versions of these values are also available,
        /// such as "t" for "today".
        #[structopt(default_value = "today")]
        period: Period,
    },
}
