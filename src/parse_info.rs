use crate::cli::Cli;
use crate::config::ConfigFile;
use clap::Parser;
use std::env;
use std::fs;
use temp_dir::TempDir;

pub struct ParseInfo {
    pub args: Cli,
    pub config: ConfigFile,
    pub working_dir: TempDir,
}

pub fn parse() -> ParseInfo {
    let args = Cli::parse();

    let working_dir = TempDir::new()
        .expect("Was not able to create temporary directory. Check for permissions or similar!");

    let config = match args.config {
        Some(ref path) => {
            serde_yaml::from_str(&fs::read_to_string(path).expect("Cloud not read config file!"))
        }
        None => serde_yaml::from_str(
            &fs::read_to_string(
                env::current_dir()
                    .expect("Cloud not get executable dir!")
                    .join("bara.yml"),
            )
            .expect("Could not find config file!"),
        ),
    };

    ParseInfo {
        args,
        working_dir,
        config: config.expect("Reason"),
    }
}
