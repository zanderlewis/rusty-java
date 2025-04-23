use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
pub struct Config {
    pub project: Project,
    pub dependencies: Option<HashMap<String, String>>,
}

#[derive(Deserialize)]
pub struct Project {
    pub name: String,
    pub version: String,
    pub main_class: String,
    pub base_namespace: String,         // Base namespace for the project
    pub root_path: Option<String>,      // Path to the project root
    pub gradle_version: Option<String>, // Optional Gradle distribution version
    pub shadow_plugin_version: Option<String>, // Optional Shadow plugin version
    pub use_shadow: Option<bool>,       // Whether to apply the ShadowJar plugin
}

pub fn load_config() -> Result<Config, String> {
    let config_path = "rsj.toml";

    if !Path::new(config_path).exists() {
        return Err(format!(
            "Error: Missing `{}` file. Run 'rsj init' to create a new project.",
            config_path
        ));
    }

    // Load config file content
    let config_content = fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read `{}`: {}", config_path, e))?;

    // Parse TOML content into Config struct
    let config: Config = toml::from_str(&config_content)
        .map_err(|e| format!("Invalid TOML format in `{}`: {}", config_path, e))?;

    // Basic validation
    if config.project.name.trim().is_empty() {
        return Err("Project name cannot be empty in rsj.toml".to_string());
    }

    if config.project.main_class.trim().is_empty() {
        return Err("Main class name cannot be empty in rsj.toml".to_string());
    }

    Ok(config)
}
