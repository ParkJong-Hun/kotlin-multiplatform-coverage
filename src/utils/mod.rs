use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// File system utility functions
pub struct FileUtils;

impl FileUtils {
    /// Finds files matching a specific pattern in a directory
    pub fn find_files(root: &Path, pattern: &str) -> Vec<PathBuf> {
        WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.contains(pattern))
                    .unwrap_or(false)
            })
            .map(|e| e.path().to_path_buf())
            .collect()
    }

    /// Finds Kotlin source files
    pub fn find_kotlin_files(root: &Path) -> Vec<PathBuf> {
        WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == "kt" || ext == "kts")
                    .unwrap_or(false)
            })
            .map(|e| e.path().to_path_buf())
            .collect()
    }

    /// Finds Gradle build files
    #[allow(dead_code)]
    pub fn find_gradle_files(root: &Path) -> Vec<PathBuf> {
        WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n == "build.gradle" || n == "build.gradle.kts")
                    .unwrap_or(false)
            })
            .map(|e| e.path().to_path_buf())
            .collect()
    }
}

/// Git utility functions
#[allow(dead_code)]
pub struct GitUtils;

#[allow(dead_code)]
impl GitUtils {
    /// Checks if a git repository exists
    pub fn is_git_repo(path: &Path) -> bool {
        git2::Repository::open(path).is_ok()
    }

    /// Gets the current branch name
    pub fn get_current_branch(path: &Path) -> Option<String> {
        let repo = git2::Repository::open(path).ok()?;
        let head = repo.head().ok()?;
        head.shorthand().map(|s| s.to_string())
    }
}
