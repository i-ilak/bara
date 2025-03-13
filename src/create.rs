use crate::css::convert_scss_to_css;
use crate::parse_info::ParseInfo;
use crate::templates::process_jinja;
use crate::util::copy_dir_all;

use futures::join;

fn copy_highlight_related_code(parsed_info: &ParseInfo) {
    let config = &parsed_info.config;
    let working_dir = &parsed_info.working_dir;
    copy_dir_all(
        &config.root.join("static/highlightjs_styles"),
        &working_dir.path().join("static/highlightjs_styles"),
    );
    std::fs::create_dir(working_dir.path().join("scripts"));
    std::fs::copy(
        config.root.join("scripts/highlight.min.js"),
        working_dir.path().join("scripts/highlight.min.js"),
    )
    .expect("Could not copy highlight.min.js!");
    std::fs::copy(
        config.root.join("scripts/highlightjs-line-numbers.js"),
        working_dir
            .path()
            .join("scripts/highlightjs-line-numbers.js"),
    )
    .expect("Could not copy highlightjs-line-numbers.js!");
}

pub fn create(parsed_info: &ParseInfo) {
    let config = &parsed_info.config;
    let working_dir = &parsed_info.working_dir;
    futures::executor::block_on(async {
        let jinja_handle = process_jinja(config, working_dir.path());
        let scss_handle = convert_scss_to_css(config, working_dir.path());

        join!(jinja_handle, scss_handle);
    });
    std::fs::copy(
        config.root.join("robots.txt"),
        working_dir.path().join("robots.txt"),
    )
    .expect("Could not copy robots.txt");
    copy_dir_all(&config.static_dir, &working_dir.path().join("static"));
    copy_dir_all(
        &config.time_machine,
        &working_dir.path().join("time_machine"),
    );
    copy_highlight_related_code(parsed_info);
}
