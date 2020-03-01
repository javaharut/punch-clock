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

use crate::Event;

pub type Sheet = Vec<Event>;
