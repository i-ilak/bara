use serde::{Deserialize, Deserializer};
use std::env;
use std::path::PathBuf;

#[derive(Clone, Debug)]
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

impl<'de> Deserialize<'de> for ConfigFile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let root = env::current_dir().expect("Cloud not determine executable dir.");

        #[derive(Deserialize)]
        struct RawConfig {
            output: PathBuf,
            content: PathBuf,
            templates: PathBuf,
            projects: PathBuf,
            scss_source: PathBuf,
            static_dir: PathBuf,
            time_machine: PathBuf,
        }

        let raw = RawConfig::deserialize(deserializer)?;

        Ok(ConfigFile {
            root: root.clone(),
            output: root.join(raw.output),
            content: root.join(raw.content),
            templates: root.join(raw.templates),
            projects: root.join(raw.projects),
            scss_source: root.join(raw.scss_source),
            static_dir: root.join(raw.static_dir),
            time_machine: root.join(raw.time_machine),
        })
    }
}
