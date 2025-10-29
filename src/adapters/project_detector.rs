/// Dynamic project detection module
/// Automatically detects KMP, Android, and iOS projects by analyzing project structure
/// and configuration files

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Detected project information
#[derive(Debug, Clone)]
pub struct DetectedProject {
    pub project_type: ProjectType,
    pub root_path: PathBuf,
    pub source_dirs: Vec<PathBuf>,
}

/// Type of detected project
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectType {
    KotlinMultiplatform,
    Android,
    IOS,
}

/// Main project detector
pub struct ProjectDetector;

impl ProjectDetector {
    /// Scans a directory and detects all projects
    pub fn detect_all_projects(root_path: &Path) -> Result<Vec<DetectedProject>> {
        let mut projects = Vec::new();

        // Find KMP projects
        projects.extend(Self::find_kmp_projects(root_path)?);

        // Find Android projects
        projects.extend(Self::find_android_projects(root_path)?);

        // Find iOS projects
        projects.extend(Self::find_ios_projects(root_path)?);

        Ok(projects)
    }

    /// Finds Kotlin Multiplatform projects
    fn find_kmp_projects(root_path: &Path) -> Result<Vec<DetectedProject>> {
        let mut projects = Vec::new();

        // Strategy 1: Look for build.gradle.kts with kotlin("multiplatform")
        for entry in WalkDir::new(root_path)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.file_name() == Some("build.gradle.kts".as_ref())
                || path.file_name() == Some("build.gradle".as_ref())
            {
                if Self::is_kmp_gradle_file(path)? {
                    if let Some(project_dir) = path.parent() {
                        let source_dirs = Self::find_kmp_source_dirs(project_dir)?;
                        if !source_dirs.is_empty() {
                            projects.push(DetectedProject {
                                project_type: ProjectType::KotlinMultiplatform,
                                root_path: project_dir.to_path_buf(),
                                source_dirs,
                            });
                        }
                    }
                }
            }
        }

        // Strategy 2: Look for typical KMP directory structures
        if projects.is_empty() {
            projects.extend(Self::find_kmp_by_structure(root_path)?);
        }

        Ok(projects)
    }

    /// Checks if a gradle file is a KMP project
    fn is_kmp_gradle_file(path: &Path) -> Result<bool> {
        let content = fs::read_to_string(path)?;

        // Check for multiplatform plugin
        let has_multiplatform = content.contains("kotlin(\"multiplatform\")")
            || content.contains("kotlin-multiplatform")
            || content.contains("org.jetbrains.kotlin.multiplatform");

        // Check for KMP-specific configurations
        let has_kmp_config = content.contains("commonMain")
            || content.contains("androidMain")
            || content.contains("iosMain")
            || content.contains("sourceSets");

        Ok(has_multiplatform || has_kmp_config)
    }

    /// Finds KMP source directories within a project
    fn find_kmp_source_dirs(project_root: &Path) -> Result<Vec<PathBuf>> {
        let mut source_dirs = Vec::new();

        // Common KMP source set names
        let kmp_source_sets = [
            "commonMain/kotlin",
            "commonMain",
            "androidMain/kotlin",
            "androidMain",
            "iosMain/kotlin",
            "iosMain",
            "commonTest/kotlin",
            "commonTest",
            "src/commonMain/kotlin",
            "src/commonMain",
            "src/androidMain/kotlin",
            "src/androidMain",
            "src/iosMain/kotlin",
            "src/iosMain",
        ];

        for source_set in &kmp_source_sets {
            let src_path = project_root.join(source_set);
            if src_path.exists() && src_path.is_dir() {
                source_dirs.push(src_path);
            }
        }

        // Also look for "shared" module (common in KMP projects)
        let shared_path = project_root.join("shared/src");
        if shared_path.exists() {
            for entry in WalkDir::new(shared_path)
                .max_depth(3)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if path.is_dir()
                    && (path.ends_with("commonMain") || path.ends_with("kotlin"))
                {
                    source_dirs.push(path.to_path_buf());
                }
            }
        }

        Ok(source_dirs)
    }

    /// Finds KMP projects by directory structure patterns
    fn find_kmp_by_structure(root_path: &Path) -> Result<Vec<DetectedProject>> {
        let mut projects = Vec::new();

        // Look for directories with "shared" + commonMain structure
        for entry in WalkDir::new(root_path)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_dir() && path.file_name() == Some("shared".as_ref()) {
                let common_main = path.join("src/commonMain");
                if common_main.exists() {
                    let source_dirs = Self::find_kmp_source_dirs(path)?;
                    if !source_dirs.is_empty() {
                        projects.push(DetectedProject {
                            project_type: ProjectType::KotlinMultiplatform,
                            root_path: path.to_path_buf(),
                            source_dirs,
                        });
                    }
                }
            }
        }

        Ok(projects)
    }

    /// Finds Android projects
    fn find_android_projects(root_path: &Path) -> Result<Vec<DetectedProject>> {
        let mut projects = Vec::new();

        // Strategy 1: Look for AndroidManifest.xml
        for entry in WalkDir::new(root_path)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.file_name() == Some("AndroidManifest.xml".as_ref()) {
                if let Some(manifest_dir) = path.parent() {
                    // Go up to find the module root (usually one or two levels up)
                    let mut project_root = manifest_dir;
                    for _ in 0..3 {
                        if let Some(parent) = project_root.parent() {
                            let build_gradle = parent.join("build.gradle");
                            let build_gradle_kts = parent.join("build.gradle.kts");
                            if build_gradle.exists() || build_gradle_kts.exists() {
                                project_root = parent;
                                break;
                            }
                            project_root = parent;
                        }
                    }

                    let source_dirs = Self::find_android_source_dirs(project_root)?;
                    if !source_dirs.is_empty() {
                        projects.push(DetectedProject {
                            project_type: ProjectType::Android,
                            root_path: project_root.to_path_buf(),
                            source_dirs,
                        });
                    }
                }
            }
        }

        // Strategy 2: Look for build.gradle with Android plugin
        if projects.is_empty() {
            projects.extend(Self::find_android_by_gradle(root_path)?);
        }

        Ok(projects)
    }

    /// Finds Android source directories
    fn find_android_source_dirs(project_root: &Path) -> Result<Vec<PathBuf>> {
        let mut source_dirs = Vec::new();

        // Common Android source directories
        let android_src_patterns = [
            "src/main/java",
            "src/main/kotlin",
            "src/main",
            "app/src/main/java",
            "app/src/main/kotlin",
            "app/src/main",
            "android/src/main/java",
            "android/src/main/kotlin",
            "androidApp/src/main/java",
            "androidApp/src/main/kotlin",
        ];

        for pattern in &android_src_patterns {
            let src_path = project_root.join(pattern);
            if src_path.exists() && src_path.is_dir() {
                // Check if it contains actual source files
                if Self::contains_source_files(&src_path, &["kt", "java"])? {
                    source_dirs.push(src_path);
                }
            }
        }

        Ok(source_dirs)
    }

    /// Finds Android projects by analyzing gradle files
    fn find_android_by_gradle(root_path: &Path) -> Result<Vec<DetectedProject>> {
        let mut projects = Vec::new();

        for entry in WalkDir::new(root_path)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.file_name() == Some("build.gradle.kts".as_ref())
                || path.file_name() == Some("build.gradle".as_ref())
            {
                if Self::is_android_gradle_file(path)? {
                    if let Some(project_dir) = path.parent() {
                        let source_dirs = Self::find_android_source_dirs(project_dir)?;
                        if !source_dirs.is_empty() {
                            projects.push(DetectedProject {
                                project_type: ProjectType::Android,
                                root_path: project_dir.to_path_buf(),
                                source_dirs,
                            });
                        }
                    }
                }
            }
        }

        Ok(projects)
    }

    /// Checks if a gradle file is an Android project
    fn is_android_gradle_file(path: &Path) -> Result<bool> {
        let content = fs::read_to_string(path)?;

        Ok(content.contains("com.android.application")
            || content.contains("com.android.library")
            || content.contains("android {"))
    }

    /// Finds iOS projects
    fn find_ios_projects(root_path: &Path) -> Result<Vec<DetectedProject>> {
        let mut projects = Vec::new();

        // Strategy 1: Look for .xcodeproj or .xcworkspace
        for entry in WalkDir::new(root_path)
            .max_depth(4)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if let Some(file_name) = path.file_name() {
                let name = file_name.to_string_lossy();
                if name.ends_with(".xcodeproj") || name.ends_with(".xcworkspace") {
                    if let Some(project_dir) = path.parent() {
                        let source_dirs = Self::find_ios_source_dirs(project_dir)?;
                        if !source_dirs.is_empty() {
                            projects.push(DetectedProject {
                                project_type: ProjectType::IOS,
                                root_path: project_dir.to_path_buf(),
                                source_dirs,
                            });
                        }
                    }
                }
            }
        }

        // Strategy 2: Look for typical iOS directory structures
        if projects.is_empty() {
            projects.extend(Self::find_ios_by_structure(root_path)?);
        }

        Ok(projects)
    }

    /// Finds iOS source directories
    fn find_ios_source_dirs(project_root: &Path) -> Result<Vec<PathBuf>> {
        let mut source_dirs = Vec::new();

        // Common iOS app directory names
        let ios_dir_names = ["iosApp", "iOS", "ios"];

        for dir_name in &ios_dir_names {
            let ios_path = project_root.join(dir_name);
            if ios_path.exists() && ios_path.is_dir() {
                // Check if it contains Swift or Objective-C files
                if Self::contains_source_files(&ios_path, &["swift", "m", "mm"])? {
                    source_dirs.push(ios_path.clone());
                }

                // Also check subdirectory with the same name (e.g., iosApp/iosApp)
                let sub_dir = ios_path.join(dir_name);
                if sub_dir.exists()
                    && Self::contains_source_files(&sub_dir, &["swift", "m", "mm"])?
                {
                    source_dirs.push(sub_dir);
                }
            }
        }

        // Look for any directory containing Swift files
        if source_dirs.is_empty() {
            for entry in WalkDir::new(project_root)
                .max_depth(3)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if path.is_dir()
                    && Self::contains_source_files(path, &["swift", "m", "mm"])?
                {
                    source_dirs.push(path.to_path_buf());
                }
            }
        }

        Ok(source_dirs)
    }

    /// Finds iOS projects by directory structure
    fn find_ios_by_structure(root_path: &Path) -> Result<Vec<DetectedProject>> {
        let mut projects = Vec::new();

        let ios_indicators = ["iosApp", "iOS", "ios"];

        for indicator in &ios_indicators {
            let ios_path = root_path.join(indicator);
            if ios_path.exists() && ios_path.is_dir() {
                let source_dirs = Self::find_ios_source_dirs(&ios_path)?;
                if !source_dirs.is_empty() {
                    projects.push(DetectedProject {
                        project_type: ProjectType::IOS,
                        root_path: ios_path,
                        source_dirs,
                    });
                }
            }
        }

        Ok(projects)
    }

    /// Checks if a directory contains source files with given extensions
    fn contains_source_files(dir: &Path, extensions: &[&str]) -> Result<bool> {
        for entry in WalkDir::new(dir)
            .max_depth(10)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if let Some(ext) = entry.path().extension() {
                if let Some(ext_str) = ext.to_str() {
                    if extensions.contains(&ext_str) {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    /// Gets all source files from a project
    pub fn get_all_source_files(project: &DetectedProject) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        let extensions = match project.project_type {
            ProjectType::KotlinMultiplatform => vec!["kt", "kts"],
            ProjectType::Android => vec!["kt", "kts", "java"],
            ProjectType::IOS => vec!["swift", "m", "mm", "h"],
        };

        for source_dir in &project.source_dirs {
            for entry in WalkDir::new(source_dir)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    if let Some(ext) = entry.path().extension() {
                        if let Some(ext_str) = ext.to_str() {
                            if extensions.contains(&ext_str) {
                                files.push(entry.path().to_path_buf());
                            }
                        }
                    }
                }
            }
        }

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_kmp_project() -> Result<()> {
        let temp = TempDir::new()?;
        let root = temp.path();

        // Create KMP project structure
        let shared = root.join("shared");
        fs::create_dir_all(shared.join("src/commonMain/kotlin"))?;
        fs::write(
            shared.join("build.gradle.kts"),
            r#"
            plugins {
                kotlin("multiplatform")
            }
            "#,
        )?;
        fs::write(
            shared.join("src/commonMain/kotlin/Test.kt"),
            "class Test",
        )?;

        let projects = ProjectDetector::detect_all_projects(root)?;
        let kmp_projects: Vec<_> = projects
            .iter()
            .filter(|p| p.project_type == ProjectType::KotlinMultiplatform)
            .collect();

        assert!(!kmp_projects.is_empty(), "Should detect KMP project");

        Ok(())
    }

    #[test]
    fn test_detect_android_project() -> Result<()> {
        let temp = TempDir::new()?;
        let root = temp.path();

        // Create Android project structure
        let app = root.join("app");
        fs::create_dir_all(app.join("src/main/java"))?;
        fs::write(
            app.join("src/main/AndroidManifest.xml"),
            r#"<manifest package="com.example"/>"#,
        )?;
        fs::write(app.join("src/main/java/Test.java"), "class Test {}")?;

        let projects = ProjectDetector::detect_all_projects(root)?;
        let android_projects: Vec<_> = projects
            .iter()
            .filter(|p| p.project_type == ProjectType::Android)
            .collect();

        assert!(!android_projects.is_empty(), "Should detect Android project");

        Ok(())
    }

    #[test]
    fn test_detect_ios_project() -> Result<()> {
        let temp = TempDir::new()?;
        let root = temp.path();

        // Create iOS project structure
        let ios_app = root.join("iosApp");
        fs::create_dir_all(&ios_app)?;
        fs::write(ios_app.join("ContentView.swift"), "import SwiftUI")?;

        let projects = ProjectDetector::detect_all_projects(root)?;
        let ios_projects: Vec<_> = projects
            .iter()
            .filter(|p| p.project_type == ProjectType::IOS)
            .collect();

        assert!(!ios_projects.is_empty(), "Should detect iOS project");

        Ok(())
    }
}
