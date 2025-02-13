use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn write_file(path: PathBuf, content: String) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("Cloud not create directory to create file!");
    }
    fs::write(path, content).expect("Could not write content to file!");
}

fn replace_in_file(file_path: &str, from: &str, to: &str) -> io::Result<()> {
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
