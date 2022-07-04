use clap::{Arg, Command};

const MEMORY_DEFAULT: &str = "1GB";
const DELAY_DEFAULT: &str = "30000";

pub struct Config {
    pub memory_to_occupy: usize,
    pub check_delay: u64,
    pub parallel: bool,
    pub verbose: bool,
}

impl Config {
    pub fn new() -> Result<Self, String> {
        let args = Command::new("cosmic ray detector")
            .about("Monitors memory for bit-flips (won't work on ECC memory). The chance of detection scales with the physical size of your DRAM modules and the percentage of them you allocate to this program.")
            .version("v1.0.2")
            .author("Johanna Sörngård (jsorngard@gmail.com)")
            .arg(
                Arg::with_name("memory_size")
                    .help("The size of the memory to monitor for bit flips, understands e.g. 200, 5kB, 2GB and 3Mb")
                    .short('m')
                    .takes_value(true)
                    .required(false)
                    .default_value(MEMORY_DEFAULT),
            )
            .arg(
                Arg::with_name("check_delay")
                    .help("An optional delay in between each integrity check (in milliseconds)")
                    .short('d')
                    .takes_value(true)
                    .required(false)
                    .default_value(DELAY_DEFAULT),
            )
            .arg(
                Arg::with_name("parallel")
                    .help("Whether to run the integrity check in parallel to speed it up")
                    .long("parallel")
                    .takes_value(false)
                    .required(false),
            )
            .arg(
                Arg::with_name("quiet")
                    .help("Whether to only print information about eventual detections")
                    .long("quiet")
                    .takes_value(false)
                    .required(false),
            )
            .get_matches();

        let parallel = args.is_present("parallel");

        let verbose = !args.is_present("quiet");

        let memory_to_occupy = match parse_size_string(args.value_of("memory_size").unwrap_or("1GB").to_owned()) {
            Ok(s) => s,
            Err(e) => return Err(e),
        };

        let check_delay: u64 = match args.value_of("check_delay") {
            Some(s) => match s.parse() {
                Ok(t) => t,
                Err(e) => return Err(e.to_string()),
            },
            None => 0,
        };

        Ok(Config {
            memory_to_occupy,
            check_delay,
            parallel,
            verbose,
        })
    }
}

///Parses a string describing a number of bytes into an integer.
///The string can use common SI prefixes as well, like '4GB' or '30kB'.
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