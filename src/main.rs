/*
 * Copyright (C) 2025 spekie
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::time::{SystemTime, UNIX_EPOCH};

const ARRAY_MIN: usize = 1024;
const ARRAY_MAX: usize = 4096 * 4096;

static mut X: [usize; ARRAY_MAX] = [0; ARRAY_MAX];

fn get_seconds() -> f64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    now.as_secs() as f64 + now.subsec_nanos() as f64 / 1e9
}

fn label(bytes: usize) {
    if bytes < 1_000 {
        print!("{:>4}B,", bytes);
    } else if bytes < 1_000_000 {
        print!("{:>4}K,", bytes / 1024);
    } else {
        print!("{:>4}M,", bytes / (1024 * 1024));
    }
}

fn main() {
    unsafe {
        // print header for stride sizes
        print!(",");
        let mut stride = 1;
        while stride <= ARRAY_MAX / 2 {
            label(stride * std::mem::size_of::<usize>());
            stride *= 2;
        }
        println!();

        // loops over cache sizes
        let mut csize = ARRAY_MIN;
        while csize <= ARRAY_MAX {
            label(csize * std::mem::size_of::<usize>());

            let mut stride = 1;
            while stride <= csize / 2 {
                // sets pointer-chasing pattern
                let mut index = 0;
                while index < csize {
                    X[index] = index + stride;
                    index += stride;
                }
                X[index - stride] = 0; // loop back

                // sync
                let lastsec = get_seconds();
                let mut sec0 = get_seconds();
                while sec0 == lastsec {
                    sec0 = get_seconds();
                }

                let mut steps = 0.0;
                let mut nextstep;

                sec0 = get_seconds();
                loop {
                    for _ in 0..stride {
                        nextstep = 0;
                        loop {
                            nextstep = X[nextstep];
                            if nextstep == 0 {
                                break;
                            }
                        }
                    }
                    steps += 1.0;
                    let sec1 = get_seconds();
                    if sec1 - sec0 >= 20.0 {
                        break;
                    }
                }
                let mut sec = get_seconds() - sec0;

                // loop overhead
                let overhead_sec0 = get_seconds(); // fix
                let mut tsteps = 0.0;
                loop {
                    for _ in 0..stride {
                        let mut index = 0;
                        while index < csize {
                            index += stride;
                        }
                    }
                    tsteps += 1.0;
                    if tsteps >= steps {
                        break;
                    }
                }
                let overhead_sec1 = get_seconds();
                sec -= overhead_sec1 - overhead_sec0;

                let loadtime = (sec * 1e9) / (steps * csize as f64);
                print!("{:4.1},", if loadtime < 0.1 { 0.1 } else { loadtime });

                stride *= 2;
            }

            println!();
            csize *= 2;
        }
    }
}
