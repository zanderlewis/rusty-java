use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;

use crate::buildtools::{gradle::setup_gradle_project, maven::setup_maven_project};
use crate::config::load_config;
use crate::utils::{
    gradle_maven_seperator, printerr, printinfo, rsj_seperator, GRADLE_PATH, MAVEN_PATH,
    OUTPUT_PATH,
};

pub fn init_project() -> Result<(), String> {
    let config_path = Path::new("rsj.toml");
    if config_path.exists() {
        return Err("Error: `rsj.toml` already exists.".to_string());
    }

    let src_dir = Path::new("src");
    if src_dir.exists() {
        return Err("Error: `src` directory already exists.".to_string());
    }

    // Create `rsj.toml`
    let mut config_file =
        File::create(config_path).map_err(|_| "Failed to create `rsj.toml`.".to_string())?;
    writeln!(
        config_file,
        r#"[project]
name = "my_project"
version = "0.1.0"
main_class = "Main"
build_tool = "gradle" # or "maven"
base_namespace = "com.example"

# [dependencies]
# junit = "junit:junit:4.13.2"
"#
    )
    .map_err(|_| "Failed to write to `rsj.toml`.".to_string())?;

    // Create `src` directory
    fs::create_dir(src_dir).map_err(|_| "Failed to create `src` directory.".to_string())?;

    // Create a sample Main.java file
    let main_class_path = src_dir.join("Main.java");
    let mut main_class_file =
        File::create(main_class_path).map_err(|_| "Failed to create `Main.java`.".to_string())?;
    writeln!(
        main_class_file,
        r#"public class Main {{

    public static void main(String[] args) {{
        ClassOne.oneMethod();
        ClassTwo.twoMethod();
    }}
}}
"#
    )
    .map_err(|_| "Failed to write to `Main.java`.".to_string())?;

    // Create sample ClassOne.java
    let classone_dir = src_dir.join("classone");
    fs::create_dir(&classone_dir).map_err(|_| "Failed to create `classone` directory.".to_string())?;
    let classone_path = classone_dir.join("ClassOne.java");
    let mut classone_file =
        File::create(classone_path).map_err(|_| "Failed to create `ClassOne.java`.".to_string())?;
    writeln!(
        classone_file,
        r#"public class ClassOne {{

    public static void oneMethod() {{
        System.out.println("ClassOne method");
    }}
}}
"#
    )
    .map_err(|_| "Failed to write to `ClassOne.java`.".to_string())?;

    // Create sample ClassTwo.java
    let classtwo_dir = src_dir.join("classtwo");
    fs::create_dir(&classtwo_dir).map_err(|_| "Failed to create `classtwo` directory.".to_string())?;
    let classtwo_path = classtwo_dir.join("ClassTwo.java");
    let mut classtwo_file =
        File::create(classtwo_path).map_err(|_| "Failed to create `ClassTwo.java`.".to_string())?;
    writeln!(
        classtwo_file,
        r#"public class ClassTwo {{

    public static void twoMethod() {{
        System.out.println("ClassTwo method");
    }}
}}
"#
    )
    .map_err(|_| "Failed to write to `ClassTwo.java`.".to_string())?;

    printinfo("Initialized a new RSJ project.");

    Ok(())
}

pub fn build_project() -> Result<(), String> {
    let config = load_config().map_err(|e| e)?;

    let src_dir = Path::new(config.project.root_path.as_deref().unwrap_or(".")).join("src");
    if !src_dir.exists() {
        return Err("Error: `src` directory is missing.".to_string());
    }

    // Create a temporary directory for the build
    let temp_path = Path::new(OUTPUT_PATH).to_path_buf();
    fs::create_dir_all(&temp_path)
        .map_err(|_| "Failed to create temporary build directory.".to_string())?;

    // Create .gitignore for the whole folder
    let gitignore_path = temp_path.join(".gitignore");
    let mut gitignore_file =
        File::create(&gitignore_path).map_err(|_| "Failed to create `.gitignore`.".to_string())?;
    writeln!(gitignore_file, "*\n").map_err(|_| "Failed to write to `.gitignore`.".to_string())?;

    printinfo(&format!(
        "Using temporary build directory: {}",
        temp_path.display()
    ));

    gradle_maven_seperator();

    match config.project.build_tool.to_lowercase().as_str() {
        "gradle" => setup_gradle_project(&config, src_dir.to_str().unwrap(), &temp_path)?,
        "maven" => setup_maven_project(&config, src_dir.to_str().unwrap(), &temp_path)?,
        _ => {
            printerr("Unsupported build tool specified.");
        }
    }

    // Define the Gradle project directory
    let gradle_project_dir = temp_path.join(GRADLE_PATH);

    // Build the project
    let build_status = match config.project.build_tool.to_lowercase().as_str() {
        "gradle" => {
            #[cfg(target_os = "windows")]
            {
                Command::new("cmd")
                    .args(&["/C", "gradle", "build"])
                    .current_dir(&gradle_project_dir)
                    .status()
            }
            #[cfg(not(target_os = "windows"))]
            {
                Command::new("gradle")
                    .arg("build")
                    .current_dir(&gradle_project_dir)
                    .status()
            }
        }
        "maven" => Command::new("mvn")
            .arg("package")
            .current_dir(&temp_path.join(MAVEN_PATH))
            .status(),
        _ => unreachable!(),
    }
    .map_err(|_| "Failed to run the build tool.".to_string())?;

    rsj_seperator();

    if build_status.success() {
        printinfo("Build succeeded! Output is in the temporary directory.");
        Ok(())
    } else {
        Err("Build failed.".to_string())
    }
}

pub fn clean_build() -> Result<(), String> {
    if Path::new(OUTPUT_PATH).exists() {
        fs::remove_dir_all(OUTPUT_PATH)
            .map_err(|_| "Failed to clean the build output.".to_string())?;
        printinfo("Build output cleaned.");
    } else {
        printinfo("Build output not found.");
    }
    Ok(())
}
