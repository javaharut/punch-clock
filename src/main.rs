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

use chrono::{DateTime, Local, Utc};
use structopt::StructOpt;

use opt::Opt;

use punch_clock::{Event, Sheet};

fn main() {
    use std::{fs::File, io::Read, path::PathBuf};

    let opt = Opt::from_args();

    let home_dir = env!("HOME");
    if home_dir.is_empty() {
        panic!("Value of $HOME environment variable not valid.");
    }

    let mut sheet_dir = PathBuf::from(home_dir);
    sheet_dir.push(".local/share/punch");

    let mut sheet_path = sheet_dir.clone();
    sheet_path.push("sheet.json");

    let mut sheet_json = String::new();

    {
        let mut sheet_file = File::open(&sheet_path)
            .or_else(|_| File::create(&sheet_path).and_then(|_| File::open(&sheet_path)))
            .or_else(|_| {
                std::fs::create_dir_all(&sheet_dir)
                    .and_then(|_| File::create(&sheet_path))
                    .and_then(|_| File::open(&sheet_path))
            })
            .expect("Unable to find or create sheet.json");

        sheet_file
            .read_to_string(&mut sheet_json)
            .expect("Unable to read contents of sheet.json");
    }

    let mut sheet: Sheet = serde_json::from_str(&sheet_json).unwrap_or_default();

    match opt {
        Opt::In { time: _ } => {
            let time_local = Local::now();

            println!(
                "Punching in at {}.",
                time_local.format("%H:%M:%S").to_string()
            );

            let time_utc: DateTime<Utc> = time_local.into();
            let event = Event::Start(time_utc);

            sheet.push(event);
        }
        Opt::Out { time: _ } => {
            let time_local = Local::now();

            println!(
                "Punching out at {}.",
                time_local.format("%H:%M:%S").to_string()
            );

            let time_utc: DateTime<Utc> = time_local.into();
            let event = Event::Stop(time_utc);

            sheet.push(event);
        }
        Opt::Status => match sheet.last() {
            Some(Event::Stop(_)) | None => {
                println!("Not tracking time.");
            }
            Some(Event::Start(time)) => {
                println!("Started tracking time at {}.", time);
            }
        },
        Opt::Count { period } => {
            println!("Time worked {}: none.", period.to_string().to_lowercase());
        }
    }

    let mut sheet_file =
        File::create(&sheet_path).expect("Unable to open sheet.json for overwriting.");

    use std::io::Write;

    let new_sheet_json = serde_json::to_string(&sheet).unwrap();

    write!(&mut sheet_file, "{}", new_sheet_json).expect("Uh oh");
}
