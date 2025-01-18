use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::config::Config;
use crate::utils::{MAVEN_PATH, copy_src_files};

pub fn setup_maven_project(config: &Config, src_dir: &str, temp_path: &Path) -> Result<(), String> {
    let maven_dir = temp_path.join(MAVEN_PATH);
    let pom_file_path = maven_dir.join("pom.xml");

    fs::create_dir_all(maven_dir.join("src/main/java"))
        .map_err(|_| "Failed to create Maven project structure.".to_string())?;

    // Copy source files
    copy_src_files(src_dir, &maven_dir.join("src/main/java".to_owned() + "/" + &config.project.base_namespace.replace(".", "/")), &config.project.base_namespace)?;

    // Create `pom.xml`
    let mut pom_file = File::create(pom_file_path)
        .map_err(|_| "Failed to create `pom.xml`.".to_string())?;
    writeln!(
        pom_file,
        r#"
<project xmlns="http://maven.apache.org/POM/4.0.0" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
    xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>
    <groupId>{}</groupId>
    <artifactId>{}</artifactId>
    <version>{}</version>
    <dependencies>
{}
    </dependencies>
    <build>
        <plugins>
            <plugin>
                <groupId>org.apache.maven.plugins</groupId>
                <artifactId>maven-compiler-plugin</artifactId>
                <version>3.8.1</version>
                <configuration>
                    <source>11</source>
                    <target>11</target>
                </configuration>
            </plugin>
            <plugin>
                <groupId>org.apache.maven.plugins</groupId>
                <artifactId>maven-jar-plugin</artifactId>
                <version>3.2.0</version>
                <configuration>
                    <archive>
                        <manifest>
                            <mainClass>{}</mainClass>
                        </manifest>
                    </archive>
                </configuration>
            </plugin>
            <plugin>
                <groupId>org.apache.maven.plugins</groupId>
                <artifactId>maven-shade-plugin</artifactId>
                <version>3.2.4</version>
                <executions>
                    <execution>
                        <phase>package</phase>
                        <goals>
                            <goal>shade</goal>
                        </goals>
                        <configuration>
                            <createDependencyReducedPom>false</createDependencyReducedPom>
                            <transformers>
                                <transformer implementation="org.apache.maven.plugins.shade.resource.ManifestResourceTransformer">
                                    <mainClass>{}</mainClass>
                                </transformer>
                            </transformers>
                        </configuration>
                    </execution>
                </executions>
            </plugin>
        </plugins>
    </build>
</project>"#,
        config.project.name,
        config.project.name,
        config.project.version,
        generate_maven_dependencies(&config.dependencies),
        config.project.base_namespace.to_owned() + "." + &config.project.main_class,
        config.project.base_namespace.to_owned() + "." + &config.project.main_class
    ).map_err(|_| "Failed to write to `pom.xml`.".to_string())?;

    Ok(())
}

fn generate_maven_dependencies(dependencies: &Option<std::collections::HashMap<String, String>>) -> String {
    dependencies
        .as_ref()
        .map(|deps| {
            deps.iter()
                .map(|(_, dep)| {
                    let parts: Vec<&str> = dep.split(':').collect();
                    if parts.len() == 3 {
                        format!(
                            r#"
    <dependency>
        <groupId>{}</groupId>
        <artifactId>{}</artifactId>
        <version>{}</version>
    </dependency>"#,
                            parts[0], parts[1], parts[2]
                        )
                    } else {
                        String::new()
                    }
                })
                .collect::<Vec<String>>()
                .join("\n")
        })
        .unwrap_or_default()
}