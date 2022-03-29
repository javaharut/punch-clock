//! Working with recorded timesheets (lists of events).

use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use chrono::{DateTime, Duration, Utc};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::Event;

/// List of events, together comprising a log of work from which totals can be calculated for
/// various periods of time.
#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Sheet {
    pub events: Vec<Event>,
}

impl Sheet {
    /// Attempt to load a sheet from the file at the default location, as determined by
    /// [`default_loc()`][default].
    ///
    /// [default]: #method.default_loc
    pub fn load_default() -> Result<Sheet, SheetError> {
        Self::load(Self::default_loc()?)
    }

    /// Attempt to load a sheet from the file at the given path.
    pub fn load<P>(path: P) -> Result<Sheet, SheetError>
    where
        P: AsRef<Path>,
    {
        let mut sheet_json = String::new();

        {
            let mut sheet_file = File::open(&path).map_err(SheetError::OpenSheet)?;

            sheet_file
                .read_to_string(&mut sheet_json)
                .map_err(SheetError::ReadSheet)?;
        }

        if sheet_json.is_empty() {
            Ok(Sheet::default())
        } else {
            serde_json::from_str(&sheet_json).map_err(SheetError::ParseSheet)
        }
    }

    /// Get the default directory in which sheets are stored.
    ///
    /// The directory is determined using the [directories][directories] crate by platform as
    /// follows:
    ///
    /// + Linux: `$XDG_CONFIG_HOME/punchclock/sheet.json`
    /// + macOS: `$HOME/Library/Application Support/dev.neros.PunchClock/sheet.json`
    /// + Windows: `%APPDATA%\Local\Neros\PunchClock\sheet.json`
    ///
    /// [directories]: https://crates.io/crates/directories
    pub fn default_dir() -> Result<PathBuf, SheetError> {
        ProjectDirs::from("dev", "neros", "PunchClock")
            .ok_or(SheetError::FindSheet)
            .map(|dirs| dirs.data_dir().to_owned())
    }

    /// Get the path to the file the default sheet is stored in.
    ///
    /// This is the file `sheet.json` inside the directory returned from
    /// [`default_dir()`][default].
    ///
    /// [default]: #method.default_dir
    pub fn default_loc() -> Result<PathBuf, SheetError> {
        Self::default_dir().map(|mut dir| {
            dir.push("sheet.json");
            dir
        })
    }

    /// Attempt to write a sheet to the file at the default location, as determined by
    /// [`default_loc()`][default].
    ///
    /// [default]: #method.default_loc
    pub fn write_default(&self) -> Result<(), SheetError> {
        self.write(Self::default_loc()?)
    }

    /// Attempt to write a sheet to the file at the given path.
    pub fn write<P>(&self, path: P) -> Result<(), SheetError>
    where
        P: AsRef<Path>,
    {
        let new_sheet_json = serde_json::to_string(self).unwrap();

        match File::create(&path) {
            Ok(mut sheet_file) => {
                write!(&mut sheet_file, "{}", new_sheet_json).map_err(SheetError::WriteSheet)
            }
            Err(e) => Err(SheetError::WriteSheet(e)),
        }
    }

    /// Record a punch-in (start of a time-tracking period) at the current time.
    pub fn punch_in(&mut self) -> Result<DateTime<Utc>, SheetError> {
        self.punch_in_at(Utc::now())
    }

    /// Record a punch-in (start of a time-tracking period) at the given time.
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

    /// Record a punch-out (end of a time-tracking period) at the current time.
    pub fn punch_out(&mut self) -> Result<DateTime<Utc>, SheetError> {
        self.punch_out_at(Utc::now())
    }

    /// Record a punch-out (end of a time-tracking period) at the given time.
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

    /// Get the current status of time-tracking, including the time at which the status last
    /// changed.
    pub fn status(&self) -> SheetStatus {
        match self.events.last() {
            Some(Event {
                stop: Some(stop), ..
            }) => SheetStatus::PunchedOut(*stop),
            Some(Event { start, .. }) => SheetStatus::PunchedIn(*start),
            None => SheetStatus::Empty,
        }
    }

    /// Count the amount of time for which there was recorded work between the two given instants,
    /// including an ongoing time-tracking period if there is one.
    pub fn count_range(&self, begin: DateTime<Utc>, end: DateTime<Utc>) -> Duration {
        self.events
            .iter()
            .map(|e| (e.start, e.stop.unwrap_or_else(Utc::now)))
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

/// Whether or not time is currently being tracked.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SheetStatus {
    /// Time is currently being tracked, and has been since the given instant.
    PunchedIn(DateTime<Utc>),
    /// Time is not currently being tracked, as of the given instant.
    PunchedOut(DateTime<Utc>),
    /// No time has ever been tracked.
    Empty,
}

/// Errors arising through the use of [`Sheet`][sheet].
///
/// [sheet]: ./struct.Sheet.html
#[derive(Error, Debug)]
pub enum SheetError {
    #[error("already punched in at {0}")]
    PunchedIn(DateTime<Utc>),
    #[error("not punched in, last punched out at {0}")]
    PunchedOut(DateTime<Utc>),
    #[error("not punched in, no punch-ins recorded")]
    NoPunches,
    #[error("unable to find sheet file")]
    FindSheet,
    #[error("unable to open sheet file")]
    OpenSheet(#[source] std::io::Error),
    #[error("unable to read sheet file")]
    ReadSheet(#[source] std::io::Error),
    #[error("unable to parse sheet")]
    ParseSheet(#[source] serde_json::Error),
    #[error("unable to write sheet to file")]
    WriteSheet(#[source] std::io::Error),
}
