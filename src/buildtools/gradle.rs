use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::config::Config;
use crate::utils::{copy_src_files, GRADLE_PATH};

pub fn setup_gradle_project(
    config: &Config,
    src_dir: &str,
    temp_path: &Path,
) -> Result<(), String> {
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

    // Create `settings.gradle` and set the project name
    let mut settings_file = File::create(&settings_file_path)
        .map_err(|_| "Failed to create `settings.gradle`.".to_string())?;
    writeln!(
        settings_file,
        r#"rootProject.name = '{}'

// Apply centralized repository configuration
dependencyResolutionManagement {{
    repositories {{
        mavenCentral()
        google()
        gradlePluginPortal()
    }}
}}"#,
        config.project.name
    )
    .map_err(|_| "Failed to write to `settings.gradle`.".to_string())?;

    // Create gradle.properties with performance options
    let mut properties_file = File::create(&gradle_properties_path)
        .map_err(|_| "Failed to create `gradle.properties`.".to_string())?;
    writeln!(
        properties_file,
        r#"# Gradle performance improvements
org.gradle.jvmargs=-Xmx2g -XX:MaxMetaspaceSize=512m -XX:+HeapDumpOnOutOfMemoryError
org.gradle.parallel=true
org.gradle.caching=true
org.gradle.configureondemand=true

# Enable file system watching for faster incremental builds
org.gradle.vfs.watch=true"#
    )
    .map_err(|_| "Failed to write to `gradle.properties`.".to_string())?;

    // Create `build.gradle`
    let mut build_file = File::create(&build_file_path)
        .map_err(|_| "Failed to create `build.gradle`.".to_string())?;
    writeln!(
        build_file,
        r#"plugins {{
    id 'java'
    id 'application'
    id 'java-library'
    id 'com.github.johnrengelman.shadow' version '7.1.2'
}}

group = '{}'
version = '{}'

application {{
    mainClass = '{}'
}}

java {{
    withSourcesJar()
    withJavadocJar()
}}

repositories {{
    mavenCentral()
    google()
}}

dependencies {{
{}
    // Testing dependencies
    testImplementation 'org.junit.jupiter:junit-jupiter-api:5.8.2'
    testRuntimeOnly 'org.junit.jupiter:junit-jupiter-engine:5.8.2'
}}

test {{
    useJUnitPlatform()
    testLogging {{
        events "passed", "skipped", "failed"
    }}
}}

tasks.named('jar') {{
    manifest {{
        attributes(
            'Main-Class': '{}'
        )
    }}
}}

shadowJar {{
    archiveClassifier.set('')
    archiveVersion.set(version)
    mergeServiceFiles()
}}"#,
        config.project.name,
        config.project.version,
        config.project.base_namespace.to_owned() + "." + &config.project.main_class,
        generate_gradle_dependencies(&config.dependencies),
        config.project.base_namespace.to_owned() + "." + &config.project.main_class
    )
    .map_err(|_| "Failed to write to `build.gradle`.".to_string())?;

    // Create a Gradle wrapper
    create_gradle_wrapper(&gradle_dir)?;

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

fn create_gradle_wrapper(gradle_dir: &Path) -> Result<(), String> {
    // Create the gradle/wrapper directory
    fs::create_dir_all(gradle_dir.join("gradle/wrapper"))
        .map_err(|_| "Failed to create Gradle wrapper directory.".to_string())?;

    // Create gradle-wrapper.properties
    let mut properties_file =
        File::create(gradle_dir.join("gradle/wrapper/gradle-wrapper.properties"))
            .map_err(|_| "Failed to create gradle-wrapper.properties.".to_string())?;

    writeln!(
        properties_file,
        r#"distributionBase=GRADLE_USER_HOME
distributionPath=wrapper/dists
distributionUrl=https\://services.gradle.org/distributions/gradle-8.4-bin.zip
networkTimeout=10000
zipStoreBase=GRADLE_USER_HOME
zipStorePath=wrapper/dists"#
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
