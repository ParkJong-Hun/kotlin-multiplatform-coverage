use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::{detect_usage_with_patterns, Platform, PlatformType};
use crate::analyzer::models::SymbolUsage;

/// iOS platform implementation (Swift + Objective-C)
pub struct IOSPlatform {
    #[allow(dead_code)]
    import_regex: Regex,
    #[allow(dead_code)]
    kmp_framework_regex: Regex,
}

impl IOSPlatform {
    pub fn new() -> Self {
        Self {
            // Match: import Shared, import ComposeApp, etc.
            import_regex: Regex::new(r"(?m)^import\s+([A-Za-z0-9_]+)").unwrap(),
            // Detect KMP framework imports (common patterns)
            kmp_framework_regex: Regex::new(r"(?m)^import\s+(Shared|ComposeApp|[A-Z][a-zA-Z]*KMP|[A-Z][a-zA-Z]*Shared)").unwrap(),
        }
    }

    /// Checks if a line is a Swift comment
    fn is_swift_comment(line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*")
    }

    /// Checks if a line is an Objective-C comment
    fn is_objc_comment(line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*")
    }

    /// Counts code lines for Swift files
    fn count_swift_lines(content: &str) -> usize {
        content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !Self::is_swift_comment(trimmed)
            })
            .count()
    }

    /// Counts code lines for Objective-C files
    fn count_objc_lines(content: &str) -> usize {
        content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !Self::is_objc_comment(trimmed)
            })
            .count()
    }

    /// Finds Swift files in a directory
    fn find_swift_files(root: &Path) -> Vec<PathBuf> {
        WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == "swift")
                    .unwrap_or(false)
            })
            .map(|e| e.path().to_path_buf())
            .collect()
    }

    /// Finds Objective-C files in a directory
    fn find_objc_files(root: &Path) -> Vec<PathBuf> {
        WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == "m" || ext == "mm" || ext == "h")
                    .unwrap_or(false)
            })
            .map(|e| e.path().to_path_buf())
            .collect()
    }

    /// Detects if a Swift file imports KMP framework
    #[allow(dead_code)]
    pub fn has_kmp_import(&self, file_path: &Path) -> Result<bool> {
        let content = fs::read_to_string(file_path)?;
        Ok(self.kmp_framework_regex.is_match(&content))
    }
}

impl Default for IOSPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl Platform for IOSPlatform {
    fn platform_type(&self) -> PlatformType {
        PlatformType::IOS
    }

    fn file_extensions(&self) -> Vec<&str> {
        vec!["swift", "m", "mm", "h"]
    }

    fn app_directory_patterns(&self) -> Vec<&str> {
        vec![
            "iosApp",
            "iosApp/iosApp",
            "ios",
            "iOS",
            "composeApp/src/iosMain",
        ]
    }

    fn find_app_files(&self, project_path: &Path) -> Result<Vec<PathBuf>> {
        let mut app_files = Vec::new();

        for pattern in self.app_directory_patterns() {
            let search_path = project_path.join(pattern);
            if search_path.exists() {
                // Find Swift files
                let swift_files = Self::find_swift_files(&search_path);
                app_files.extend(swift_files);

                // Find Objective-C files
                let objc_files = Self::find_objc_files(&search_path);
                app_files.extend(objc_files);
            }
        }

        Ok(app_files)
    }

    fn detect_symbol_usage(
        &self,
        file_path: &Path,
        kmp_symbols: &[String],
    ) -> Result<HashMap<String, SymbolUsage>> {
        let content = fs::read_to_string(file_path)?;

        // Swift and Objective-C use similar comment syntax
        let comment_prefixes = vec!["//", "/*", "*", "import ", "#import"];
        Ok(detect_usage_with_patterns(
            &content,
            file_path,
            kmp_symbols,
            &comment_prefixes,
        ))
    }

    fn extract_imports(&self, file_path: &Path) -> Result<Vec<String>> {
        let content = fs::read_to_string(file_path)?;
        let mut imports = Vec::new();

        for cap in self.import_regex.captures_iter(&content) {
            if let Some(import) = cap.get(1) {
                imports.push(import.as_str().to_string());
            }
        }

        // Also check for Objective-C style imports
        let objc_import_regex = Regex::new(r#"(?m)^#import\s+[<"]([A-Za-z0-9_/]+)[>"]"#).unwrap();
        for cap in objc_import_regex.captures_iter(&content) {
            if let Some(import) = cap.get(1) {
                imports.push(import.as_str().to_string());
            }
        }

        Ok(imports)
    }

    fn count_code_lines(&self, content: &str) -> usize {
        // Detect if Swift or Objective-C by file patterns
        if content.contains("func ") || content.contains("let ") || content.contains("var ") {
            Self::count_swift_lines(content)
        } else {
            Self::count_objc_lines(content)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_file_extensions() {
        let platform = IOSPlatform::new();
        let extensions = platform.file_extensions();
        assert!(extensions.contains(&"swift"));
        assert!(extensions.contains(&"m"));
    }

    #[test]
    fn test_detect_swift_usage() {
        let platform = IOSPlatform::new();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "let repo = UserRepository()").unwrap();

        let symbols = vec!["UserRepository".to_string()];
        let usages = platform.detect_symbol_usage(file.path(), &symbols).unwrap();

        assert_eq!(usages.len(), 1);
        assert!(usages.contains_key("UserRepository"));
    }

    #[test]
    fn test_has_kmp_import() {
        let platform = IOSPlatform::new();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "import Shared").unwrap();
        writeln!(file, "import SwiftUI").unwrap();

        let has_import = platform.has_kmp_import(file.path()).unwrap();
        assert!(has_import);
    }

    #[test]
    fn test_extract_swift_imports() {
        let platform = IOSPlatform::new();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "import Shared").unwrap();
        writeln!(file, "import SwiftUI").unwrap();

        let imports = platform.extract_imports(file.path()).unwrap();
        assert_eq!(imports.len(), 2);
        assert!(imports.contains(&"Shared".to_string()));
        assert!(imports.contains(&"SwiftUI".to_string()));
    }

    #[test]
    fn test_count_swift_lines() {
        let platform = IOSPlatform::new();
        let content = "func main() {\n    // comment\n    print(\"hello\")\n}\n";
        let lines = platform.count_code_lines(content);
        assert_eq!(lines, 3); // Excludes comment
    }
}
