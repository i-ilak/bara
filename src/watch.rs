use crate::build::build;
use crate::parse_info::ParseInfo;
use std::sync::mpsc::channel;

use notify::{
    Config, Event, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher,
};

pub fn watch(parsed_info: &ParseInfo) {
    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(
        move |result: NotifyResult<Event>| {
            if let Ok(event) = result {
                tx.send(event).unwrap();
            }
        },
        Config::default(),
    )
    .unwrap();

    watcher
        .watch(&parsed_info.config.root, RecursiveMode::Recursive)
        .unwrap();
    let output_dir = &parsed_info.config.root.join(&parsed_info.config.output);
    for event in rx {
        let should_process = event.paths.iter().any(|path| {
            if path.starts_with(output_dir) || path.ends_with("~") {
                return false;
            }

            let path_str = path.to_string_lossy();
            path.starts_with(&parsed_info.config.root)
                && (path_str.ends_with(".md")
                    || path_str.ends_with(".scss")
                    || path_str.ends_with(".jinja2"))
        });

        if should_process {
            build(parsed_info);
            println!("Rebuild triggered!");
        }
    }
}
