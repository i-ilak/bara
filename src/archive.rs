use semver::Version;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const TIME_MACHINE_DIR: &str = "time_machine";

fn get_all_versions() -> Result<Vec<Version>, Box<dyn std::error::Error>> {
    let mut versions = Vec::new();

    // Read the contents of the TIME_MACHINE_DIR
    for entry in fs::read_dir(TIME_MACHINE_DIR)? {
        let entry = entry?;
        let path = entry.path();

        // Check if the entry is a directory
        if path.is_dir() {
            if let Some(dir_name) = path.file_name() {
                if let Some(dir_name_str) = dir_name.to_str() {
                    // Try to parse the directory name as a semantic version
                    match Version::parse(dir_name_str) {
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

fn copy_files_to_archive(working_dir: &Path, version: &str) -> std::io::Result<()> {
    // Create the destination directory
    let dest_dir = PathBuf::from(TIME_MACHINE_DIR).join(version);
    fs::create_dir_all(&dest_dir)?;

    // Folders to copy
    let folders = ["posts", "projects"];

    for folder in folders.iter() {
        let source_folder = working_dir.join(folder);
        let dest_folder = dest_dir.join(folder);

        // Walk through the source folder
        for entry in WalkDir::new(&source_folder) {
            let entry = entry?;
            let path = entry.path();

            // Calculate the relative path
            let rel_path = path.strip_prefix(&source_folder).unwrap();
            let target_path = dest_folder.join(rel_path);

            if path.is_dir() {
                // Create the target directory if it doesn't exist
                fs::create_dir_all(&target_path)?;
            } else if path.is_file() {
                // Copy the file
                fs::copy(path, &target_path)?;
            }
        }
    }

    // Copy individual files
    let files = ["index.html", "index.css", "rss.xml", "privacy_policy.html"];
    for file in files.iter() {
        let source_file = working_dir.join(file);
        let dest_file = dest_dir.join(file);
        fs::copy(&source_file, &dest_file)?;
    }

    Ok(())
}

pub fn archive(working_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Get all existing versions
    let versions = get_all_versions()?;

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
    copy_files_to_archive(working_dir, &version)?;

    Ok(())
}
