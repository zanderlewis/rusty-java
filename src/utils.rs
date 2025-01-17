use colored::Colorize;
use std::fs;
use std::path::Path;
use regex::Regex;

pub const GRADLE_PATH: &str = "gradle";
pub const MAVEN_PATH: &str = "maven";
pub const OUTPUT_PATH: &str = "rsj_build";

pub fn printerr(msg: &str) {
    println!("{}{}", "[ERROR] ".red().bold(), msg);
}

pub fn printinfo(msg: &str) {
    println!("{}{}", "[INFO] ".blue().bold(), msg);
}

pub fn copy_src_files(src_dir: &str, dest_dir: &Path) -> Result<(), String> {
    let package_regex = Regex::new(r"^package\s+([\w\\.]+);").map_err(|e| e.to_string())?;

    for entry in fs::read_dir(src_dir).map_err(|_| "Failed to read src directory.".to_string())? {
        let entry = entry.map_err(|_| "Failed to process directory entry.".to_string())?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("java") {
            let content = fs::read_to_string(&path).map_err(|_| "Failed to read Java file.".to_string())?;
            if let Some(captures) = package_regex.captures(&content) {
                let package = captures.get(1).unwrap().as_str();
                let package_path = package.replace('.', "/");
                let target_dir = dest_dir.join(package_path);
                fs::create_dir_all(&target_dir).map_err(|_| "Failed to create package directory.".to_string())?;
                fs::copy(&path, target_dir.join(path.file_name().unwrap())).map_err(|_| "Failed to copy Java file.".to_string())?;
            } else {
                // Handle default package
                fs::copy(&path, dest_dir.join(path.file_name().unwrap())).map_err(|_| "Failed to copy Java file.".to_string())?;
            }
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

pub fn gradle_maven_seperator() {
    println!(
        "{}",
        "==============================Gradle||Maven==============================".green()
    );
}

pub fn basic_seperator() {
    println!(
        "{}",
        "=========================================================================".green()
    );
}
