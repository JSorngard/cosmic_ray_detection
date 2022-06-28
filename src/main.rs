use rayon::prelude::*;
use std::io::{stdout, Write};
use std::ptr::{read_volatile, write_volatile};
use std::thread::sleep;
use std::time::{Duration, Instant};
use crate::config::Config;

mod config;

fn main() -> Result<(), String> {

    let conf: Config = match Config::new() {
        Ok(c) => c,
        Err(e) => return Err(e),
    };

    let size: usize = match parse_size_string(conf.memory_to_occupy) {
        Ok(s) => s,
        Err(e) => return Err(e),
    };

    let verbose: bool = conf.verbose;

    let sleep_duration: Duration = Duration::from_millis(conf.check_delay);

    if verbose {
        println!("\n------------ Runtime settings ------------");
        println!("Using {} bits of RAM as detector", 8 * size);

        if conf.check_delay == 0 {
            println!("Will do continuous integrity checks");
        } else {
            println!("Waiting {:?} between integrity checks", sleep_duration);
        }
        if conf.parallel {
            println!("Checking memory integrity in parallel");
        }
        println!("------------------------------------------\n");

        print!("Allocating detector memory...");
        flush();
    }

    //Instead of building a detector out of scintillators and photo multiplier tubes,
    //we just allocate some memory on this here computer.
    //Less exciting, much less accurate and sensitive, but much cheaper
    let mut detector: Vec<u8> = vec![0; size];
    detector
        .iter_mut()
        .for_each(|n| unsafe { write_volatile(n, 42) }); //Avoid the pitfalls of virtual memory by writing values to the allocated memory first. Thanks to /u/csdt0 on reddit for this idea.
    if verbose {
        println!("done");
    }

    let start: Instant = Instant::now();

    let mut checks: u64 = 1;
    let mut everything_is_fine: bool;
    if verbose {
        println!("\nBeginning detection loop");
    }
    loop {
        //Reset detector!
        if verbose {
            print!("Zeroing detector memory... ");
            flush();
        }
        if conf.parallel {
            detector.par_iter_mut().for_each(|n| unsafe {write_volatile(n, 0)});
        } else {
            detector.iter_mut().for_each(|n| unsafe {write_volatile(n, 0)});
        }
        everything_is_fine = true;

        //Some feedback for the user that the program is still running
        if verbose {
            println!("done");
            print!("Waiting for first check");
            flush();
        }

        {
            //In order to prevent the optimizer from removing the reads of the memory that make up the detector
            //we will create a reference to it, and use volatile reads on it.
            //thanks to /u/HeroicKatora on reddit for this idea.
            let detector_viewer = &detector;

            while everything_is_fine {
                //We're not gonna miss any events by being too slow
                sleep(sleep_duration);
                //Check if all the bytes are still zero
                everything_is_fine = if conf.parallel {
                    detector_viewer
                        .par_iter()
                        .all(|i| unsafe { read_volatile(i) == 0 })
                } else {
                    detector_viewer
                        .iter()
                        .all(|i| unsafe { read_volatile(i) == 0 })
                };
                if verbose {
                    print!("\rIntegrity checks passed: {}", checks);
                    flush();
                }
                checks += 1;
            }
        }

        println!();

        println!(
            "Detected a bitflip after {:?} on integrity check number {}",
            start.elapsed(),
            checks
        );
        match detector.iter().position(|&r| r != 0) {
            Some(index) => println!("Bit flip in byte {}, it became {}", index, detector[index]),
            None => println!("The same bit flipped back before we could find which one it was! Incredible!"),
        };
        
    }
}

#[inline(always)]
fn flush() {
    stdout().flush().unwrap();
}

fn parse_size_string(size_string: String) -> Result<usize, String> {
    match size_string.parse::<usize>() {
        Ok(t) => Ok(t),
        Err(_) => {
            let chars: Vec<char> = size_string.chars().collect();
            let len: usize = chars.len();
            let last: char = match chars.last() {
                Some(l) => *l,
                None => return Err("memory_to_occupy was empty".to_owned()),
            };

            if (last != 'B' && last != 'b') || len < 2 {
                return Err("memory_to_occupy was incorrectly formatted".to_owned());
            }

            let next_to_last: char = chars[len - 2];

            let si_prefix_factor: f64 = if next_to_last == 'k' {
                1e3
            } else if next_to_last == 'M' {
                1e6
            } else if next_to_last == 'G' {
                1e9
            } else if next_to_last == 'T' {
                //Future proofing...
                1e12
            } else if next_to_last == 'P' {
                //HOW?!
                1e15
            } else if !next_to_last.is_digit(10) {
                return Err("unsupported memory size".to_owned());
            } else {
                return Err("could not parse memory size".to_owned());
            };

            let bit_size: f64 = if last == 'B' { 1.0 } else { 1.0 / 8.0 };

            let factor: usize = (si_prefix_factor * bit_size) as usize;

            let digits: String = chars[..len - 2].iter().collect();
            let number: usize = match digits.parse() {
                Ok(n) => n,
                Err(e) => return Err(e.to_string()),
            };

            Ok(number * factor)
        }
    }
}