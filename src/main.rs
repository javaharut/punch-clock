//  main.rs
//  punch
//
//  Created by Søren Mortensen <soren@neros.dev> on 2019-12-26.
//  Copyright (c) 2019 Søren Mortensen.
//
//  Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
//  http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
//  http://opensource.org/licenses/MIT>, at your option. This file may not be
//  copied, modified, or distributed except according to those terms.

use chrono::{DateTime, Local};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "punch", about = "Lightweight time-tracking utility.")]
enum Opt {
    /// Start tracking time.
    In {
        /// The time to start the tracking period from (default: now).
        ///
        /// Currently unimplemented.
        #[structopt(short = "t", long = "time")]
        time: Option<DateTime<Local>>,
    },
    /// Stop tracking time.
    Out {
        /// The time to end the tracking period at (default: now).
        ///
        /// Currently unimplemented.
        #[structopt(short = "t", long = "time")]
        time: Option<DateTime<Local>>,
    },
    /// Check the current status of time-tracking.
    Status,
    /// Count the amount of time worked over a certain period of time.
    Count {
        /// Period of time to count from.
        ///
        /// Values for <period> include: today, yesterday, week, month, last week, last month.
        /// Shortened versions of these values are also available, such as "t" for "today".
        period: String,
    },
}

fn main() {
    let opt = Opt::from_args();

    match opt {
        Opt::In { time: _ } => {
            let time = Local::now();

            println!("Punching in at {}.", time.format("%H:%M:%S").to_string());
        }
        Opt::Out { time: _ } => {
            let time = Local::now();

            println!("Punching out at {}.", time.format("%H:%M:%S").to_string());
        }
        Opt::Status => {
            println!("Not tracking time.");
        }
        Opt::Count { period } => {
            let pretty_period = match &*period.to_lowercase() {
                "today" | "t" => "today",
                "yesterday" | "y" => "yesterday",
                "week" | "this week" | "w" => "this week",
                "last week" | "lastweek" | "lw" => "last week",
                "month" | "this month" | "m" => "this month",
                "last month" | "lastmonth" | "lm" => "last month",
                _ => {
                    eprintln!("Unrecognised period \"{}\".", period);
                    return;
                }
            };

            println!("Time worked {}: none.", pretty_period);
        }
    }
}
