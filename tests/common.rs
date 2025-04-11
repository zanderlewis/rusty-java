use std::fs;
use std::path::Path;
use std::process::Command;
use std::env;

// Helper function to run a command in a specific directory
pub fn run_command_in_dir(dir: &Path, program: &str, args: &[&str]) -> Result<(), String> {
    let output = Command::new(program)
        .current_dir(dir)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Command failed with: {}\nStderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

// Helper function to clean up build artifacts after a test
pub fn cleanup_build_dir(project_dir: &Path) {
    let build_dir = project_dir.join("rsj_build");
    if build_dir.exists() {
        let _ = fs::remove_dir_all(build_dir);
    }
}

// Prepare an example project for testing
#[allow(dead_code)] // This is to please the compiler, as this is actually used
pub fn prepare_example_project(example_name: &str) -> Result<std::path::PathBuf, String> {
    // Get the cargo manifest directory
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| "Failed to get CARGO_MANIFEST_DIR".to_string())?;
    
    // Create path to example project
    let example_path = Path::new(&manifest_dir).join("examples").join(example_name);
    
    if !example_path.exists() {
        return Err(format!("Example project '{}' does not exist", example_name));
    }
    
    // Clean up any previous build artifacts
    cleanup_build_dir(&example_path);
    
    Ok(example_path)
}

// Find the path to the rusty-java binary
fn find_binary_path() -> Result<std::path::PathBuf, String> {
    // First try the cargo env var approach
    if let Ok(path) = env::var("CARGO_BIN_EXE_rusty-java") {
        let binary_path = Path::new(&path);
        if binary_path.exists() {
            return Ok(binary_path.to_path_buf());
        }
    }

    // If that fails, try to find it in the target directory
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| "Failed to get CARGO_MANIFEST_DIR".to_string())?;
    
    // Try debug build
    let debug_path = Path::new(&manifest_dir).join("target/debug/rusty-java");
    if debug_path.exists() {
        return Ok(debug_path);
    }

    // Try release build
    let release_path = Path::new(&manifest_dir).join("target/release/rusty-java");
    if release_path.exists() {
        return Ok(release_path);
    }

    // If we can't find it, build it
    println!("Binary not found, attempting to build it...");
    let status = Command::new("cargo")
        .args(&["build"])
        .current_dir(&manifest_dir)
        .status()
        .map_err(|_| "Failed to build rusty-java binary".to_string())?;

    if !status.success() {
        return Err("Failed to build rusty-java binary".to_string());
    }

    let debug_path = Path::new(&manifest_dir).join("target/debug/rusty-java");
    if debug_path.exists() {
        return Ok(debug_path);
    }

    Err("Could not find or build rusty-java binary".to_string())
}

// Execute rsj command on an example project
pub fn run_rsj_command(example_path: &Path, command: &str) -> Result<(), String> {
    let bin_path = find_binary_path()?;
    
    run_command_in_dir(
        example_path,
        bin_path.to_str().unwrap(),
        &[command]
    )
}