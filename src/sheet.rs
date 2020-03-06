//  sheet.rs
//  punch-clock
//
//  Created by Søren Mortensen <soren@neros.dev> on 2020-03-01.
//  Copyright (c) 2020 Søren Mortensen.
//
//  Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
//  http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
//  http://opensource.org/licenses/MIT>, at your option. This file may not be
//  copied, modified, or distributed except according to those terms.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::Event;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Sheet {
    pub events: Vec<Event>,
}

impl Sheet {
    pub fn punch_in(&mut self) -> Result<DateTime<Utc>, SheetError> {
        self.punch_in_at(Utc::now())
    }

    pub fn punch_in_at(&mut self, time: DateTime<Utc>) -> Result<DateTime<Utc>, SheetError> {
        match self.events.last() {
            Some(Event { stop: Some(_), .. }) | None => {
                let event = Event::new(time);
                self.events.push(event);
                Ok(time)
            }
            Some(Event {
                start: start_time, ..
            }) => Err(SheetError::PunchedIn(*start_time)),
        }
    }

    pub fn punch_out(&mut self) -> Result<DateTime<Utc>, SheetError> {
        self.punch_out_at(Utc::now())
    }

    pub fn punch_out_at(&mut self, time: DateTime<Utc>) -> Result<DateTime<Utc>, SheetError> {
        match self.events.last_mut() {
            Some(ref mut event @ Event { stop: None, .. }) => {
                event.stop = Some(time);
                Ok(time)
            }
            Some(Event {
                stop: Some(stop_time),
                ..
            }) => Err(SheetError::PunchedOut(*stop_time)),
            None => Err(SheetError::NoPunches),
        }
    }

    pub fn status(&self) -> SheetStatus {
        match self.events.last() {
            Some(Event {
                stop: Some(stop), ..
            }) => SheetStatus::PunchedOut(*stop),
            Some(Event { start, .. }) => SheetStatus::PunchedIn(*start),
            None => SheetStatus::Empty,
        }
    }

    pub fn count_range(&self, begin: DateTime<Utc>, end: DateTime<Utc>) -> Duration {
        self.events
            .iter()
            .map(|e| (e.start, e.stop.unwrap_or(Utc::now())))
            .filter(|(start, stop)| {
                let entirely_before = start < &begin && stop < &begin;
                let entirely_after = start > &end && stop > &end;

                !(entirely_before || entirely_after)
            })
            .map(|(start, stop)| {
                let real_begin = std::cmp::max(begin, start);
                let real_end = std::cmp::min(end, stop);

                real_end - real_begin
            })
            .fold(Duration::zero(), |acc, next| acc + next)
    }
}

impl Default for Sheet {
    fn default() -> Self {
        Sheet { events: vec![] }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SheetStatus {
    PunchedIn(DateTime<Utc>),
    PunchedOut(DateTime<Utc>),
    Empty,
}

#[derive(Error, Debug)]
pub enum SheetError {
    #[error("already punched in at {0}")]
    PunchedIn(DateTime<Utc>),
    #[error("not punched in, last punched out at {0}")]
    PunchedOut(DateTime<Utc>),
    #[error("not punched in, no punch-ins recorded")]
    NoPunches,
}
