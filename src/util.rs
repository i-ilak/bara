use regex::Regex;
use std::fs;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn write_file(path: &Path, content: String) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("Cloud not create directory to create file!");
    }
    fs::write(path, content).expect("Could not write content to file!");
}

fn replace_in_file(file_path: &Path, from: &str, to: &str) -> io::Result<()> {
    let contents = fs::read_to_string(file_path)?;
    let new_contents = contents.replace(from, to);
    let mut file = fs::File::create(file_path)?;
    file.write_all(new_contents.as_bytes())?;

    Ok(())
}

pub fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_path = entry.path();
        let new_path = dst.join(entry.file_name());

        if entry_path.is_dir() {
            copy_dir_all(&entry_path, &new_path)?;
        } else {
            fs::copy(&entry_path, &new_path)?;
        }
    }

    Ok(())
}

fn relative_to(path: &Path, base: &Path) -> Option<PathBuf> {
    // Canonicalize both paths to ensure they are absolute and normalized
    let abs_path = path.canonicalize().ok()?;
    let abs_base = base.canonicalize().ok()?;

    // Iterate through the components of the base path
    let mut base_components = abs_base.components();
    let mut path_components = abs_path.components();

    // Find the common prefix between the two paths
    loop {
        match (base_components.next(), path_components.next()) {
            (Some(base_part), Some(path_part)) if base_part == path_part => continue,
            (None, Some(_)) => {
                // The base path is a prefix of the target path
                let mut relative_path = PathBuf::new();
                for _ in 0..base_components.count() {
                    relative_path.push("..");
                }
                for part in path_components {
                    relative_path.push(part);
                }
                return Some(relative_path);
            }
            _ => {
                // The paths do not share a common prefix
                return None;
            }
        }
    }
}

pub fn patch_basepath(working_dir: &Path) {
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
