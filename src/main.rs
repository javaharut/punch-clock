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

mod config;
mod opt;

use chrono::Local;
use structopt::StructOpt;

use opt::Opt;

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
            println!("Time worked {}: none.", period.to_string().to_lowercase());
        }
    }
}
