use crate::config::Config;
use rayon::prelude::*;
use std::io::{stdout, Write};
use std::ptr::{read_volatile, write_volatile};
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::error::Error;

mod config;

fn main() -> Result<(), Box<dyn Error>> {
    let conf: Config = Config::new()?;

    let size: usize = conf.memory_to_occupy;
    let verbose: bool = conf.verbose;
    let parallel: bool = conf.parallel;

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
    let mut detector: Vec<u8> = vec![0; size];
    //Less exciting, much less accurate and sensitive, but much cheaper

    //Avoid the pitfalls of virtual memory by writing nonzero values to the allocated memory.
    write_to_detector(&mut detector, 42, parallel);

    if verbose {
        println!("done");
        println!("\nBeginning detection loop");
    }

    let mut checks: u64 = 1;
    let mut everything_is_fine: bool;
    let start: Instant = Instant::now();
    loop {
        //Reset detector!
        if verbose {
            print!("Zeroing detector memory... ");
            flush();
        }
        write_to_detector(&mut detector, 0, parallel);
        everything_is_fine = true;

        //Some feedback for the user that the program is still running
        if verbose {
            println!("done");
            print!("Waiting for first check");
            flush();
        }

        while everything_is_fine {
            //We're not gonna miss any events by being too slow
            sleep(sleep_duration);
            //Check if all the bytes are still zero
            everything_is_fine = detector_equals(&detector, 0, parallel);
            if verbose {
                print!("\rIntegrity checks passed: {}", checks);
                flush();
            }
            checks += 1;
        }

        println!(
            "\nDetected a bitflip after {:?} on integrity check number {}",
            start.elapsed(),
            checks
        );
        match detector.iter().position(|&r| r != 0) {
            Some(index) => println!("Bit flip in byte {}, it became {}", index, detector[index]),
            None => println!(
                "The same bit flipped back before we could find which one it was! Incredible!"
            ),
        };
    }
}

//In order to prevent the optimizer from removing the reads of the memory that make up the detector
//we will create a reference to it, and use volatile reads and writes on it.

///Writes the given value to every element of the detector memory.
///Is done in parallel if `parallel` is set to true.
fn write_to_detector(detector: &mut [u8], value: u8, parallel: bool) {
    if parallel {
        detector
            .par_iter_mut()
            .for_each(|n| unsafe { write_volatile(n, value) });
    } else {
        detector
            .iter_mut()
            .for_each(|n| unsafe { write_volatile(n, value) });
    }
}

///Checks if every element of the detector memory is equal to the given value.
///Is done in parallel if `parallel` is set to true.
fn detector_equals(detector: &[u8], value: u8, parallel: bool) -> bool {
    if parallel {
        detector
            .par_iter()
            .all(|i| unsafe { read_volatile(i) == value })
    } else {
        detector
            .iter()
            .all(|i| unsafe { read_volatile(i) == value })
    }
}

#[inline(always)]
fn flush() {
    stdout().flush().unwrap();
}
