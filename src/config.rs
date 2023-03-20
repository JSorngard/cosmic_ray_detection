use clap::{ArgGroup, Parser};
use std::num::NonZeroUsize;
use std::time::Duration;

const DEFAULT_DELAY: &str = "30s";

/// Monitors memory for bit-flips (won't work on ECC memory).
/// The chance of detection scales with the physical size of your DRAM modules
/// and the percentage of them you allocate to this program.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(group(
    ArgGroup::new("detector memory size")
        .required(true)
        .args(&["memory_to_occupy", "maximize_memory"])
))]
pub struct Cli {
    #[arg(short, value_parser(parse_size_string))]
    /// The size of the memory to monitor for bit flips, understands e.g. 200, 5kB, 2GB and 3MB.
    /// If no suffix is given the program will assume that the given number is the number of bytes to monitor.
    pub memory_to_occupy: Option<NonZeroUsize>,

    #[arg(long)]
    /// Allocate as much memory as possible to the detector.
    pub maximize_memory: bool,

    #[arg(short, value_parser = parse_delay_string, default_value = DEFAULT_DELAY)]
    /// The delay in between each integrity check.
    pub delay_between_checks: Duration,

    #[arg(long)]
    /// Run the integrity check in parallel.
    pub parallel: bool,

    #[arg(short, long)]
    /// Print extra information.
    pub verbose: bool,
}

/// Parses a string describing a number of bytes into an integer.
/// The string can use common SI prefixes as well, like '4GB' or '30kB'.
pub fn parse_size_string(size_string: &str) -> Result<NonZeroUsize, String> {
    match size_string.parse() {
        // The input was a number, interpret it as the number of bytes if nonzero.
        Ok(t) => NonZeroUsize::new(t).ok_or_else(|| "zero is not a valid value".into()),
        // The input was more than just a number
        Err(_) => {
            // Find index of first suffix letter
            let (number, suffix) = size_string.split_at(
                size_string
                    .chars()
                    .position(|c| !c.is_ascii_digit())
                    .expect("in this match arm there should be some non-digit in the string"),
            );

            let mut num_bytes: f64 = number
                .parse()
                .map_err(|_| format!("could not interpret '{number}' as a number"))?;

            for c in suffix.chars() {
                num_bytes *= parse_memory_modifier(c)?;
            }

            NonZeroUsize::new(num_bytes as usize).ok_or_else(|| "zero is not a valid value".into())
        }
    }
}

fn parse_memory_modifier(c: char) -> Result<f64, String> {
    if c == 'B' {
        Ok(1.0)
    } else if c == 'k' {
        Ok(1e3)
    } else if c == 'M' {
        Ok(1e6)
    } else if c == 'G' {
        Ok(1e9)
    } else if c == 'T' {
        Ok(1e12)
    } else if c == 'P' {
        // Values higher than this one should not be needed, but are included for completeness.
        Ok(1e15)
    } else if c == 'E' {
        Ok(1e18)
    } else if c == 'Z' {
        Ok(1e21)
    } else if c == 'Y' {
        Ok(1e24)
    } else {
        Err(format!("'{c}' is an unsupported suffix component"))
    }
}

fn parse_delay_string(s: &str) -> Result<Duration, String> {
    match s.parse::<humantime::Duration>() {
        Ok(d) => Ok(d.into()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod test {
    use super::parse_size_string;

    #[test]
    fn check_memory_parsing() {
        for s in (0..10).map(|i| 2_usize.pow(i)) {
            assert_eq!(parse_size_string(&format!("{s}")).unwrap().get(), s);
            assert_eq!(
                parse_size_string(&format!("{s}kB")).unwrap().get(),
                s * 1000
            );
            assert_eq!(
                parse_size_string(&format!("{s}MB")).unwrap().get(),
                s * 1000000
            );
            assert_eq!(
                parse_size_string(&format!("{s}GB")).unwrap().get(),
                s * 1000000000
            );
            assert_eq!(
                parse_size_string(&format!("{s}TB")).unwrap().get(),
                s * 1000000000000
            );
            assert_eq!(
                parse_size_string(&format!("{s}PB")).unwrap().get(),
                s * 1000000000000000
            );
        }
    }
}
