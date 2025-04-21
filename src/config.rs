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
        return Err("Error: Missing `rsj.toml` file.".to_string());
    }

    let config_content =
        fs::read_to_string(config_path).map_err(|_| "Failed to read `rsj.toml`.".to_string())?;
    let config: Config =
        toml::from_str(&config_content).map_err(|_| "Invalid TOML format.".to_string())?;

    Ok(config)
}
