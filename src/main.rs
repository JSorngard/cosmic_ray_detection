use std::error::Error;
use std::io::{stdout, Write};
use std::thread::sleep;
use std::time::Instant;

use clap::Parser;
use humantime::format_duration;

mod config;
mod detector;

use crate::{config::Cli, detector::Detector};

fn main() -> Result<(), Box<dyn Error>> {
    let conf = Cli::parse();

    let verbose: bool = conf.verbose;
    let parallel: bool = conf.parallel;
    let sleep_duration = conf.delay_between_checks;

    if verbose {
        println!("\n------------ Runtime settings ------------");
        println!(
            "Using {} as detector",
            match conf.memory_to_occupy {
                Some(s) => format!("{} bytes", s.get()),
                None => "as many bytes as possible".to_string(),
            }
        );

        println!(
            "Waiting {} between integrity checks",
            format_duration(sleep_duration)
        );

        if parallel {
            println!("Checking memory integrity in parallel");
        }
        println!("------------------------------------------\n");

        print!("Allocating detector memory...");
        stdout().flush()?;
    }

    // Instead of building a detector out of scintillators and photo multiplier tubes,
    // we just allocate some memory on this here computer.
    let mut detector = match conf.memory_to_occupy {
        Some(s) => Detector::new(parallel, 0, s.get()),
        None => Detector::new_with_maximum_size(parallel, 0),
    };
    // Less exciting, much less accurate and sensitive, but much cheaper

    // Avoid the pitfalls of virtual memory by writing nonzero values to the allocated memory.
    detector.write(42);

    if verbose {
        print!(" done");
        if conf.memory_to_occupy.is_none() {
            print!(" with allocation of {} bytes", detector.capacity());
        }
        println!("\nBeginning detection loop");
    }

    let mut checks: u64 = 1;
    let mut everything_is_fine: bool;
    let start: Instant = Instant::now();
    loop {
        // Reset detector!
        if verbose {
            print!("Zeroing detector memory... ");
            stdout().flush()?;
        }
        detector.reset();
        everything_is_fine = true;

        // Some feedback for the user that the program is still running
        if verbose {
            println!("done");
            print!("Waiting for first check");
            stdout().flush()?;
        }

        while everything_is_fine {
            // We're not gonna miss any events by being too slow
            sleep(sleep_duration);
            // Check if all the bytes are still zero
            everything_is_fine = detector.is_intact();
            if verbose {
                print!("\rIntegrity checks passed: {}", checks);
                stdout().flush()?;
            }
            checks += 1;
        }

        println!(
            "\nDetected a bitflip after {:?} on integrity check number {}",
            start.elapsed(),
            checks
        );

        match detector.find_index_of_changed_element() {
            Some(index) => println!(
                "Bit flip in byte at index {}, it became {}",
                index,
                detector
                    .get(index)
                    .expect("already found the index of the value in the detector earlier"),
            ),
            None => println!(
                "The same bit flipped back before we could find which one it was! Incredible!"
            ),
        }
    }
}
