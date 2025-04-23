use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::utils::copy_src_files;

pub const GRADLE_PATH: &str = "gradle";

// Helper to write content to file with error mapping
fn write_file(path: &Path, content: &str) -> Result<(), String> {
    fs::write(path, content).map_err(|e| format!("Failed to write {}: {}", path.display(), e))
}

pub fn setup_gradle_project(
    config: &Config,
    src_dir: &str,
    temp_path: &Path,
) -> Result<(), String> {
    // Get versions from config or use defaults
    let gradle_ver = config.project.gradle_version.as_deref().unwrap_or("8.4");
    let shadow_ver = config
        .project
        .shadow_plugin_version
        .as_deref()
        .unwrap_or("7.1.2");
    let use_shadow = config.project.use_shadow.unwrap_or(true);

    // Setup directories
    let gradle_dir = temp_path.join(GRADLE_PATH);
    setup_gradle_directories(&gradle_dir)?;

    // Copy source files
    setup_source_files(config, src_dir, &gradle_dir)?;

    // Write Gradle configuration files
    write_gradle_config_files(config, &gradle_dir, shadow_ver, use_shadow)?;

    // Setup Gradle wrapper
    create_gradle_wrapper(&gradle_dir, gradle_ver)?;

    Ok(())
}

// Setup Gradle directory structure
fn setup_gradle_directories(gradle_dir: &Path) -> Result<(), String> {
    // Create main directories
    fs::create_dir_all(gradle_dir.join("src/main/java"))
        .map_err(|e| format!("Failed to create Gradle project structure: {}", e))?;

    // Create resources directory
    fs::create_dir_all(gradle_dir.join("src/main/resources"))
        .map_err(|e| format!("Failed to create resources directory: {}", e))?;

    Ok(())
}

// Copy source files to Gradle structure
fn setup_source_files(config: &Config, src_dir: &str, gradle_dir: &Path) -> Result<(), String> {
    // Prepare the path for Java files
    let java_path = format!(
        "src/main/java/{}",
        &config.project.base_namespace.replace(".", "/")
    );

    // Copy source files with correct namespace
    copy_src_files(
        src_dir,
        &gradle_dir.join(java_path),
        &config.project.base_namespace,
    )
}

// Write all Gradle configuration files
fn write_gradle_config_files(
    config: &Config,
    gradle_dir: &Path,
    shadow_ver: &str,
    use_shadow: bool,
) -> Result<(), String> {
    // Write settings.gradle
    write_settings_gradle(config, gradle_dir)?;

    // Write gradle.properties
    write_gradle_properties(gradle_dir)?;

    // Write build.gradle
    write_build_gradle(config, gradle_dir, shadow_ver, use_shadow)?;

    Ok(())
}

// Write settings.gradle file
fn write_settings_gradle(config: &Config, gradle_dir: &Path) -> Result<(), String> {
    let settings = format!(
        r#"rootProject.name = '{}'

dependencyResolutionManagement {{
    repositories {{
        mavenCentral()
        google()
        gradlePluginPortal()
    }}
}}"#,
        config.project.name
    );

    write_file(&gradle_dir.join("settings.gradle"), &settings)
}

// Write gradle.properties file
fn write_gradle_properties(gradle_dir: &Path) -> Result<(), String> {
    let properties = r#"# Gradle performance improvements
org.gradle.jvmargs=-Xmx2g -XX:MaxMetaspaceSize=512m -XX:+HeapDumpOnOutOfMemoryError
org.gradle.parallel=true
org.gradle.caching=true
org.gradle.configureondemand=true

# Enable file system watching for faster incremental builds
org.gradle.vfs.watch=true"#;

    write_file(&gradle_dir.join("gradle.properties"), properties)
}

// Write build.gradle file
fn write_build_gradle(
    config: &Config,
    gradle_dir: &Path,
    shadow_ver: &str,
    use_shadow: bool,
) -> Result<(), String> {
    // Generate dependencies section
    let deps = generate_gradle_dependencies(&config.dependencies);

    // Build plugins section
    let plugins_block = generate_plugins_block(shadow_ver, use_shadow);

    // Build the main Gradle file content
    let mut build = format!(
        "plugins {{\n{}\n}}\n\ngroup = '{}'\nversion = '{}'\n\napplication {{\n    mainClass = '{}.{}'\n}}\n\njava {{\n    withSourcesJar()\n    withJavadocJar()\n}}\n\nrepositories {{\n    mavenCentral()\n    google()\n}}\n\ndependencies {{\n{}\n    testImplementation 'org.junit.jupiter:junit-jupiter-api:5.8.2'\n    testRuntimeOnly 'org.junit.jupiter:junit-jupiter-engine:5.8.2'\n}}\n\ntest {{\n    useJUnitPlatform()\n    testLogging {{\n        events \"passed\", \"skipped\", \"failed\"\n    }}\n}}\n\ntasks.named('jar') {{\n    manifest {{\n        attributes(\n            'Main-Class': '{}.{}'\n        )\n    }}\n}}",
        plugins_block,
        config.project.name,
        config.project.version,
        config.project.base_namespace,
        config.project.main_class,
        deps,
        config.project.base_namespace,
        config.project.main_class
    );

    // Add shadow configuration if enabled
    if use_shadow {
        build.push_str(
            "\nshadowJar {\n    archiveClassifier.set('')\n    archiveVersion.set(version)\n    mergeServiceFiles()\n}\n",
        );
    }

    write_file(&gradle_dir.join("build.gradle"), &build)
}

// Generate the plugins block for build.gradle
fn generate_plugins_block(shadow_ver: &str, use_shadow: bool) -> String {
    let mut plugins = vec![
        "    id 'java'".to_string(),
        "    id 'application'".to_string(),
        "    id 'java-library'".to_string(),
    ];

    if use_shadow {
        plugins.push(format!(
            "    id 'com.github.johnrengelman.shadow' version '{}'",
            shadow_ver
        ));
    }

    plugins.join("\n")
}

// Generate dependencies section for build.gradle
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

// Create Gradle wrapper files
fn create_gradle_wrapper(gradle_dir: &Path, gradle_version: &str) -> Result<(), String> {
    // Create wrapper directory
    let wrapper_dir = gradle_dir.join("gradle/wrapper");
    fs::create_dir_all(&wrapper_dir)
        .map_err(|_| "Failed to create Gradle wrapper directory.".to_string())?;

    // Write wrapper properties
    let properties_content = format!(
        "distributionBase=GRADLE_USER_HOME\ndistributionPath=wrapper/dists\ndistributionUrl=https://services.gradle.org/distributions/gradle-{}-bin.zip\nnetworkTimeout=10000\nzipStoreBase=GRADLE_USER_HOME\nzipStorePath=wrapper/dists",
        gradle_version
    );
    write_file(
        &wrapper_dir.join("gradle-wrapper.properties"),
        &properties_content,
    )?;

    // Create Unix wrapper script
    let gradlew_content = r#"#!/bin/sh
# Gradle wrapper script for Unix-based systems
exec "$(dirname "$0")"/gradle/wrapper/gradle-wrapper.jar "$@""#;
    write_file(&gradle_dir.join("gradlew"), gradlew_content)?;

    // Make the script executable on Unix systems
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(gradle_dir.join("gradlew"))
            .map_err(|_| "Failed to get gradlew metadata.".to_string())?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(gradle_dir.join("gradlew"), permissions)
            .map_err(|_| "Failed to set executable permissions on gradlew.".to_string())?;
    }

    // Create Windows batch script
    let gradlew_bat_content = r#"@rem Gradle wrapper script for Windows
@echo off
java -jar "%~dp0/gradle/wrapper/gradle-wrapper.jar" %*"#;
    write_file(&gradle_dir.join("gradlew.bat"), gradlew_bat_content)?;

    Ok(())
}
