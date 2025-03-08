use crate::css::convert_scss_to_css;
use crate::parse_info::ParseInfo;
use crate::templates::process_jinja;
use crate::util::copy_dir_all;
use crate::typescript_transpile::transpile_typescript_files;

use futures::join;

pub fn create(parsed_info: &ParseInfo) {
    let config = &parsed_info.config;
    let working_dir = &parsed_info.working_dir;
    futures::executor::block_on(async {
        let jinja_handle = process_jinja(&config, working_dir.path());
        let scss_handle = convert_scss_to_css(&config, working_dir.path());
        let transpiler = transpile_typescript_files(
            &config, 
            &working_dir.path());

        join!(jinja_handle, scss_handle, transpiler);
    });
    std::fs::copy(
        &config.root.join("robots.txt"), 
        working_dir.path().join("robots.txt"))
        .expect("Could not copy robots.txt");
    copy_dir_all(&config.static_dir, &working_dir.path().join("static"));
    copy_dir_all(
        &config.time_machine,
        &working_dir.path().join("time_machine"),
    );
}
