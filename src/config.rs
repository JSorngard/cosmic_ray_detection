#[cfg(feature = "rayon")]
use clap::crate_version;
#[cfg(all(not(target_os = "windows"), not(target_os = "freebsd")))]
use clap::ValueEnum;
use clap::{ArgGroup, Parser};
use std::num::NonZeroUsize;
use std::time::Duration;

const DEFAULT_DELAY: &str = "30s";

#[cfg(all(not(target_os = "windows"), not(target_os = "freebsd")))]
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum AllocationMode {
    Available,
    Free,
}

#[cfg(feature = "rayon")]
const LONG_VERSION: &str = concat!(crate_version!(), "\nparallelization enabled");
#[cfg(not(feature = "rayon"))]
const LONG_VERSION: Option<&str> = None;

/// Monitors memory for bit-flips.
/// Won't work on ECC memory, and may not work on DDR5 memory modules and later since they contain onboard ECC.
/// The chance of detection scales with the physical size of your DRAM modules
/// and the percentage of them you allocate to this program.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, long_version = LONG_VERSION)]
#[clap(group(
    ArgGroup::new("detector memory size")
        .required(true)
        .args(&["memory_to_monitor", "use_all"])
))]
pub struct Cli {
    #[arg(short, long, value_parser(parse_memory_string))]
    /// The size of the memory to monitor for bit flips, understands e.g. 200, 5kB, 2GB and 3Mb.
    /// If no suffix is given the program will assume that the given number is the number of bytes to monitor.
    pub memory_to_monitor: Option<NonZeroUsize>,

    // There is a difference between free and available memory,
    // and on most operating systems we can detect this difference.
    // This option lets the user specify which alternative they mean.
    #[cfg(all(not(target_os = "windows"), not(target_os = "freebsd")))]
    #[arg(long, value_enum, value_name = "ALLOCATION_MODE")]
    /// Allocate as much memory as possible to the detector.
    /// If "free" is specified the program will allocate all currently unused memory,
    /// while if "available" is specified the program will also try to eject things that sit in memory
    /// but haven't been used in a while.
    pub use_all: Option<AllocationMode>,

    // On Windows and FreeBSD sysinfo has no way to differentiate free and available memory,
    // so we just allocate as much as the OS gives us.
    #[cfg(any(target_os = "windows", target_os = "freebsd"))]
    #[arg(long)]
    /// Allocate as much memory as possible to the detector.
    pub use_all: bool,

    #[arg(short, value_parser = parse_delay_string, default_value = DEFAULT_DELAY)]
    /// The delay in between each integrity check.
    pub delay: Duration,

    #[arg(short, long)]
    /// Print extra information.
    pub verbose: bool,
}

/// Parses a string describing a number of bytes into an integer.
/// The string can use common SI prefixes as well, like '4GB' or '30kB'.
pub fn parse_memory_string(size_string: &str) -> Result<NonZeroUsize, String> {
    if let Ok(t) = size_string.parse() {
        // The input was a number, interpret it as the number of bytes if nonzero.
        NonZeroUsize::new(t).ok_or_else(|| "zero is not a valid value".to_owned())
    } else {
        // The input was more than just an integer

        // We begin by splitting the string into the number and the suffix.
        let (number, suffix) = match size_string
            .chars()
            .position(|c| !c.is_ascii_digit() && c != '.')
        {
            Some(index) => Ok(size_string.split_at(index)),
            None => Err("you need to specify a suffix to use non-integer numbers".to_owned()),
        }?;

        // Parse the number part
        let mut num_bytes: f64 = number
            .parse()
            .map_err(|_| format!("could not interpret '{number}' as a number"))?;

        if suffix.len() > 2 {
            return Err("the suffix can be at most two letters long".to_owned());
        }

        let mut chars = suffix.chars().rev();

        if let Some(ending) = chars.next() {
            match ending {
                'B' => {
                    if let Some(si_prefix) = chars.next() {
                        num_bytes *= parse_si_prefix(si_prefix)?;
                    }
                }
                'b' => {
                    let si_prefix = chars.next().ok_or_else(|| {
                        "if the suffix ends with 'b' it must be two characters long".to_owned()
                    })?;

                    num_bytes *= parse_si_prefix(si_prefix)? / 8.0;
                }
                _ => {
                    return Err(format!(
                        "the suffix must end with either 'B' or 'b', not '{ending}'"
                    ));
                }
            }
        }

        NonZeroUsize::new(num_bytes as usize)
            .ok_or_else(|| "the size must be at least one byte".to_owned())
    }
}

fn parse_si_prefix(c: char) -> Result<f64, String> {
    match c {
        'k' => Ok(1e3),
        'M' => Ok(1e6),
        'G' => Ok(1e9),
        'T' => Ok(1e12),
        // Values higher than this one should not be needed, but are included for completeness.
        'P' => Ok(1e15),
        'E' => Ok(1e18),
        'Z' => Ok(1e21),
        'Y' => Ok(1e24),
        _ => Err(format!("'{c}' is not a supported SI prefix")),
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
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }

    #[test]
    fn check_memory_parsing() {
        for s in (0..10).map(|i| 2_usize.pow(i)) {
            assert_eq!(parse_memory_string(&format!("{s}")).unwrap().get(), s);
            assert_eq!(
                parse_memory_string(&format!("{s}kB")).unwrap().get(),
                s * 1000
            );
            assert_eq!(
                parse_memory_string(&format!("{s}MB")).unwrap().get(),
                s * 1000000
            );
            assert_eq!(
                parse_memory_string(&format!("{s}GB")).unwrap().get(),
                s * 1000000000
            );
            assert_eq!(
                parse_memory_string(&format!("{s}TB")).unwrap().get(),
                s * 1000000000000
            );
            assert_eq!(
                parse_memory_string(&format!("{s}PB")).unwrap().get(),
                s * 1000000000000000
            );
        }
    }
}
