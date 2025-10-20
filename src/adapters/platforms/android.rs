use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::{detect_usage_with_patterns, Platform, PlatformType};
use crate::analyzer::models::SymbolUsage;
use crate::utils::FileUtils;

/// Android platform implementation (Kotlin + Java)
pub struct AndroidPlatform {
    #[allow(dead_code)]
    package_regex: Regex,
    #[allow(dead_code)]
    import_regex: Regex,
}

impl AndroidPlatform {
    pub fn new() -> Self {
        Self {
            package_regex: Regex::new(r"(?m)^package\s+([a-zA-Z0-9_.]+)").unwrap(),
            import_regex: Regex::new(r"(?m)^import\s+([a-zA-Z0-9_.]+)").unwrap(),
        }
    }

    /// Checks if a line is a Kotlin comment
    fn is_kotlin_comment(line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*")
    }

    /// Checks if a line is a Java comment
    fn is_java_comment(line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*")
    }

    /// Counts code lines for Kotlin files
    fn count_kotlin_lines(content: &str) -> usize {
        content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !Self::is_kotlin_comment(trimmed)
            })
            .count()
    }

    /// Counts code lines for Java files
    fn count_java_lines(content: &str) -> usize {
        content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !Self::is_java_comment(trimmed)
            })
            .count()
    }
}

impl Default for AndroidPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl Platform for AndroidPlatform {
    fn platform_type(&self) -> PlatformType {
        PlatformType::Android
    }

    fn file_extensions(&self) -> Vec<&str> {
        vec!["kt", "kts", "java"]
    }

    fn app_directory_patterns(&self) -> Vec<&str> {
        vec![
            "app/src",
            "android/src",
            "androidApp/src",
            "composeApp/src/androidMain",
        ]
    }

    fn find_app_files(&self, project_path: &Path) -> Result<Vec<PathBuf>> {
        let mut app_files = Vec::new();

        for pattern in self.app_directory_patterns() {
            let search_path = project_path.join(pattern);
            if search_path.exists() {
                // Find Kotlin files
                let kt_files = FileUtils::find_kotlin_files(&search_path);
                app_files.extend(kt_files);

                // Find Java files
                let java_files = FileUtils::find_files(&search_path, ".java");
                app_files.extend(java_files);
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

        // Use common detection logic for both Kotlin and Java
        let comment_prefixes = vec!["//", "/*", "*", "import "];
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

        Ok(imports)
    }

    fn count_code_lines(&self, content: &str) -> usize {
        // Try to determine if it's Java or Kotlin by simple heuristics
        if content.contains("fun ") || content.contains("val ") || content.contains("var ") {
            Self::count_kotlin_lines(content)
        } else {
            Self::count_java_lines(content)
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
        let platform = AndroidPlatform::new();
        let extensions = platform.file_extensions();
        assert!(extensions.contains(&"kt"));
        assert!(extensions.contains(&"java"));
    }

    #[test]
    fn test_detect_kotlin_usage() {
        let platform = AndroidPlatform::new();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "val repo = UserRepository()").unwrap();

        let symbols = vec!["UserRepository".to_string()];
        let usages = platform.detect_symbol_usage(file.path(), &symbols).unwrap();

        assert_eq!(usages.len(), 1);
        assert!(usages.contains_key("UserRepository"));
    }

    #[test]
    fn test_detect_java_usage() {
        let platform = AndroidPlatform::new();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "UserRepository repo = new UserRepository();").unwrap();

        let symbols = vec!["UserRepository".to_string()];
        let usages = platform.detect_symbol_usage(file.path(), &symbols).unwrap();

        assert_eq!(usages.len(), 1);
        assert!(usages.contains_key("UserRepository"));
    }

    #[test]
    fn test_extract_imports() {
        let platform = AndroidPlatform::new();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "import com.example.UserRepository").unwrap();
        writeln!(file, "import com.example.User").unwrap();

        let imports = platform.extract_imports(file.path()).unwrap();
        assert_eq!(imports.len(), 2);
        assert!(imports.contains(&"com.example.UserRepository".to_string()));
    }

    #[test]
    fn test_count_kotlin_lines() {
        let platform = AndroidPlatform::new();
        let content = "fun main() {\n    // comment\n    println(\"hello\")\n}\n";
        let lines = platform.count_code_lines(content);
        assert_eq!(lines, 3); // Excludes comment
    }
}
