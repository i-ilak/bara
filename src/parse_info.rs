use crate::cli::Cli;
use crate::config::ConfigFile;
use clap::Parser;
use std::env;
use std::fs;
use tempfile::TempDir;

pub struct ParseInfo {
    pub args: Cli,
    pub config: ConfigFile,
    pub working_dir: TempDir,
}

pub fn parse() -> ParseInfo {
    let args = Cli::parse();

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

    let safe_config: ConfigFile = config.expect("Config file could not be parsed!");
    let working_dir = TempDir::new_in(safe_config.clone().root)
        .expect("Was not able to create temporary directory. Check for permissions or similar!");

    ParseInfo {
        args,
        working_dir,
        config: safe_config,
    }
}
