use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigFile {
    pub root: PathBuf,
    pub output: PathBuf,
    pub content: PathBuf,
    pub templates: PathBuf,
    pub projects: PathBuf,
    pub scss_source: PathBuf,
    pub static_dir: PathBuf,
    pub time_machine: PathBuf,
}
