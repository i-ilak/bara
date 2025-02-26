use temp_dir::TempDir;
use crate::cli::Cli;
use clap::Parser;
use crate::config::ConfigFile;
use serde_yaml;
use std::fs;

pub struct ParseInfo {
    pub args: Cli,
    pub config: ConfigFile,
    pub working_dir: TempDir,
}

pub fn parse() -> ParseInfo {
    let args = Cli::parse();

    let working_dir = TempDir::new().expect("Was not able to create temporary directory.");

    let config: ConfigFile = serde_yaml::from_str(
        &fs::read_to_string(args.config.clone()).expect("Could not find config file!"),
    )
    .expect("Could not parse config file! Are you sure its valid yaml?");

    ParseInfo{args, working_dir, config}
}