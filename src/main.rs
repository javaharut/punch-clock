mod opt;

use chrono::{prelude::*, Duration};
use directories::ProjectDirs;
use opt::Opt;
use punch_clock::{
    sheet::{SheetError, SheetStatus},
    Period, Sheet,
};
use structopt::StructOpt;

const SAME_DAY_FORMAT: &str = "%H:%M:%S";
const DIFF_DAY_FORMAT: &str = "%H:%M:%S on %e %b";

fn main() {
    let opt = Opt::from_args();

    // Try to load the sheet from the default location. If loading fails due to a missing file,
    // create a new empty sheet.
    let mut sheet = Sheet::load_default()
        .or_else(|err| match err {
            SheetError::OpenSheet(io_err) if io_err.raw_os_error() == Some(2) => {
                Ok(Sheet::default())
            }
            _ => Err(err),
        })
        .unwrap();

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

    // Try to write the sheet to the default location. If loading fails due to a missing directory,
    // create the directory.
    sheet
        .write_default()
        .or_else(|err| match err {
            SheetError::WriteSheet(io_err) if io_err.raw_os_error() == Some(2) => {
                let dd = ProjectDirs::from("dev", "neros", "PunchClock")
                    .expect("Unable to locate data directory for punch-clock.")
                    .data_dir()
                    .to_owned();

                std::fs::create_dir(dd).expect("Unable to create data directory for punch-clock.");
                sheet.write_default()
            }
            _ => Err(err),
        })
        .unwrap();
}
