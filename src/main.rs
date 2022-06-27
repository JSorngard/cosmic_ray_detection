use clap::{Arg, Command};
use rayon::prelude::*;
use std::io::{self, Write};
use std::thread::sleep;
use std::time::{Duration, Instant};

fn main() {
    let conf = Config::new();

    let size: usize = parse_size_string(conf.memory_to_occupy);

    print!("Allocating {} bits of detector RAM... ", size);
    io::stdout().flush().unwrap();
    let detector_mass: Vec<bool> = vec![true; size];
    println!("done");

    let start = Instant::now();
    let sleep_duration = Duration::from_millis(conf.check_delay);

    if conf.check_delay == 0 {
        println!("Will do continuous integrity checks");
    } else {
        println!("Waiting {:?} between integrity checks", sleep_duration);
    }
    let mut checks = 1;
    if conf.parallel {
        println!("Running checks in parallel");
        while detector_mass.par_iter().all(|i| *i) {
            sleep(sleep_duration);
            print!("\rChecks completed: {}", checks);
            io::stdout().flush().unwrap();
            checks += 1;
        }
    } else {
        while detector_mass.iter().all(|i| *i) {
            sleep(sleep_duration);
            print!("\rChecks completed: {}", checks);
            io::stdout().flush().unwrap();
            checks += 1;
        }
    }

    println!(
        "Detected a bitflip after {:?} on the {} integrity check",
        start.elapsed(),
        checks
    );
    let location = detector_mass.iter().position(|&r| !r).unwrap() + 1;
    println!("It was the {}:th boolean that flipped", location);
}

fn parse_size_string(size_string: String) -> usize {
    match size_string.parse() {
        Ok(t) => t,
        Err(_) => {
            let chars: Vec<char> = size_string.chars().collect();
            let len = chars.len();
            //unwrap is okay, because clap doesn't let the program run without input in this variable
            let last = *chars.last().unwrap();
            if (last != 'B' && last != 'b') || len < 2 {
                panic!("memory_to_occupy was incorrectly formatted");
            }
            let next_to_last = chars[len - 2];

            let si_prefix_factor = if next_to_last == 'k' {
                1e3
            } else if next_to_last == 'M' {
                1e6
            } else if next_to_last == 'G' {
                1e9
            } else if next_to_last == 'T' {
                1e12
            } else if !next_to_last.is_digit(10) {
                panic!("unsupported memory size");
            } else {
                panic!("could not parse memory size");
            };

            let bit_size = if last == 'B' { 1.0 } else { 1.0 / 8.0 };

            //unwrap is okay because si_prefix_factor always fits in an f64
            let factor: usize = (f64::try_from(si_prefix_factor).unwrap() * bit_size) as usize;

            let digits: String = chars[..len - 2].into_iter().collect();
            let number: usize = match digits.parse() {
                Ok(n) => n,
                Err(e) => panic!("{}", e),
            };

            number * factor
        }
    }
}

struct Config {
    memory_to_occupy: String,
    check_delay: u64,
    parallel: bool,
}

impl Config {
    fn new() -> Self {
        let args = Command::new("cosmic ray detector")
            .about("monitors memory for bit-flips (won't work on ECC memory)")
            .version("v0.1.0")
            .author("Johanna Sörngård (jsorngard@gmail.com)")
            .arg(
                Arg::with_name("memory_size")
                    .help("the size of the memory to monitor for bit flips")
                    .short('m')
                    .takes_value(true)
                    .required(true),
            )
            .arg(
                Arg::with_name("check_delay")
                    .help("an optional delay in between each integrity check (in milliseconds)")
                    .short('d')
                    .takes_value(true)
                    .required(false),
            )
            .arg(
                Arg::with_name("parallel")
                    .help("whether to run the integrity check in parallel to speed it up")
                    .takes_value(false)
                    .required(false),
            )
            .get_matches();

        let parallel = args.is_present("parallel");

        let memory_to_occupy = args.value_of("memory_size").unwrap().to_owned();

        let check_delay: u64 = match args.value_of("check_delay") {
            Some(s) => match s.parse() {
                Ok(t) => t,
                Err(e) => panic!("could not parse check delay: {}", e),
            },
            None => 0,
        };

        Config {
            memory_to_occupy,
            check_delay,
            parallel,
        }
    }
}
