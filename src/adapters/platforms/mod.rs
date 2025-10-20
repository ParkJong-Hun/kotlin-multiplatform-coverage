use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::analyzer::models::{SymbolUsage, UsageLocation};

pub mod android;
pub mod ios;

/// Platform type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlatformType {
    Android,
    IOS,
}

impl PlatformType {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            PlatformType::Android => "Android",
            PlatformType::IOS => "iOS",
        }
    }
}

/// Trait for platform-specific analysis
pub trait Platform: Send + Sync {
    /// Returns the platform type
    fn platform_type(&self) -> PlatformType;

    /// Returns file extensions this platform uses
    #[allow(dead_code)]
    fn file_extensions(&self) -> Vec<&str>;

    /// Returns directory patterns to search for app files
    fn app_directory_patterns(&self) -> Vec<&str>;

    /// Detects if a file belongs to this platform
    #[allow(dead_code)]
    fn is_platform_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return self.file_extensions().contains(&ext_str);
            }
        }
        false
    }

    /// Finds all app files for this platform in the given project path
    fn find_app_files(&self, project_path: &Path) -> Result<Vec<PathBuf>>;

    /// Detects usage of KMP symbols in a file
    #[allow(dead_code)]
    fn detect_symbol_usage(
        &self,
        file_path: &Path,
        kmp_symbols: &[String],
    ) -> Result<HashMap<String, SymbolUsage>>;

    /// Extracts imports from a file
    #[allow(dead_code)]
    fn extract_imports(&self, file_path: &Path) -> Result<Vec<String>>;

    /// Counts code lines (excluding comments and empty lines)
    fn count_code_lines(&self, content: &str) -> usize;
}

/// Platform registry for managing multiple platforms
pub struct PlatformRegistry {
    platforms: Vec<Box<dyn Platform>>,
}

impl PlatformRegistry {
    /// Creates a new PlatformRegistry with default platforms
    pub fn new() -> Self {
        let platforms: Vec<Box<dyn Platform>> = vec![
            Box::new(android::AndroidPlatform::new()),
            Box::new(ios::IOSPlatform::new()),
        ];

        Self { platforms }
    }

    /// Gets all registered platforms
    #[allow(dead_code)]
    pub fn get_all(&self) -> &[Box<dyn Platform>] {
        &self.platforms
    }

    /// Gets a specific platform by type
    pub fn get(&self, platform_type: PlatformType) -> Option<&dyn Platform> {
        self.platforms
            .iter()
            .find(|p| p.platform_type() == platform_type)
            .map(|p| p.as_ref())
    }

    /// Finds all app files across all platforms
    pub fn find_all_app_files(&self, project_path: &Path) -> Result<HashMap<PlatformType, Vec<PathBuf>>> {
        let mut result = HashMap::new();

        for platform in &self.platforms {
            let files = platform.find_app_files(project_path)?;
            if !files.is_empty() {
                result.insert(platform.platform_type(), files);
            }
        }

        Ok(result)
    }

    /// Detects which platform a file belongs to
    #[allow(dead_code)]
    pub fn detect_platform(&self, file_path: &Path) -> Option<PlatformType> {
        for platform in &self.platforms {
            if platform.is_platform_file(file_path) {
                return Some(platform.platform_type());
            }
        }
        None
    }
}

impl Default for PlatformRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to detect usage of symbols using regex patterns
pub fn detect_usage_with_patterns(
    content: &str,
    file_path: &Path,
    kmp_symbols: &[String],
    comment_prefixes: &[&str],
) -> HashMap<String, SymbolUsage> {
    use regex::Regex;
    use std::collections::HashSet;

    let mut usages: HashMap<String, SymbolUsage> = HashMap::new();
    let lines: Vec<&str> = content.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Skip comments
        if comment_prefixes.iter().any(|prefix| trimmed.starts_with(prefix)) {
            continue;
        }

        // Check each symbol
        for symbol_name in kmp_symbols {
            // Match symbol usage in various contexts
            let pattern = format!(r"\b{}\b(?:\s*\(|\.|\s*:|<|\s+)", regex::escape(symbol_name));
            if let Ok(regex) = Regex::new(&pattern) {
                if regex.is_match(line) {
                    let usage = usages.entry(symbol_name.clone()).or_insert_with(|| {
                        SymbolUsage {
                            symbol_name: symbol_name.clone(),
                            reference_count: 0,
                            used_in_files: HashSet::new(),
                            usage_lines: Vec::new(),
                        }
                    });

                    usage.reference_count += 1;
                    usage.used_in_files.insert(file_path.to_string_lossy().to_string());
                    usage.usage_lines.push(UsageLocation {
                        file: file_path.to_string_lossy().to_string(),
                        line: line_num + 1,
                        context: trimmed.to_string(),
                    });
                }
            }
        }
    }

    usages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_registry() {
        let registry = PlatformRegistry::new();
        assert_eq!(registry.get_all().len(), 2);

        let android = registry.get(PlatformType::Android);
        assert!(android.is_some());

        let ios = registry.get(PlatformType::IOS);
        assert!(ios.is_some());
    }

    #[test]
    fn test_detect_platform() {
        let registry = PlatformRegistry::new();

        let kt_path = Path::new("app/src/main/kotlin/MainActivity.kt");
        assert_eq!(registry.detect_platform(kt_path), Some(PlatformType::Android));

        let swift_path = Path::new("iosApp/ContentView.swift");
        assert_eq!(registry.detect_platform(swift_path), Some(PlatformType::IOS));
    }
}
