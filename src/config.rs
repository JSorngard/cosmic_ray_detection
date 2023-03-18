use clap::Parser;
use std::num::NonZeroUsize;
use std::num::ParseIntError;

const DELAY_DEFAULT: u64 = 30000;

///Monitors memory for bit-flips (won't work on ECC memory).
///The chance of detection scales with the physical size of your DRAM modules
///and the percentage of them you allocate to this program.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, value_parser(parse_size_string))]
    /// The size of the memory to monitor for bit flips, understands e.g. 200, 5kB, 2GB and 3Mb.
    /// If this argument is not provided the program will dynamically try to allocate as much as it can
    /// (expect it to be able to allocate roughly half of your memory).
    pub memory_to_occupy: Option<NonZeroUsize>,

    #[arg(short, default_value_t = DELAY_DEFAULT)]
    ///An optional delay in between each integrity check (in milliseconds)
    pub delay_between_checks: u64,

    #[arg(long)]
    ///Whether to run the integrity check in parallel to speed it up
    pub parallel: bool,

    #[arg(short, long)]
    ///Whether to print extra information"
    pub verbose: bool,
}

///Parses a string describing a number of bytes into an integer.
///The string can use common SI prefixes as well, like '4GB' or '30kB'.
pub fn parse_size_string(size_string: &str) -> Result<NonZeroUsize, String> {
    match size_string.parse::<NonZeroUsize>() {
        Ok(t) => Ok(t),
        Err(_) => {
            let chars: Vec<char> = size_string.chars().collect();
            let len: usize = chars.len();
            let last: char = match chars.last() {
                Some(l) => *l,
                None => return Err("memory_to_occupy was empty".into()),
            };

            if last == '0' {
                return Err("must occupy a nonzero amount of memory".into());
            }

            if (last != 'B' && last != 'b') || len < 2 {
                return Err("memory_to_occupy was incorrectly formatted".into());
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
                return Err("unsupported memory size".into());
            } else {
                return Err("could not parse memory size".into());
            };

            let bit_size: f64 = if last == 'B' { 1.0 } else { 1.0 / 8.0 };

            let factor: usize = (si_prefix_factor * bit_size) as usize;

            let digits: String = chars[..len - 2].iter().collect();
            let number: usize = digits.parse().map_err(|e: ParseIntError| e.to_string())?;

            NonZeroUsize::new(number * factor).ok_or_else(|| "zero is not a valid value".into())
        }
    }
}
