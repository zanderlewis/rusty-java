mod common;

use serial_test::serial;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

// Test the Gradle wrapper functionality
#[test]
#[serial]
fn test_gradle_wrapper() {
    let test_dir = setup_test_project("gradle_wrapper_test").unwrap();

    // Build the project (which will create a Gradle wrapper)
    println!("Building gradle_wrapper_test example");
    common::run_rsj_command(&test_dir, "build").unwrap();

    // Verify wrapper files exist
    let wrapper_script = test_dir.join("rsj_build").join("gradle").join("gradlew");
    let wrapper_bat = test_dir
        .join("rsj_build")
        .join("gradle")
        .join("gradlew.bat");
    let wrapper_jar_dir = test_dir
        .join("rsj_build")
        .join("gradle")
        .join("gradle")
        .join("wrapper");
    let wrapper_properties = wrapper_jar_dir.join("gradle-wrapper.properties");

    assert!(wrapper_script.exists(), "Gradle wrapper script not found");
    assert!(wrapper_bat.exists(), "Gradle wrapper batch file not found");
    assert!(
        wrapper_properties.exists(),
        "Gradle wrapper properties not found"
    );

    // Cleanup
    common::cleanup_build_dir(&test_dir);
    let _ = fs::remove_dir_all(&test_dir);
}

// Test the Gradle performance options
#[test]
#[serial]
fn test_gradle_performance_options() {
    let test_dir = setup_test_project("gradle_perf_test").unwrap();

    // Build the project
    println!("Building gradle_perf_test example");
    common::run_rsj_command(&test_dir, "build").unwrap();

    // Verify gradle.properties file has performance settings
    let gradle_properties = test_dir
        .join("rsj_build")
        .join("gradle")
        .join("gradle.properties");

    assert!(gradle_properties.exists(), "gradle.properties not found");

    let content = fs::read_to_string(&gradle_properties).unwrap();
    assert!(
        content.contains("org.gradle.parallel=true"),
        "Parallel execution not enabled"
    );
    assert!(
        content.contains("org.gradle.caching=true"),
        "Build cache not enabled"
    );
    assert!(
        content.contains("org.gradle.vfs.watch=true"),
        "File system watching not enabled"
    );

    // Cleanup
    common::cleanup_build_dir(&test_dir);
    let _ = fs::remove_dir_all(&test_dir);
}

// Setup a test project with given name
fn setup_test_project(project_name: &str) -> Result<std::path::PathBuf, String> {
    // Get the cargo manifest directory
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| "Failed to get CARGO_MANIFEST_DIR".to_string())?;

    let test_dir = Path::new(&manifest_dir)
        .join("target")
        .join("test_projects")
        .join(project_name);

    // Create test directory
    fs::create_dir_all(&test_dir).map_err(|e| format!("Failed to create test directory: {}", e))?;

    // Create src directory
    let src_dir = test_dir.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| format!("Failed to create src directory: {}", e))?;

    // Create a simple Main.java file
    let main_java = src_dir.join("Main.java");
    let mut main_file =
        File::create(&main_java).map_err(|e| format!("Failed to create Main.java: {}", e))?;

    writeln!(
        main_file,
        r#"
        public class Main {{
    public static void main(String[] args) {{
        System.out.println("Hello from {}, built with enhanced Gradle!");
    }}
}}"#,
        project_name
    )
    .map_err(|e| format!("Failed to write to Main.java: {}", e))?;

    // Create rsj.toml
    let rsj_toml = test_dir.join("rsj.toml");
    let mut toml_file =
        File::create(&rsj_toml).map_err(|e| format!("Failed to create rsj.toml: {}", e))?;

    writeln!(
        toml_file,
        r#"
        [project]
name = "{}"
version = "1.0.0"
main_class = "Main"
build_tool = "gradle"
base_namespace = "com.example"
"#,
        project_name
    )
    .map_err(|e| format!("Failed to write to rsj.toml: {}", e))?;

    Ok(test_dir)
}
