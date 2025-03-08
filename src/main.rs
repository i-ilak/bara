mod archive;
mod build;
mod cli;
mod config;
mod content;
mod create;
mod css;
mod map;
mod markdown_parsing;
mod parse_info;
mod projects;
mod templates;
mod util;
mod watch;
mod typescript_transpile;

use archive::archive;
use build::build;
use parse_info::parse;
use watch::watch;

fn main() {
    let parsed_info = parse();
    if parsed_info.args.watch {
        watch(&parsed_info);
    } else if parsed_info.args.archive {
        archive(&parsed_info).expect("Could not archive current version.");
    } else {
        build(&parsed_info);
    }
}
