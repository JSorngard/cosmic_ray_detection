use clap::Parser;
use std::error::Error;
use std::fmt;

const MEMORY_DEFAULT: &str = "1GB";
const DELAY_DEFAULT: u64 = 30000;

///Monitors memory for bit-flips (won't work on ECC memory).
///The chance of detection scales with the physical size of your DRAM modules
///and the percentage of them you allocate to this program.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(short, value_parser,  default_value_t = MEMORY_DEFAULT.to_owned(), help = "The size of the memory to monitor for bit flips, understands e.g. 200, 5kB, 2GB and 3Mb")]
    pub memory_to_occupy: String,

    #[clap(short, value_parser, default_value_t = DELAY_DEFAULT, help = "An optional delay in between each integrity check (in milliseconds)")]
    pub delay_between_checks: u64,

    #[clap(
        long,
        help = "Whether to run the integrity check in parallel to speed it up"
    )]
    pub parallel: bool,

    #[clap(short, long, help = "Whether to print extra information")]
    pub verbose: bool,
}

///Parses a string describing a number of bytes into an integer.
///The string can use common SI prefixes as well, like '4GB' or '30kB'.
pub fn parse_size_string(size_string: String) -> Result<usize, Box<dyn Error>> {
    match size_string.parse::<usize>() {
        Ok(t) => Ok(t),
        Err(_) => {
            let chars: Vec<char> = size_string.chars().collect();
            let len: usize = chars.len();
            let last: char = match chars.last() {
                Some(l) => *l,
                None => {
                    return Err(Box::new(MemoryStringError::new(
                        "memory_to_occupy was empty".to_owned(),
                    )))
                }
            };

            if (last != 'B' && last != 'b') || len < 2 {
                return Err(Box::new(MemoryStringError::new(
                    "memory_to_occupy was incorrectly formatted".to_owned(),
                )));
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
            } else if !next_to_last.is_ascii_digit() {
                return Err(Box::new(MemoryStringError::new(
                    "unsupported memory size".to_owned(),
                )));
            } else {
                return Err(Box::new(MemoryStringError::new(
                    "could not parse memory size".to_owned(),
                )));
            };

            let bit_size: f64 = if last == 'B' { 1.0 } else { 1.0 / 8.0 };

            let factor: usize = (si_prefix_factor * bit_size) as usize;

            let digits: String = chars[..len - 2].iter().collect();
            let number: usize = digits.parse()?;

            Ok(number * factor)
        }
    }
}

#[derive(Debug, Clone)]
struct MemoryStringError {
    message: String,
}

impl Error for MemoryStringError {}

impl MemoryStringError {
    fn new(message: String) -> Self {
        MemoryStringError { message }
    }
}

impl fmt::Display for MemoryStringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
