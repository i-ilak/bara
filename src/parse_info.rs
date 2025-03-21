use crate::cli::Cli;
use crate::config::ConfigFile;
use crate::errors::{BaraError, ConfigError};
use clap::Parser;
use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Holds the parsed command-line arguments, configuration and working directory.
pub struct ParseInfo {
    pub args: Cli,
    pub config: ConfigFile,
    pub working_dir: TempDir,
}

/// Parses the YAML configuration content.
fn parse_yaml(content: &str) -> Result<ConfigFile, BaraError> {
    match serde_yaml::from_str::<ConfigFile>(content) {
        Ok(config) => Ok(config),
        Err(err) => {
            if let Some(location) = err.location() {
                let lines: Vec<&str> = content.lines().collect();
                let line_idx = location.line().saturating_sub(2);
                let line_content = lines.get(line_idx).unwrap_or(&"");

                Err(BaraError::ConfigError(ConfigError::InvalidValue {
                    field: format!("line {}", location.line() - 1),
                    value: line_content.trim().to_string(),
                }))
            } else {
                Err(BaraError::ConfigError(ConfigError::DeserializationError(
                    err.to_string(),
                )))
            }
        }
    }
}

/// Reads the configuration file content from the given path or a default file.
fn get_config_file_content(config_path: Option<String>) -> Result<String, BaraError> {
    let config_file_path = match config_path {
        Some(value) => PathBuf::from(value),
        None => env::current_dir()
            .expect("Could not get current directory!")
            .join("bara.yml"),
    };
    fs::read_to_string(config_file_path)
        .map_err(|er| BaraError::ConfigError(ConfigError::FileNotFoundOrPermission(er.to_string())))
}

/// Parses the configuration file by reading its content and converting it from YAML.
fn parse_config(config_path: Option<String>) -> Result<ConfigFile, BaraError> {
    let yaml_content = get_config_file_content(config_path)?;
    parse_yaml(&yaml_content)
}

/// Helper that builds the `ParseInfo` struct from a given Cli instance.
fn build_parse_info_with_cli(cli: Cli) -> Result<ParseInfo, BaraError> {
    let config = parse_config(cli.config.clone())?;
    let working_dir = TempDir::new_in(config.clone().root).map_err(|e| {
        BaraError::ConfigError(ConfigError::FileNotFoundOrPermission(e.to_string()))
    })?;
    Ok(ParseInfo {
        args: cli,
        config,
        working_dir,
    })
}

/// Builds the `ParseInfo` struct by parsing command-line arguments,
/// configuration, and creating a working directory.
///
/// In production, this function calls `Cli::parse()`, but tests can use `build_parse_info_with_cli`.
fn build_parse_info() -> Result<ParseInfo, BaraError> {
    let cli = Cli::parse();
    build_parse_info_with_cli(cli)
}

/// The public entry point that wraps `build_parse_info()` and handles errors by exiting the process.
/// Isolated form the core logic to enable testing.
pub fn parse() -> ParseInfo {
    match build_parse_info() {
        Ok(info) => info,
        Err(err) => {
            eprintln!("Unrecoverable Error: {}", err);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::{BaraError, ConfigError};
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn valid_yaml() -> &'static str {
        r#"
output: "output"
content: "content"
templates: "templates"
projects: "projects"
scss_source: "scss"
static_dir: "static"
time_machine: "time_machine"
"#
        .trim()
    }

    // This YAML is intentionally malformed.
    fn invalid_yaml() -> &'static str {
        r#"
output: "output"
content "content"   # missing colon here
templates: "templates"
projects: "projects"
scss_source: "scss"
static_dir: "static"
time_machine: "time_machine"
"#
        .trim()
    }

    #[test]
    fn test_parse_yaml_valid() {
        // Capture the expected current directory before parsing.
        let expected_dir = env::current_dir().expect("Could not get current dir");

        let result = parse_yaml(valid_yaml());
        assert!(result.is_ok());
        let config = result.unwrap();

        // Compare against the expected directory captured earlier.
        assert_eq!(config.root, expected_dir);
        assert_eq!(config.output, expected_dir.join("output"));
        assert_eq!(config.content, expected_dir.join("content"));
        assert_eq!(config.templates, expected_dir.join("templates"));
        assert_eq!(config.projects, expected_dir.join("projects"));
        assert_eq!(config.scss_source, expected_dir.join("scss"));
        assert_eq!(config.static_dir, expected_dir.join("static"));
        assert_eq!(config.time_machine, expected_dir.join("time_machine"));
    }

    #[test]
    fn test_parse_yaml_invalid() {
        let result = parse_yaml(invalid_yaml());
        assert!(result.is_err());
        match result {
            Err(BaraError::ConfigError(ConfigError::DeserializationError(msg))) => {
                assert!(!msg.is_empty());
            }
            Err(BaraError::ConfigError(ConfigError::InvalidValue { field, value })) => {
                assert!(!field.is_empty());
                assert!(!value.is_empty());
            }
            _ => panic!("Expected a YAML deserialization error variant"),
        }
    }

    #[test]
    fn test_get_config_file_content_with_valid_file() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let file_path = temp_dir.path().join("test_config.yml");
        let content = valid_yaml();
        fs::write(&file_path, content).expect("failed to write file");

        let result = get_config_file_content(Some(file_path.to_string_lossy().to_string()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), content);
    }

    #[test]
    fn test_get_config_file_content_with_missing_file() {
        let missing_path = PathBuf::from("non_existent_config.yml");
        let result = get_config_file_content(Some(missing_path.to_string_lossy().to_string()));
        assert!(result.is_err());
        if let Err(BaraError::ConfigError(ConfigError::FileNotFoundOrPermission(msg))) = result {
            assert!(!msg.is_empty());
        } else {
            panic!("Expected FileNotFoundOrPermission error");
        }
    }

    #[test]
    fn test_parse_config_valid() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let file_path = temp_dir.path().join("test_config.yml");
        let content = valid_yaml();
        fs::write(&file_path, content).expect("failed to write file");

        // Capture the expected directory from current env.
        let expected_dir = env::current_dir().expect("Could not get current dir");

        let result = parse_config(Some(file_path.to_string_lossy().to_string()));
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.root, expected_dir);
        assert_eq!(config.output, expected_dir.join("output"));
        assert_eq!(config.content, expected_dir.join("content"));
        assert_eq!(config.templates, expected_dir.join("templates"));
        assert_eq!(config.projects, expected_dir.join("projects"));
        assert_eq!(config.scss_source, expected_dir.join("scss"));
        assert_eq!(config.static_dir, expected_dir.join("static"));
        assert_eq!(config.time_machine, expected_dir.join("time_machine"));
    }

    #[test]
    fn test_parse_config_invalid_yaml() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let file_path = temp_dir.path().join("test_config.yml");
        let content = invalid_yaml();
        fs::write(&file_path, content).expect("failed to write file");

        let result = parse_config(Some(file_path.to_string_lossy().to_string()));
        assert!(result.is_err());
    }
}
