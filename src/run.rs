use std::path::Path;
use std::process::Command;

use crate::build::build_project;
use crate::config::load_config;
use crate::utils::{basic_seperator, printinfo, GRADLE_PATH, MAVEN_PATH, OUTPUT_PATH};

pub fn run_project() -> Result<(), String> {
    build_project()?;

    // Assuming build output is in temp_path/build/libs or temp_path/target
    let config = load_config().map_err(|e| e)?;

    let temp_path = Path::new(OUTPUT_PATH).to_path_buf();

    let jar_path = if config.project.build_tool.to_lowercase() == "gradle" {
        temp_path
            .join(GRADLE_PATH)
            .join("build")
            .join("libs")
            .join(format!(
                "{}-{}.jar",
                config.project.name, config.project.version
            ))
    } else {
        temp_path.join(MAVEN_PATH).join("target").join(format!(
            "{}-{}.jar",
            config.project.name, config.project.version
        ))
    };

    if !jar_path.exists() {
        return Err("Build output JAR not found.".to_string());
    }

    printinfo(&format!("Running {}", jar_path.display()));

    basic_seperator();

    let status = Command::new("java")
        .arg("-jar")
        .arg(&jar_path)
        .status()
        .map_err(|_| "Failed to run the Java application.".to_string())?;

    if status.success() {
        Ok(())
    } else {
        Err("Java application exited with an error.".to_string())
    }
}
