use colored::Colorize;
use std::fs;
use std::path::Path;

pub const GRADLE_PATH: &str = "gradle";
pub const OUTPUT_PATH: &str = "rsj_build";

pub fn printerr(msg: &str) {
    println!("{}{}", "[ERROR] ".red().bold(), msg);
}

pub fn printinfo(msg: &str) {
    println!("{}{}", "[INFO] ".blue().bold(), msg);
}

pub fn copy_src_files(src_dir: &str, dest_dir: &Path, base_namespace: &str) -> Result<(), String> {
    let src_path = Path::new(src_dir);

    // Iterate through all Java files in the src directory recursively
    for entry in fs::read_dir(src_path).map_err(|_| "Failed to read src directory.".to_string())? {
        let entry = entry.map_err(|_| "Failed to process directory entry.".to_string())?;
        let path = entry.path();

        if path.is_dir() {
            // Recursively copy from subdirectories
            copy_src_files(
                path.to_str().unwrap(),
                &dest_dir.join(
                    path.strip_prefix(src_path)
                        .map_err(|_| "Failed to determine relative path.".to_string())?,
                ),
                // Append directory name to base namespace
                &format!(
                    "{}.{}",
                    base_namespace,
                    path.file_name().unwrap().to_str().unwrap()
                ),
            )?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("java") {
            // Calculate relative path from src directory
            let relative_path = path
                .strip_prefix(src_path)
                .map_err(|_| "Failed to determine relative path.".to_string())?;

            // Derive package name from relative path
            let parent = relative_path.parent().unwrap_or_else(|| Path::new(""));
            let relative_package = if parent.as_os_str().is_empty() {
                String::from("")
            } else {
                parent
                    .to_str()
                    .ok_or("Failed to convert path to string.".to_string())?
                    .replace("\\", ".")
                    .replace("/", ".")
            };

            let package = if relative_package.is_empty() {
                base_namespace.to_string()
            } else {
                format!("{}.{}", base_namespace, relative_package)
            };

            // Create target directory based on package
            let target_dir = dest_dir.join(parent);
            fs::create_dir_all(&target_dir)
                .map_err(|_| "Failed to create package directory.".to_string())?;

            // Read the original Java file
            let content =
                fs::read_to_string(&path).map_err(|_| "Failed to read Java file.".to_string())?;

            // Add or replace package declaration
            let new_content = if let Some(start) = content.find("package ") {
                let end = content[start..]
                    .find(';')
                    .ok_or("Failed to find end of package declaration.".to_string())?
                    + start
                    + 1;
                format!("package {};\n{}", package, &content[end..])
            } else {
                format!("package {};\n{}", package, content)
            };

            // Write the modified content to the target directory
            let target_file_path = target_dir.join(path.file_name().unwrap());
            fs::write(&target_file_path, new_content)
                .map_err(|_| "Failed to write Java file.".to_string())?;
        }
    }
    Ok(())
}

pub fn rsj_seperator() {
    println!(
        "{}",
        "===================================RSJ===================================".green()
    );
}

pub fn gradle_seperator() {
    println!(
        "{}",
        "==============================Gradle Build==============================".green()
    );
}

pub fn basic_seperator() {
    println!(
        "{}",
        "=========================================================================".green()
    );
}
