# RSJ: Rusty Java
[![Tests](https://github.com/zanderlewis/rusty-java/actions/workflows/tests.yml/badge.svg)](https://github.com/zanderlewis/rusty-java/actions/workflows/tests.yml)

RSJ is a tool written in Rust for using a cargo-like file structure for Gradle Java projects. It is designed to be a simple and easy-to-use tool for managing Java projects with a clean, minimal directory structure.

## Features

- Simple project structure inspired by Cargo
- Enhanced Gradle support with modern Java features
- Automatic namespace/package management
- Gradle wrapper included for reproducible builds
- Optimized Gradle configuration for better performance
- JUnit 5 testing support
- ShadowJar for creating fat JARs with dependencies

## Getting Started

```
$ rsj init    # Create a new RSJ project
$ rsj build   # Build the project
$ rsj run     # Build and run the project
$ rsj clean   # Clean build artifacts
```