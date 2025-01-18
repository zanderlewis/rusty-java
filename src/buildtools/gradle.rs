use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::config::Config;
use crate::utils::{copy_src_files, GRADLE_PATH};

pub fn setup_gradle_project(
    config: &Config,
    src_dir: &str,
    temp_path: &Path,
) -> Result<(), String> {
    let gradle_dir = temp_path.join(GRADLE_PATH);
    let build_file_path = gradle_dir.join("build.gradle");
    let settings_file_path = gradle_dir.join("settings.gradle");

    // Create Gradle project structure
    fs::create_dir_all(gradle_dir.join("src/main/java"))
        .map_err(|_| "Failed to create Gradle project structure.".to_string())?;

    // Copy source files with base_namespace
    copy_src_files(
        src_dir,
        &gradle_dir.join("src/main/java".to_owned() + "/" + &config.project.base_namespace.replace(".", "/")),
        &config.project.base_namespace,
    )?;

    // Create `settings.gradle` and set the project name
    let mut settings_file = File::create(&settings_file_path)
        .map_err(|_| "Failed to create `settings.gradle`.".to_string())?;
    writeln!(
        settings_file,
        "rootProject.name = '{}'",
        config.project.name
    )
    .map_err(|_| "Failed to write to `settings.gradle`.".to_string())?;

    // Create `build.gradle`
    let mut build_file = File::create(&build_file_path)
        .map_err(|_| "Failed to create `build.gradle`.".to_string())?;
    writeln!(
        build_file,
        r#"plugins {{
    id 'java'
    id 'com.github.johnrengelman.shadow' version '7.1.2'
}}

group = '{}'
version = '{}'

repositories {{
    mavenCentral()
}}

dependencies {{
{}
}}

tasks.jar {{
    manifest {{
        attributes(
            'Main-Class': '{}'
        )
    }}
}}"#,
        config.project.name,
        config.project.version,
        generate_gradle_dependencies(&config.dependencies),
        config.project.base_namespace.to_owned() + "." + &config.project.main_class
    )
    .map_err(|_| "Failed to write to `build.gradle`.".to_string())?;

    Ok(())
}

fn generate_gradle_dependencies(
    dependencies: &Option<std::collections::HashMap<String, String>>,
) -> String {
    dependencies
        .as_ref()
        .map(|deps| {
            deps.iter()
                .map(|(_, dep)| format!("    implementation '{}'", dep))
                .collect::<Vec<String>>()
                .join("\n")
        })
        .unwrap_or_default()
}
