use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::load_config;
use crate::gradle::setup_gradle_project;
use crate::utils::{GRADLE_PATH, OUTPUT_PATH, printinfo, separator};

// Helper to create a file with error mapping
fn create_file(path: &Path, content: &str) -> Result<(), String> {
    fs::write(path, content).map_err(|e| format!("Failed to write to `{}`: {}", path.display(), e))
}

// Helper to create a directory with error mapping
fn create_directory(path: &Path) -> Result<(), String> {
    fs::create_dir(path).map_err(|e| format!("Failed to create `{}`: {}", path.display(), e))
}

pub fn init_project() -> Result<(), String> {
    // Check if project files already exist
    let config_path = Path::new("rsj.toml");
    let src_dir = Path::new("src");

    if config_path.exists() {
        return Err("Error: `rsj.toml` already exists.".to_string());
    }

    if src_dir.exists() {
        return Err("Error: `src` directory already exists.".to_string());
    }

    // Create config file
    let config_content = r#"[project]
name = "my_project"
version = "0.1.0"
main_class = "Main"
base_namespace = "com.example"

# [dependencies]
# junit = "org.junit.jupiter:junit-jupiter:5.9.1"
"#;
    create_file(config_path, config_content)?;

    // Create src directory
    create_directory(src_dir)?;

    // Create sample Java files
    create_java_sample_files(src_dir)?;

    printinfo("Initialized a new RSJ project with Gradle.");
    Ok(())
}

fn create_java_sample_files(src_dir: &Path) -> Result<(), String> {
    // Main.java
    let main_content = r#"package com.example;

import com.example.classone.ClassOne;
import com.example.classtwo.ClassTwo;

public class Main {
    public static void main(String[] args) {
        ClassOne.oneMethod();
        ClassTwo.twoMethod();
    }
}"#;
    create_file(&src_dir.join("Main.java"), main_content)?;

    // ClassOne.java
    let classone_dir = src_dir.join("classone");
    create_directory(&classone_dir)?;
    let classone_content = r#"package com.example.classone;

public class ClassOne {
    public static void oneMethod() {
        System.out.println("ClassOne method");
    }
}"#;
    create_file(&classone_dir.join("ClassOne.java"), classone_content)?;

    // ClassTwo.java
    let classtwo_dir = src_dir.join("classtwo");
    create_directory(&classtwo_dir)?;
    let classtwo_content = r#"package com.example.classtwo;

public class ClassTwo {
    public static void twoMethod() {
        System.out.println("ClassTwo method");
    }
}"#;
    create_file(&classtwo_dir.join("ClassTwo.java"), classtwo_content)?;

    Ok(())
}

pub fn build_project() -> Result<(), String> {
    let config = load_config()?;

    // Verify src directory exists
    let src_dir = Path::new(config.project.root_path.as_deref().unwrap_or(".")).join("src");
    if !src_dir.exists() {
        return Err("Error: `src` directory is missing.".to_string());
    }

    // Create and prepare build directory
    let temp_path = prepare_build_directory()?;

    printinfo(&format!(
        "Using temporary build directory: {}",
        temp_path.display()
    ));

    separator();

    // Setup Gradle project
    setup_gradle_project(&config, src_dir.to_str().unwrap(), &temp_path)?;

    // Run Gradle build
    run_gradle_build(&config, &temp_path)?;

    separator();

    printinfo("Build succeeded! Output is in the temporary directory.");
    Ok(())
}

fn prepare_build_directory() -> Result<PathBuf, String> {
    let temp_path = Path::new(OUTPUT_PATH).to_path_buf();

    // Create build directory
    fs::create_dir_all(&temp_path)
        .map_err(|_| "Failed to create temporary build directory.".to_string())?;

    // Create .gitignore file
    let mut gitignore_file = File::create(temp_path.join(".gitignore"))
        .map_err(|_| "Failed to create `.gitignore`.".to_string())?;

    writeln!(gitignore_file, "*\n").map_err(|_| "Failed to write to `.gitignore`.".to_string())?;

    Ok(temp_path)
}

fn run_gradle_build(config: &crate::config::Config, temp_path: &Path) -> Result<(), String> {
    // Define the Gradle project directory
    let gradle_project_dir = temp_path.join(GRADLE_PATH);

    // Determine build parameters
    let use_shadow = config.project.use_shadow.unwrap_or(true);
    let task = if use_shadow { "shadowJar" } else { "build" };

    // Check for Gradle wrapper
    let gradlew_path = gradle_project_dir.join("gradlew");
    let wrapper_jar_path = gradle_project_dir
        .join("gradle")
        .join("wrapper")
        .join("gradle-wrapper.jar");

    // Select the appropriate Gradle command
    let (program, args) = if gradlew_path.exists() && wrapper_jar_path.exists() {
        ("./gradlew", vec![task])
    } else {
        ("gradle", vec![task])
    };

    // Run the build
    let build_status = Command::new(program)
        .args(&args)
        .current_dir(&gradle_project_dir)
        .status()
        .map_err(|_| "Failed to run Gradle build.".to_string())?;

    if !build_status.success() {
        return Err("Build failed.".to_string());
    }

    Ok(())
}

pub fn clean_build() -> Result<(), String> {
    let output_path = Path::new(OUTPUT_PATH);

    if output_path.exists() {
        fs::remove_dir_all(output_path)
            .map_err(|e| format!("Failed to clean the build output: {}", e))?;
        printinfo("Build output cleaned.");
    } else {
        printinfo("Build output not found.");
    }

    Ok(())
}
