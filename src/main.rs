mod archive;
mod cli;
mod config;
mod content;
mod css;
mod map;
mod projects;
mod swc;
mod templates;
mod util;

use archive::archive;
use clap::Parser;
use cli::Cli;
use config::ConfigFile;
use css::convert_scss_to_css;
use serde_yaml;
use std::fs;
use std::path::{Path, PathBuf};
use swc::compile_folder;
use temp_dir::TempDir;
use templates::process_jinja;
use util::{copy_dir_all, patch_basepath};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let working_dir = TempDir::new().expect("Was not able to create temporary directory.");

    let config_file_string = fs::read_to_string("/Users/iilak/prg/internal/bara/bara.yml")
        .expect("Could not find config file!");
    let config: ConfigFile = serde_yaml::from_str(&config_file_string)
        .expect("Could not parse config file! Are you sure its valid yaml?");

    process_jinja(&config, working_dir.path());
    convert_scss_to_css(&config, working_dir.path());
    let mut static_dir = working_dir.path().to_path_buf();
    static_dir.push("static");
    copy_dir_all(&PathBuf::from(config.static_dir), &static_dir);

    let mut scripts_dir = working_dir.path().to_path_buf();
    scripts_dir.push("scripts");
    compile_folder(&PathBuf::from(config.scripts), &scripts_dir);

    if args.archive {
        archive(working_dir.path());
    }

    let binding = String::from("/Users/iilak/prg/internal/bara/output/");
    let output_dir: &Path = binding.as_ref();

    patch_basepath(working_dir.path());
    copy_dir_all(working_dir.path(), output_dir).unwrap();

    Ok(())
}
