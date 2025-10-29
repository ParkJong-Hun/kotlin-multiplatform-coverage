use anyhow::Result;
use log::{debug, info};
use std::collections::HashMap;
use std::fs;

use crate::adapters::project_detector::{ProjectDetector, ProjectType};
use crate::adapters::platforms::{PlatformRegistry, PlatformType};
use crate::domain::{Language, Platform, SourceFile, SourceFileRepository};
use crate::utils::FileUtils;

/// Adapter implementation of SourceFileRepository with dynamic project detection
pub struct SourceFileRepositoryImpl {
    platform_registry: PlatformRegistry,
}

impl SourceFileRepositoryImpl {
    pub fn new() -> Self {
        Self {
            platform_registry: PlatformRegistry::new(),
        }
    }

    fn detect_language(file_path: &str) -> Language {
        if file_path.ends_with(".kt") || file_path.ends_with(".kts") {
            Language::Kotlin
        } else if file_path.ends_with(".java") {
            Language::Java
        } else if file_path.ends_with(".swift") {
            Language::Swift
        } else {
            Language::ObjectiveC
        }
    }

    fn convert_platform(platform_type: &PlatformType) -> Platform {
        match platform_type {
            PlatformType::Android => Platform::Android,
            PlatformType::IOS => Platform::IOS,
        }
    }
}

impl Default for SourceFileRepositoryImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceFileRepository for SourceFileRepositoryImpl {
    fn find_kmp_files(&self, project_path: &str) -> Result<Vec<String>> {
        let path = std::path::Path::new(project_path);
        info!("🔍 Dynamically detecting KMP projects in: {}", project_path);

        // Use dynamic project detection
        let all_projects = ProjectDetector::detect_all_projects(path)?;
        let kmp_projects: Vec<_> = all_projects
            .iter()
            .filter(|p| p.project_type == ProjectType::KotlinMultiplatform)
            .collect();

        info!("✓ Found {} KMP project(s)", kmp_projects.len());

        let mut kmp_files = Vec::new();

        for project in kmp_projects {
            debug!("  KMP project root: {:?}", project.root_path);
            debug!("  Source directories: {} dirs", project.source_dirs.len());

            let files = ProjectDetector::get_all_source_files(project)?;
            debug!("  Source files: {}", files.len());

            kmp_files.extend(files.into_iter().map(|p| p.to_string_lossy().to_string()));
        }

        // Fallback: if no projects detected, use legacy pattern matching
        if kmp_files.is_empty() {
            info!("⚠️  No KMP projects auto-detected, falling back to pattern matching");
            kmp_files = self.find_kmp_files_legacy(path)?;
        }

        info!("📦 Total KMP source files: {}", kmp_files.len());
        Ok(kmp_files)
    }

    fn find_app_files(&self, project_path: &str) -> Result<HashMap<Platform, Vec<String>>> {
        let path = std::path::Path::new(project_path);
        info!("🔍 Dynamically detecting platform projects in: {}", project_path);

        // Use dynamic project detection
        let all_projects = ProjectDetector::detect_all_projects(path)?;

        let mut result = HashMap::new();

        // Process Android projects
        let android_projects: Vec<_> = all_projects
            .iter()
            .filter(|p| p.project_type == ProjectType::Android)
            .collect();

        if !android_projects.is_empty() {
            info!("✓ Found {} Android project(s)", android_projects.len());
            let mut android_files = Vec::new();

            for project in android_projects {
                debug!("  Android project root: {:?}", project.root_path);
                let files = ProjectDetector::get_all_source_files(project)?;
                debug!("  Android files: {}", files.len());
                android_files.extend(files.into_iter().map(|p| p.to_string_lossy().to_string()));
            }

            info!("📱 Total Android files: {}", android_files.len());
            result.insert(Platform::Android, android_files);
        }

        // Process iOS projects
        let ios_projects: Vec<_> = all_projects
            .iter()
            .filter(|p| p.project_type == ProjectType::IOS)
            .collect();

        if !ios_projects.is_empty() {
            info!("✓ Found {} iOS project(s)", ios_projects.len());
            let mut ios_files = Vec::new();

            for project in ios_projects {
                debug!("  iOS project root: {:?}", project.root_path);
                let files = ProjectDetector::get_all_source_files(project)?;
                debug!("  iOS files: {}", files.len());
                ios_files.extend(files.into_iter().map(|p| p.to_string_lossy().to_string()));
            }

            info!("🍎 Total iOS files: {}", ios_files.len());
            result.insert(Platform::IOS, ios_files);
        }

        // Fallback: if no projects detected, use legacy platform registry
        if result.is_empty() {
            info!("⚠️  No platform projects auto-detected, falling back to pattern matching");
            result = self.find_app_files_legacy(path)?;
        }

        Ok(result)
    }

    fn read_source_file(&self, file_path: &str) -> Result<SourceFile> {
        let content = fs::read_to_string(file_path)?;
        let language = Self::detect_language(file_path);

        // Detect platform from path or file extension
        let platform = if file_path.contains("android") || file_path.ends_with(".kt") || file_path.ends_with(".java") {
            Platform::Android
        } else {
            Platform::IOS
        };

        Ok(SourceFile {
            path: file_path.to_string(),
            platform,
            language,
            content,
        })
    }

    fn count_code_lines(&self, content: &str, platform: Platform) -> usize {
        let platform_type = match platform {
            Platform::Android => PlatformType::Android,
            Platform::IOS => PlatformType::IOS,
        };

        if let Some(platform_impl) = self.platform_registry.get(platform_type) {
            platform_impl.count_code_lines(content)
        } else {
            0
        }
    }
}

// Legacy fallback methods
impl SourceFileRepositoryImpl {
    /// Legacy method for finding KMP files using hardcoded patterns
    fn find_kmp_files_legacy(&self, path: &std::path::Path) -> Result<Vec<String>> {
        let mut kmp_files = Vec::new();

        // Look for commonMain, androidMain, iosMain directories
        let kmp_patterns = ["commonMain", "androidMain", "iosMain", "shared/src"];

        for pattern in &kmp_patterns {
            let search_path = path.join(pattern);
            if search_path.exists() {
                let files = FileUtils::find_kotlin_files(&search_path);
                kmp_files.extend(files.into_iter().map(|p| p.to_string_lossy().to_string()));
            }
        }

        // Also search for 'shared' module
        let shared_path = path.join("shared");
        if shared_path.exists() {
            let files = FileUtils::find_kotlin_files(&shared_path);
            kmp_files.extend(files.into_iter().map(|p| p.to_string_lossy().to_string()));
        }

        Ok(kmp_files)
    }

    /// Legacy method for finding app files using platform registry
    fn find_app_files_legacy(&self, path: &std::path::Path) -> Result<HashMap<Platform, Vec<String>>> {
        let platform_files = self.platform_registry.find_all_app_files(path)?;

        let mut result = HashMap::new();
        for (platform_type, files) in platform_files {
            let platform = Self::convert_platform(&platform_type);
            let file_strings: Vec<String> = files
                .into_iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            result.insert(platform, file_strings);
        }

        Ok(result)
    }
}
