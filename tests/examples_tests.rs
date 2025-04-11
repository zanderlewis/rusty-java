mod common;

use std::path::Path;
use std::fs;
use serial_test::serial;

// Test the hello-world example
#[test]
#[serial]
fn test_hello_world_example() {
    // Prepare and test the hello-world example
    test_example_project("hello-world").expect("Hello world example test failed");
}

// Test the namespaces example
#[test]
#[serial]
fn test_namespaces_example() {
    // Prepare and test the namespaces example
    test_example_project("namespaces").expect("Namespaces example test failed");
}

// Test the dependencies example
#[test]
#[serial]
fn test_dependencies_example() {
    // Prepare and test the dependencies example
    test_example_project("dependencies").expect("Dependencies example test failed");
}

// Helper function to test a specific example project
fn test_example_project(example_name: &str) -> Result<(), String> {
    println!("Testing example project: {}", example_name);
    
    // Get the example project path
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| "Failed to get CARGO_MANIFEST_DIR".to_string())?;
    
    let example_path = Path::new(&manifest_dir).join("examples").join(example_name);
    if !example_path.exists() {
        return Err(format!("Example project '{}' does not exist", example_name));
    }
    
    // Clean up any previous build artifacts
    common::cleanup_build_dir(&example_path);
    
    // Build the project
    println!("Building example {}", example_name);
    common::run_rsj_command(&example_path, "build")?;

    // Run Gradle on it to build the actual Java project
    println!("Running Gradle build for {}", example_name);
    let gradle_path = example_path.join("rsj_build").join("gradle");
    if !gradle_path.exists() {
        return Err(format!("Gradle build directory not found for example '{}'", example_name));
    }
    common::run_command_in_dir(&gradle_path, "gradle", &["shadowJar"])?;
    
    // Verify that the build output exists
    println!("Verifying build output for {}", example_name);
    verify_build_output(&example_path, example_name)?;
    
    // Clean up build artifacts
    common::cleanup_build_dir(&example_path);
    
    println!("Example project '{}' tested successfully", example_name);
    Ok(())
}

// Verify that the build output exists and is in the expected location
fn verify_build_output(example_path: &Path, example_name: &str) -> Result<(), String> {
    // Get the project name from the rsj.toml file
    let toml_content = fs::read_to_string(example_path.join("rsj.toml"))
        .map_err(|_| format!("Failed to read rsj.toml for example '{}'", example_name))?;
    
    // Parse the project name from the TOML content
    let project_name = parse_project_name(&toml_content)
        .ok_or_else(|| format!("Failed to parse project name from rsj.toml for example '{}'", example_name))?;
    
    let version = parse_project_version(&toml_content)
        .unwrap_or_else(|| "1.0.0".to_string());
    
    // Check the build directory structure
    let build_dir = example_path.join("rsj_build").join("gradle");
    
    if !build_dir.exists() {
        return Err(format!("Build directory not found for example '{}' at: {}", 
            example_name, build_dir.display()));
    }
    
    // List the contents of the libs directory to debug
    let libs_dir = build_dir.join("build").join("libs");
    if !libs_dir.exists() {
        return Err(format!("Libs directory not found for example '{}' at: {}", 
            example_name, libs_dir.display()));
    }
    
    // List all files in the libs directory
    println!("Files in libs directory:");
    if let Ok(entries) = fs::read_dir(&libs_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                println!("  {}", entry.path().display());
            }
        }
    }
    
    // Find any jar file in the libs directory
    let jar_files: Vec<_> = fs::read_dir(&libs_dir)
        .map_err(|_| format!("Failed to read libs directory for example '{}'", example_name))?
        .filter_map(Result::ok)
        .filter(|entry| {
            if let Some(ext) = entry.path().extension() {
                ext == "jar"
            } else {
                false
            }
        })
        .collect();
    
    if jar_files.is_empty() {
        // If no jar files were found, return an error
        let expected_jar = libs_dir.join(format!("{}-{}.jar", project_name, version));
        return Err(format!(
            "Build output JAR not found for example '{}'. Expected at: {}", 
            example_name, expected_jar.display()
        ));
    }
    
    // If we found jar files, consider the test successful
    println!("Found JAR file(s) for example '{}' at: {}", 
        example_name, jar_files[0].path().display());
    Ok(())
}

// Parse the project name from the TOML content
fn parse_project_name(toml_content: &str) -> Option<String> {
    let lines: Vec<&str> = toml_content.split('\n').collect();
    for line in lines {
        let line = line.trim();
        if line.starts_with("name") {
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() >= 2 {
                let name = parts[1].trim().trim_matches('"').trim_matches('\'');
                return Some(name.to_string());
            }
        }
    }
    None
}

// Parse the project version from the TOML content
fn parse_project_version(toml_content: &str) -> Option<String> {
    let lines: Vec<&str> = toml_content.split('\n').collect();
    for line in lines {
        let line = line.trim();
        if line.starts_with("version") {
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() >= 2 {
                let version = parts[1].trim().trim_matches('"').trim_matches('\'');
                return Some(version.to_string());
            }
        }
    }
    None
}