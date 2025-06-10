/*
 * Copyright (c) 2025 spekie
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 * 1. Redistributions of source code must retain the above copyright notice, this
 *    list of conditions and the following disclaimer.
 *
 * 2. Redistributions in binary form must reproduce the above copyright notice,
 *    this list of conditions and the following disclaimer in the documentation
 *    and/or other materials provided with the distribution.
 *
 * 3. Neither the name of the copyright holder nor the names of its
 *    contributors may be used to endorse or promote products derived from
 *    this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
 * FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
 * SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
 * CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
 * OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
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
