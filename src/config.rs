use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct ConfigFile {
    pub root: String,
    pub output: String,
    pub content: String,
    pub templates: String,
    pub projects: String,
    pub scss_source: String,
    pub scripts: String,
    pub static_dir: String,
}
