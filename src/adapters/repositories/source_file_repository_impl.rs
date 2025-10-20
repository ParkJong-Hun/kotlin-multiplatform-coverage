use anyhow::Result;
use std::collections::HashMap;
use std::fs;

use crate::domain::{Language, Platform, SourceFile, SourceFileRepository};
use crate::adapters::platforms::{PlatformRegistry, PlatformType};
use crate::utils::FileUtils;

/// Adapter implementation of SourceFileRepository
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

    fn find_app_files(&self, project_path: &str) -> Result<HashMap<Platform, Vec<String>>> {
        let path = std::path::Path::new(project_path);
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
