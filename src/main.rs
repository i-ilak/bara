mod archive;
mod cli;
mod config;
mod content;
mod css;
mod map;
mod projects;
mod templates;
mod util;

use archive::archive;
use clap::Parser;
use cli::Cli;
use config::ConfigFile;
use content::Post;
use css::convert_scss_to_css;
use serde_yaml;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;
use templates::process_jinja;
use util::copy_dir_all;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    // let working_dir = tempdir().expect("Was not able to create temporary directory.");

    let binding = String::from("/Users/iilak/prg/internal/bara/output/");
    let working_dir: &Path = binding.as_ref();

    let config_file_string = fs::read_to_string("/Users/iilak/prg/internal/bara/bara.yml")
        .expect("Could not find config file!");
    let config: ConfigFile = serde_yaml::from_str(&config_file_string)
        .expect("Could not parse config file! Are you sure its valid yaml?");

    let output_dir = config.output.clone();
    let root = config.root.clone();
    let scss_source = config.scss_source.clone();

    // process_jinja(config, working_dir.path().to_path_buf());
    process_jinja(&config, working_dir);
    convert_scss_to_css(&config, working_dir);
    let mut scirpts_dir = working_dir.clone().to_path_buf();
    scirpts_dir.push("scripts");
    let source = PathBuf::from(root.to_string() + "/scripts");
    copy_dir_all(&source, &scirpts_dir);
    Command::new("tsc")
        .arg("--project")
        .arg("/Users/iilak/prg/internal/raw_blog/tsconfig.json")
        .arg("--outDir")
        .arg(binding.clone() + "/scripts")
        .output()?;

    if args.archive {
        // archive(working_dir.path()).expect("Could not create archive!");
        archive(working_dir);
    }

    // copy_dir_all(working_dir.path(), output_dir.as_ref());

    Ok(())
}
