use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::config::Config;
use crate::utils::{copy_src_files, GRADLE_PATH};

// Helper to write content to file with error mapping
fn write_file(path: &Path, content: &str) -> Result<(), String> {
    fs::write(path, content).map_err(|e| format!("Failed to write {}: {}", path.display(), e))
}

pub fn setup_gradle_project(
    config: &Config,
    src_dir: &str,
    temp_path: &Path,
) -> Result<(), String> {
    // determine versions from config or defaults
    let gradle_ver = config.project.gradle_version.as_deref().unwrap_or("8.4");
    let shadow_ver = config
        .project
        .shadow_plugin_version
        .as_deref()
        .unwrap_or("7.1.2");

    let gradle_dir = temp_path.join(GRADLE_PATH);
    let build_file_path = gradle_dir.join("build.gradle");
    let settings_file_path = gradle_dir.join("settings.gradle");
    let gradle_properties_path = gradle_dir.join("gradle.properties");

    // Create Gradle project structure
    fs::create_dir_all(gradle_dir.join("src/main/java"))
        .map_err(|_| "Failed to create Gradle project structure.".to_string())?;

    // Create resources directory for non-Java files
    fs::create_dir_all(gradle_dir.join("src/main/resources"))
        .map_err(|_| "Failed to create resources directory.".to_string())?;

    // Copy source files with base_namespace
    copy_src_files(
        src_dir,
        &gradle_dir.join(
            "src/main/java".to_owned() + "/" + &config.project.base_namespace.replace(".", "/"),
        ),
        &config.project.base_namespace,
    )?;

    // settings.gradle content
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
    write_file(&settings_file_path, &settings)?;

    // gradle.properties content
    let properties = r#"# Gradle performance improvements
org.gradle.jvmargs=-Xmx2g -XX:MaxMetaspaceSize=512m -XX:+HeapDumpOnOutOfMemoryError
org.gradle.parallel=true
org.gradle.caching=true
org.gradle.configureondemand=true

# Enable file system watching for faster incremental builds
org.gradle.vfs.watch=true"#;
    write_file(&gradle_properties_path, properties)?;

    let deps = generate_gradle_dependencies(&config.dependencies);
    // Determine if Shadow plugin should be included
    let shadow_enabled = config.project.use_shadow.unwrap_or(true);
    // Build plugins block dynamically
    let mut plugins = vec![
        "    id 'java'".to_string(),
        "    id 'application'".to_string(),
        "    id 'java-library'".to_string(),
    ];
    if shadow_enabled {
        plugins.push(format!(
            "    id 'com.github.johnrengelman.shadow' version '{}'",
            shadow_ver
        ));
    }
    let plugins_block = plugins.join("\n");
    // Assemble build.gradle content
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
    // Optionally append shadowJar task
    if shadow_enabled {
        build.push_str(
            "\nshadowJar {\n    archiveClassifier.set('')\n    archiveVersion.set(version)\n    mergeServiceFiles()\n}\n",
        );
    }
    write_file(&build_file_path, &build)?;

    // Create a Gradle wrapper with dynamic gradle version
    create_gradle_wrapper(&gradle_dir, gradle_ver)?;

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

fn create_gradle_wrapper(gradle_dir: &Path, gradle_version: &str) -> Result<(), String> {
    // Create the gradle/wrapper directory
    fs::create_dir_all(gradle_dir.join("gradle/wrapper"))
        .map_err(|_| "Failed to create Gradle wrapper directory.".to_string())?;

    // Write wrapper properties with dynamic gradle version
    let mut properties_file =
        File::create(gradle_dir.join("gradle/wrapper/gradle-wrapper.properties"))
            .map_err(|_| "Failed to create gradle-wrapper.properties.".to_string())?;

    writeln!(
        properties_file,
        "distributionBase=GRADLE_USER_HOME\ndistributionPath=wrapper/dists\ndistributionUrl=https://services.gradle.org/distributions/gradle-{}-bin.zip\nnetworkTimeout=10000\nzipStoreBase=GRADLE_USER_HOME\nzipStorePath=wrapper/dists",
        gradle_version
    )
    .map_err(|_| "Failed to write gradle-wrapper.properties.".to_string())?;

    // Create gradlew script (Unix)
    let mut gradlew_file = File::create(gradle_dir.join("gradlew"))
        .map_err(|_| "Failed to create gradlew script.".to_string())?;

    writeln!(
        gradlew_file,
        r#"#!/bin/sh
# Gradle wrapper script for Unix-based systems
exec "$(dirname "$0")"/gradle/wrapper/gradle-wrapper.jar "$@""#
    )
    .map_err(|_| "Failed to write gradlew script.".to_string())?;

    // Make the gradlew file executable on Unix systems
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

    // Create gradlew.bat script (Windows)
    let mut gradlew_bat_file = File::create(gradle_dir.join("gradlew.bat"))
        .map_err(|_| "Failed to create gradlew.bat script.".to_string())?;

    writeln!(
        gradlew_bat_file,
        r#"@rem Gradle wrapper script for Windows
@echo off
java -jar "%~dp0/gradle/wrapper/gradle-wrapper.jar" %*"#
    )
    .map_err(|_| "Failed to write gradlew.bat script.".to_string())?;

    Ok(())
}
