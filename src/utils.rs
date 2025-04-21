use colored::Colorize;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub const GRADLE_PATH: &str = "gradle";
pub const OUTPUT_PATH: &str = "rsj_build";

pub fn printerr(msg: &str) {
    println!("{}{}", "[ERROR] ".red().bold(), msg);
}

pub fn printinfo(msg: &str) {
    println!("{}{}", "[INFO] ".blue().bold(), msg);
}

pub fn copy_src_files(src_dir: &str, dest_dir: &Path, base_namespace: &str) -> Result<(), String> {
    for entry in WalkDir::new(src_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().and_then(|ext| ext.to_str()) == Some("java"))
    {
        let path = entry.path();
        let relative_path = path
            .strip_prefix(src_dir)
            .map_err(|_| "Failed to determine relative path.".to_string())?;
        let parent = relative_path.parent().unwrap_or_else(|| Path::new(""));
        let relative_package = parent
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, ".");
        let package = if relative_package.is_empty() {
            base_namespace.to_string()
        } else {
            format!("{}.{}", base_namespace, relative_package)
        };

        let target_dir = dest_dir.join(parent);
        fs::create_dir_all(&target_dir)
            .map_err(|e| format!("Failed to create package directory: {}", e))?;

        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read Java file: {}", e))?;
        let new_content = if content.contains("package ") {
            let parts: Vec<&str> = content.splitn(2, ';').collect();
            format!("package {};{}", package, parts.get(1).unwrap_or(&""))
        } else {
            format!("package {};{}", package, content)
        };

        let target_file = target_dir.join(path.file_name().unwrap());
        fs::write(&target_file, new_content)
            .map_err(|e| format!("Failed to write Java file: {}", e))?;
    }
    Ok(())
}

pub fn seperator() {
    println!(
        "{}",
        "=========================================================================".green()
    );
}
