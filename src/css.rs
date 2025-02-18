use crate::config::ConfigFile;
use crate::util::write_file;
use grass::Options;
use std::fs;
use std::path::Path;

pub async fn convert_scss_to_css(config: &ConfigFile, destination: &Path) {
    let scss_content =
        fs::read_to_string(config.scss_source.join("main.scss")).expect("Failed to read SCSS file");
    let options = Options::default().load_path(&config.scss_source);
    let css_output = grass::from_string(scss_content, &options).expect("Failed to compile SCSS");
    write_file(&destination.join("index.css"), css_output)
}
