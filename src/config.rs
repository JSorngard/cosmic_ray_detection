use clap::{Arg, Command};

const MEMORY_DEFAULT: &str = "1GB";
const DELAY_DEFAULT: &str = "30000";

pub struct Config {
    pub memory_to_occupy: String,
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
                    .help("the size of the memory to monitor for bit flips, understands e.g. 200, 5kB, 2GB and 3Mb")
                    .short('m')
                    .takes_value(true)
                    .required(false)
                    .default_value(MEMORY_DEFAULT),
            )
            .arg(
                Arg::with_name("check_delay")
                    .help("an optional delay in between each integrity check (in milliseconds)")
                    .short('d')
                    .takes_value(true)
                    .required(false)
                    .default_value(DELAY_DEFAULT),
            )
            .arg(
                Arg::with_name("parallel")
                    .help("whether to run the integrity check in parallel to speed it up")
                    .long("parallel")
                    .takes_value(false)
                    .required(false),
            )
            .arg(
                Arg::with_name("quiet")
                    .help("whether to only print information about eventual detections")
                    .long("quiet")
                    .takes_value(false)
                    .required(false),
            )
            .get_matches();

        let parallel = args.is_present("parallel");

        let verbose = !args.is_present("quiet");

        let memory_to_occupy = args.value_of("memory_size").unwrap_or("1GB").to_owned();

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
