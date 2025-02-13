use crate::util::write_file;
use grass::Options;
use std::fs;
use std::path::PathBuf;

pub fn convert_scss_to_css(source_folder: String, destination: PathBuf) {
    let scss_content =
        fs::read_to_string(source_folder.clone() + "/main.scss").expect("Failed to read SCSS file");
    let options = Options::default().load_path(source_folder);
    let css_output = grass::from_string(scss_content, &options).expect("Failed to compile SCSS");
    let mut css_file = destination.clone();
    css_file.push("index.css");
    write_file(css_file, css_output)
}
