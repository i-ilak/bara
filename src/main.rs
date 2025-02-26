mod archive;
mod cli;
mod config;
mod content;
mod css;
mod map;
mod markdown_parsing;
mod projects;
mod templates;
mod util;
mod parse_info;

use archive::archive;
use css::convert_scss_to_css;
use futures::join;
use std::path::PathBuf;
use templates::process_jinja;
use parse_info::{parse, ParseInfo};
use util::{copy_dir_all, patch_basepath};

fn create(parsed_info: &ParseInfo) {
    let config = &parsed_info.config;
    let working_dir = &parsed_info.working_dir;
    futures::executor::block_on(async {
        let jinja_handle = process_jinja(&config, working_dir.path());
        let scss_handle = convert_scss_to_css(&config, working_dir.path());

        join!(jinja_handle, scss_handle);
    });
    copy_dir_all(&config.static_dir, &working_dir.path().join("static"));
    copy_dir_all(
        &config.time_machine,
        &working_dir.path().join("time_machine"),
    );
}

fn patch(parsed_info: &ParseInfo) {
    let config = &parsed_info.config;
    let working_dir = &parsed_info.working_dir;

    patch_basepath(working_dir.path());
    let output_dir = PathBuf::from(&config.output);
    copy_dir_all(working_dir.path(), &output_dir);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parsed_info = parse();
    create(&parsed_info);

    if parsed_info.args.archive {
        archive(&parsed_info).expect("Could not archive current version.");
        return Ok(());
    }
    else {
        patch(&parsed_info);
        Ok(())
    }
}
