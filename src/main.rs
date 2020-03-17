//  main.rs
//  punch-clock
//
//  Created by Søren Mortensen <soren@neros.dev> on 2019-12-26.
//  Copyright (c) 2019 Søren Mortensen.
//
//  Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
//  http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
//  http://opensource.org/licenses/MIT>, at your option. This file may not be
//  copied, modified, or distributed except according to those terms.

mod opt;

use chrono::{prelude::*, Duration};
use directories::ProjectDirs;
use structopt::StructOpt;

use std::{
    fs::File,
    io::{Read, Write},
};

use punch_clock::{
    sheet::{SheetError, SheetStatus},
    Period, Sheet,
};

use opt::Opt;

const SAME_DAY_FORMAT: &str = "%H:%M:%S";
const DIFF_DAY_FORMAT: &str = "%H:%M:%S on %e %b";

fn main() {
    let opt = Opt::from_args();

    let mut sheet = read_sheet();

    match opt {
        Opt::In { .. } => match sheet.punch_in() {
            Ok(time_utc) => {
                let time_local: DateTime<Local> = time_utc.into();

                println!(
                    "Punching in at {}.",
                    time_local.format("%H:%M:%S").to_string()
                );
            }
            Err(SheetError::PunchedIn(start_utc)) => {
                let start_local: DateTime<Local> = start_utc.into();

                let format = if start_local.date() == Local::today() {
                    SAME_DAY_FORMAT
                } else {
                    DIFF_DAY_FORMAT
                };

                println!(
                    "Can't punch in: already punched in at {}.",
                    start_local.format(format).to_string()
                );
            }
            Err(err) => {
                panic!("Unexpected error while punching in: {}", err);
            }
        },
        Opt::Out { .. } => match sheet.punch_out() {
            Ok(time_utc) => {
                let time_local: DateTime<Local> = time_utc.into();

                println!(
                    "Punching out at {}.",
                    time_local.format("%H:%M:%S").to_string()
                );
            }
            Err(SheetError::PunchedOut(end_utc)) => {
                let end_local: DateTime<Local> = end_utc.into();

                let format = if end_local.date() == Local::today() {
                    SAME_DAY_FORMAT
                } else {
                    DIFF_DAY_FORMAT
                };

                println!(
                    "Can't punch out: already punched out at {}.",
                    end_local.format(format).to_string()
                );
            }
            Err(SheetError::NoPunches) => {
                println!("Can't punch out; no punch-in recorded.");
            }
            Err(err) => {
                panic!("Unexpected error while punching out: {}", err);
            }
        },
        Opt::Status => match sheet.status() {
            SheetStatus::PunchedIn(start_utc) => {
                let start_local: DateTime<Local> = start_utc.into();

                let format = if start_local.date() == Local::today() {
                    SAME_DAY_FORMAT
                } else {
                    DIFF_DAY_FORMAT
                };

                println!(
                    "Punched in since {}.",
                    start_local.format(format).to_string()
                );
            }
            SheetStatus::PunchedOut(end_utc) => {
                let end_local: DateTime<Local> = end_utc.into();

                let format = if end_local.date() == Local::today() {
                    SAME_DAY_FORMAT
                } else {
                    DIFF_DAY_FORMAT
                };

                println!(
                    "Not punched in; last punched out at {}.",
                    end_local.format(format).to_string()
                );
            }
            SheetStatus::Empty => {
                println!("Not punched in; no punch-ins recorded.");
            }
        },
        Opt::Count { period } => {
            if sheet.status() == SheetStatus::Empty {
                println!(
                    "Time worked {}: 0 hours, 0 minutes.",
                    period.to_string().to_lowercase()
                );
            } else {
                let (start, end) = match period {
                    Period::All => (sheet.events[0].start, Utc::now()),
                    Period::Today => {
                        let end_local = Local::now();
                        let end_utc: DateTime<Utc> = end_local.into();
                        let start_local = Local::today().and_hms(0, 0, 0);

                        let span = end_local - start_local;
                        let start_utc = end_utc - span;

                        (start_utc, end_utc)
                    }
                    Period::Yesterday => {
                        let end_local = Local::today().and_hms(0, 0, 0);
                        let end_utc: DateTime<Utc> = end_local.into();
                        let start_local = Local::today().pred().and_hms(0, 0, 0);

                        let span = end_local - start_local;
                        let start_utc = end_utc - span;

                        (start_utc, end_utc)
                    }
                    Period::Week => {
                        let mut last_monday = Local::today();
                        while last_monday.weekday() != Weekday::Mon {
                            last_monday = last_monday.pred();
                        }

                        let start_local = last_monday.and_hms(0, 0, 0);
                        let end_local = Local::now();
                        let end_utc: DateTime<Utc> = end_local.into();

                        let span = end_local - start_local;
                        let start_utc = end_utc - span;

                        (start_utc, end_utc)
                    }
                    Period::LastWeek => {
                        let mut last_monday = Local::today();
                        while last_monday.weekday() != Weekday::Mon {
                            last_monday = last_monday.pred();
                        }

                        let mut monday_before = last_monday.pred();
                        while monday_before.weekday() != Weekday::Mon {
                            monday_before = monday_before.pred();
                        }

                        let start_local = monday_before.and_hms(0, 0, 0);
                        let end_local = last_monday.and_hms(0, 0, 0);
                        let end_utc: DateTime<Utc> = end_local.into();

                        let span = end_local - start_local;
                        let start_utc = end_utc - span;

                        (start_utc, end_utc)
                    }
                    Period::Month => {
                        let now = Local::now();
                        let month_first = Local.ymd(now.year(), now.month(), 1);

                        let start_local = month_first.and_hms(0, 0, 0);
                        let end_local = now;
                        let end_utc: DateTime<Utc> = end_local.into();

                        let span = end_local - start_local;
                        let start_utc = end_utc - span;

                        (start_utc, end_utc)
                    }
                    Period::LastMonth => {
                        let today = Local::today();
                        let month_first = Local.ymd(today.year(), today.month(), 1);

                        let day_before = month_first - Duration::days(1);
                        let last_month_first = Local.ymd(day_before.year(), day_before.month(), 1);

                        let start_local = last_month_first.and_hms(0, 0, 0);
                        let end_local = month_first.and_hms(0, 0, 0);
                        let end_utc: DateTime<Utc> = end_local.into();

                        let span = end_local - start_local;
                        let start_utc = end_utc - span;

                        (start_utc, end_utc)
                    }
                };

                let total = sheet.count_range(start, end);

                println!(
                    "Time worked {}: {} hours, {} minutes.",
                    period.to_string().to_lowercase(),
                    total.num_hours(),
                    total.num_minutes() - total.num_hours() * 60,
                );
            }
        }
    }

    overwrite_sheet(&sheet);
}

fn read_sheet() -> Sheet {
    let project_dirs = ProjectDirs::from("dev", "neros", "PunchClock")
        .expect("Unable to locate project data directory for punch-clock");
    let data_dir = project_dirs.data_dir().to_owned();

    let mut sheet_path = data_dir.clone();
    sheet_path.push("sheet.json");

    let mut sheet_json = String::new();

    {
        let mut sheet_file = File::open(&sheet_path)
            .or_else(|_| File::create(&sheet_path).and_then(|_| File::open(&sheet_path)))
            .or_else(|_| {
                std::fs::create_dir_all(&data_dir)
                    .and_then(|_| File::create(&sheet_path))
                    .and_then(|_| File::open(&sheet_path))
            })
            .expect("Unable to find or create sheet.json");

        sheet_file
            .read_to_string(&mut sheet_json)
            .expect("Unable to read contents of sheet.json");
    }

    if sheet_json.is_empty() {
        Sheet::default()
    } else {
        serde_json::from_str(&sheet_json).expect("Unable to parse contents of sheet.json")
    }
}

fn overwrite_sheet(sheet: &Sheet) {
    let project_dirs = ProjectDirs::from("dev", "neros", "PunchClock")
        .expect("Unable to locate project data directory for punch-clock");

    let mut sheet_path = project_dirs.data_dir().to_owned();
    sheet_path.push("sheet.json");

    let mut sheet_file =
        File::create(&sheet_path).expect("Unable to open sheet.json for overwriting.");

    let new_sheet_json = serde_json::to_string(&sheet).unwrap();

    write!(&mut sheet_file, "{}", new_sheet_json).unwrap_or_else(|_| {
        panic!(
            "Unable to write updated timesheet. Just in case, here's what the contents should \
             have been: {}",
            new_sheet_json
        )
    });
}
