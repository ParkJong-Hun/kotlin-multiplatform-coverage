use crate::domain::{Language, Platform};

/// Platform detection utilities
pub struct PlatformDetector;

impl PlatformDetector {
    /// Detect platform from file path
    pub fn detect_platform_from_path(file_path: &str) -> Platform {
        if file_path.contains("android")
            || file_path.contains("androidApp")
            || file_path.ends_with(".kt")
            || file_path.ends_with(".java")
        {
            Platform::Android
        } else if file_path.contains("ios")
            || file_path.contains("iosApp")
            || file_path.ends_with(".swift")
            || file_path.ends_with(".m")
            || file_path.ends_with(".mm")
        {
            Platform::IOS
        } else {
            // Default to Android for ambiguous cases
            Platform::Android
        }
    }

    /// Detect language from file extension
    pub fn detect_language(file_path: &str) -> Language {
        if file_path.ends_with(".kt") || file_path.ends_with(".kts") {
            Language::Kotlin
        } else if file_path.ends_with(".java") {
            Language::Java
        } else if file_path.ends_with(".swift") {
            Language::Swift
        } else if file_path.ends_with(".m") || file_path.ends_with(".mm") || file_path.ends_with(".h") {
            Language::ObjectiveC
        } else {
            Language::Kotlin // default
        }
    }
}
