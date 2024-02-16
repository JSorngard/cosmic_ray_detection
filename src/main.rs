use std::error::Error;
use std::io::{stdout, Write};
use std::thread::sleep;
use std::time::Instant;

use chrono::Local;
use clap::Parser;
use humantime::format_duration;

mod config;
mod detector;

#[cfg(all(not(target_os = "windows"), not(target_os = "freebsd")))]
use crate::config::AllocationMode;

use crate::{config::Cli, detector::Detector};

fn main() -> Result<(), Box<dyn Error>> {
    let conf = Cli::parse();

    let verbose: bool = conf.verbose;
    let sleep_duration = conf.delay;
    let cr = !conf.log_format;

    if verbose {
        println!("\n------------ Runtime settings ------------");
        println!(
            "Using {} as detector",
            match conf.memory_to_monitor {
                Some(s) => format!("{} bytes", s.get()),
                #[cfg(all(not(target_os = "windows"), not(target_os = "freebsd")))]
                None => match conf.use_all.expect("this only happens if -m wasn't specified, and either -m or --use-all must be specified at the CLI level") {
                    AllocationMode::Available => "as much memory as possible",
                    AllocationMode::Free => "all unused memory",
                }
                .to_owned(),
                #[cfg(any(target_os = "windows", target_os = "freebsd"))]
                None => "as much memory as possible".to_owned(),
            }
        );

        println!(
            "Waiting {} between integrity checks",
            format_duration(sleep_duration)
        );

        println!("------------------------------------------\n");

        print!("Allocating detector memory...");
        stdout().flush()?;
    }

    #[cfg(feature = "rayon")]
    if let Some(jobs) = conf.jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(jobs.into())
            .build_global()
            .map_err(|e| Box::new(e))?;
    }

    // Instead of building a detector out of scintillators and photo-multiplier tubes,
    // we just allocate some memory on this here computer.
    let mut detector = match conf.memory_to_monitor {
        Some(s) => Detector::new(0, s.get()),
        #[cfg(any(target_os = "windows", target_os = "freebsd"))]
        None => Detector::new_with_maximum_size(0),
        #[cfg(all(not(target_os = "windows"), not(target_os = "freebsd")))]
        None => Detector::new_with_maximum_size_in_mode(0, conf.use_all.expect("this only happens if -m wasn't specified, and either -m or --use-all must be specified at the CLI level")),
    };
    // Less exciting, much less accurate and sensitive, but much cheaper

    if verbose {
        print!(" done");
        if conf.memory_to_monitor.is_none() {
            print!(" with allocation of {} bytes", detector.len());
        }
        println!("\nBeginning detection loop");
    }

    let mut checks: u64 = 1;
    let mut memory_is_intact: bool;
    let start: Instant = Instant::now();
    loop {
        // Reset detector!
        if verbose {
            print!("Zeroing detector memory... ");
            stdout().flush()?;
        }
        detector.reset();
        memory_is_intact = true;

        // Some feedback for the user that the program is still running
        if verbose {
            println!("done");
            print!("Waiting for first check");
            stdout().flush()?;
        }

        while memory_is_intact {
            // We're not gonna miss any events by being too slow
            sleep(sleep_duration);
            // Check if all the bytes are still zero
            memory_is_intact = detector.is_intact();
            if memory_is_intact {
                if cr {
                    print!("\r")
                }
                print!("Passed integrity check number {checks} at {}", Local::now());
                if !cr {
                    println!();
                }
                stdout().flush()?;
            }
            checks += 1;
        }

        println!(
            "\nDetected a bitflip after {} on integrity check number {checks} at {}",
            humantime::Duration::from(start.elapsed()),
            Local::now(),
        );

        match detector.position_and_value_of_changed_element() {
            Some((index, value)) => println!(
                "The byte at index {index} flipped from {} to {value}",
                detector.default(),
            ),
            None => println!(
                "The same bit flipped back before we could find which one it was! Incredible!"
            ),
        }
    }
}
