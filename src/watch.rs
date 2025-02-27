use crate::build::build;
use crate::parse_info::ParseInfo;
use std::sync::mpsc::channel;

use notify::{
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher,
};

fn handle_file_event(parsed_info: &ParseInfo, event: &Event) {
    match event.kind {
        EventKind::Create(_) => println!("Created:\t{:?}", event.paths),
        EventKind::Modify(_) => println!("Modified:\t{:?}", event.paths),
        EventKind::Remove(_) => println!("Removed:\t{:?}", event.paths),
        EventKind::Access(_) => println!("Accessed:\t{:?}", event.paths),
        EventKind::Other => println!("Other event:\t{:?}", event.paths),
        _ => println!("Other event type: {:?}", event),
    }
    build(parsed_info);
}

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
            handle_file_event(&parsed_info, &event);
        }
    }
}
