//  period.rs
//  punch-clock
//
//  Created by Søren Mortensen <soren@neros.dev> on 2020-03-16.
//  Copyright (c) 2020 Søren Mortensen.
//
//  Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
//  http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
//  http://opensource.org/licenses/MIT>, at your option. This file may not be
//  copied, modified, or distributed except according to those terms.

use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Period {
    All,
    Today,
    Yesterday,
    Week,
    LastWeek,
    Month,
    LastMonth,
}

impl FromStr for Period {
    type Err = String;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "all" | "a" => Ok(Period::All),
            "today" | "t" => Ok(Period::Today),
            "yesterday" | "y" => Ok(Period::Yesterday),
            "week" | "this week" | "w" | "tw" => Ok(Period::Week),
            "last week" | "lastweek" | "lw" => Ok(Period::LastWeek),
            "month" | "this month" | "m" | "tm" => Ok(Period::Month),
            "last month" | "lastmonth" | "lm" => Ok(Period::LastMonth),
            _ => Err("Time period not recognised.".into()),
        }
    }
}

impl Display for Period {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Period::All => write!(f, "All-Time"),
            Period::Today => write!(f, "Today"),
            Period::Yesterday => write!(f, "Yesterday"),
            Period::Week => write!(f, "This Week"),
            Period::LastWeek => write!(f, "Last Week"),
            Period::Month => write!(f, "This Month"),
            Period::LastMonth => write!(f, "Last Month"),
        }
    }
}
