use crate::config::ConfigFile;
use crate::util::write_file;
use grass::Options;
use std::fs;
use std::path::Path;

pub fn convert_scss_to_css(config: &ConfigFile, destination: &Path) {
    let scss_content = fs::read_to_string(config.scss_source.clone() + "/main.scss")
        .expect("Failed to read SCSS file");
    let options = Options::default().load_path(config.scss_source.clone());
    let css_output = grass::from_string(scss_content, &options).expect("Failed to compile SCSS");
    let mut css_file = destination.to_path_buf();
    css_file.push("index.css");
    write_file(&css_file, css_output)
}
