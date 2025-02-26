use crate::config::ConfigFile;
use crate::util::copy_dir_all;
use crate::parse_info::ParseInfo;

use semver::Version;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

fn get_all_versions(config: &ConfigFile) -> Result<Vec<Version>, Box<dyn std::error::Error>> {
    let mut versions = Vec::new();

    // Read the contents of the TIME_MACHINE_DIR
    for entry in fs::read_dir(config.time_machine.clone())? {
        let entry = entry?;
        let path = entry.path();

        // Check if the entry is a directory
        if path.is_dir() {
            if let Some(dir_name) = path.file_name() {
                if let Some(dir_name_str) = dir_name.to_str() {
                    // Try to parse the directory name as a semantic version
                    match Version::parse(&dir_name_str[1..]) {
                        Ok(version) => versions.push(version),
                        Err(_) => {
                            // Skip directories with invalid version names
                            eprintln!("Skipping invalid version: {}", dir_name_str);
                        }
                    }
                }
            }
        }
    }

    // Sort the versions
    versions.sort();

    Ok(versions)
}

fn copy_files_to_archive(
    config: &ConfigFile,
    working_dir: &Path,
    version: &str,
) -> std::io::Result<()> {
    // Create the destination directory
    let dest_dir = config.time_machine.join(version);
    fs::create_dir_all(&dest_dir).expect("Could not create new version folder!");

    // Folders to copy
    let folders = ["posts", "projects"];

    for folder in folders.iter() {
        let source_folder = working_dir.join(folder);
        let dest_folder = dest_dir.join(folder);

        for entry in WalkDir::new(&source_folder) {
            let entry = entry?;
            if entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();

            copy_dir_all(path, &dest_folder);
        }
    }

    copy_dir_all(
        &config.root.join("_serve/scripts"),
        &dest_dir.join("scripts"),
    );

    // Copy individual files
    let files = ["index.html", "index.css", "rss.xml", "privacy_policy.html"];
    for file in files.iter() {
        let source_file = working_dir.join(file);
        let dest_file = dest_dir.join(file);
        match fs::copy(&source_file, &dest_file) {
            Ok(_) => {}
            Err(_) => {
                println!("{}", format!("File does not exist:\t{:?}", file))
            }
        };
    }

    Ok(())
}

pub fn archive(parsed_info: &ParseInfo) -> Result<(), Box<dyn std::error::Error>> {
    let config = &parsed_info.config;
    let working_dir= &parsed_info.working_dir.path();

    // Get all existing versions
    let versions = get_all_versions(config).expect("Could not extract all the versions!");

    // Check if there are any existing versions
    if versions.is_empty() {
        return Err("No existing versions found".into());
    }

    // Get the latest version
    let latest_version = versions.last().unwrap();

    // Increment the patch version (micro in Python)
    let new_version = Version {
        major: latest_version.major,
        minor: latest_version.minor,
        patch: latest_version.patch + 1,
        pre: latest_version.pre.clone(),
        build: latest_version.build.clone(),
    };

    // Format the new version as a string with a "v" prefix
    let version = format!("v{}", new_version);

    // Copy files to the archive with the new version
    copy_files_to_archive(config, working_dir, &version)?;

    Ok(())
}
