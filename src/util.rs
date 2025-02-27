use crate::parse_info::ParseInfo;
use regex::Regex;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use walkdir::WalkDir;

pub fn write_file(path: &Path, content: String) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("Could not create directory to create file!");
    }
    fs::write(path, content).expect("Could not write content to file!");
}

fn replace_in_file(file_path: &Path, from: &str, to: &str) {
    let contents = fs::read_to_string(file_path).expect("Unable to read in file!");
    let new_contents = contents.replace(from, to);
    let mut file = fs::File::create(file_path).expect("Could not create file!");
    file.write_all(new_contents.as_bytes())
        .expect("Could not write data into file!");
}

pub fn copy_dir_all(src: &Path, dst: &Path) {
    if !dst.exists() {
        fs::create_dir_all(dst).expect(&format!("Could not create folder:\t {:?}", dst));
    }

    for entry in fs::read_dir(src).expect("Could not read source director!") {
        let entry = entry.expect("Problem parsing file!");
        let entry_path = entry.path();
        let new_path = dst.join(entry.file_name());

        if entry_path.is_dir() {
            copy_dir_all(&entry_path, &new_path);
        } else {
            fs::copy(&entry_path, &new_path).expect("Could not copy file!");
        }
    }
}

fn patch_basepath(working_dir: &Path) {
    let re = Regex::new(r"time_machine/v\d+\.\d+\.\d+").expect("Invalid regex");

    for entry in WalkDir::new(working_dir).into_iter().filter_map(|e| e.ok()) {
        let current_path = entry.path();

        if current_path.is_file() {
            if let Some(filename) = current_path.file_name().and_then(|name| name.to_str()) {
                if filename.ends_with(".html") || filename.ends_with(".json") {
                    let parent_dir_name = current_path
                        .strip_prefix(working_dir)
                        .expect("Failed to compute relative path");

                    let parent_dir_str = parent_dir_name.to_string_lossy();

                    if !parent_dir_str.contains("time_machine") {
                        replace_in_file(current_path, "BASEPATH", "");
                    } else if let Some(captures) = re.captures(&parent_dir_str) {
                        let match_str = captures.get(0).unwrap().as_str();
                        replace_in_file(current_path, "BASEPATH", &format!("/{}", match_str));
                    }
                }
            }
        }
    }
}

pub fn finalize(parsed_info: &ParseInfo) {
    let config = &parsed_info.config;
    let working_dir = &parsed_info.working_dir;

    patch_basepath(working_dir.path());
    let output_dir = PathBuf::from(&config.output);
    copy_dir_all(working_dir.path(), &output_dir);
}
