use serde::Deserialize;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(String),
    #[error("Failed to parse config file: {0}")]
    ParseError(String),
    #[error("Calendar not found: {0}")]
    CalendarNotFound(String),
}

#[derive(Debug, Deserialize, Clone)]
pub struct CalendarConfig {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub calendars: Vec<CalendarConfig>,
}

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content =
            fs::read_to_string(path).map_err(|e| ConfigError::ReadError(e.to_string()))?;

        let config: Config =
            toml::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))?;

        Ok(config)
    }

    /// Get a calendar by name
    pub fn get_calendar(&self, name: &str) -> Result<&CalendarConfig, ConfigError> {
        self.calendars
            .iter()
            .find(|c| c.name == name)
            .ok_or_else(|| ConfigError::CalendarNotFound(name.to_string()))
    }

    /// Get all calendar names
    #[allow(dead_code)]
    pub fn calendar_names(&self) -> Vec<String> {
        self.calendars.iter().map(|c| c.name.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_valid_config() {
        let toml_content = r#"
[[calendars]]
name = "work"
url = "https://example.com/work.ics"

[[calendars]]
name = "personal"
url = "https://example.com/personal.ics"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = Config::from_file(temp_file.path()).unwrap();

        assert_eq!(config.calendars.len(), 2);
        assert_eq!(config.calendars[0].name, "work");
        assert_eq!(config.calendars[1].name, "personal");
    }

    #[test]
    fn test_get_calendar_by_name() {
        let toml_content = r#"
[[calendars]]
name = "work"
url = "https://example.com/work.ics"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = Config::from_file(temp_file.path()).unwrap();
        let calendar = config.get_calendar("work").unwrap();

        assert_eq!(calendar.name, "work");
        assert_eq!(calendar.url, "https://example.com/work.ics");
    }

    #[test]
    fn test_get_nonexistent_calendar() {
        let toml_content = r#"
[[calendars]]
name = "work"
url = "https://example.com/work.ics"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = Config::from_file(temp_file.path()).unwrap();
        let result = config.get_calendar("nonexistent");

        assert!(result.is_err());
        assert!(matches!(result, Err(ConfigError::CalendarNotFound(_))));
    }

    #[test]
    fn test_calendar_names() {
        let toml_content = r#"
[[calendars]]
name = "work"
url = "https://example.com/work.ics"

[[calendars]]
name = "personal"
url = "https://example.com/personal.ics"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = Config::from_file(temp_file.path()).unwrap();
        let names = config.calendar_names();

        assert_eq!(names, vec!["work", "personal"]);
    }

    #[test]
    fn test_invalid_toml() {
        let toml_content = "invalid toml content {{";

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = Config::from_file(temp_file.path());

        assert!(result.is_err());
        assert!(matches!(result, Err(ConfigError::ParseError(_))));
    }
}
