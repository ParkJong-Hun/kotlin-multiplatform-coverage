use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Core domain entity: KMP Symbol
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub module: String,
    pub file_path: String,
    pub is_public: bool,
}

/// Symbol type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SymbolType {
    Class,
    Interface,
    Object,
    Function,
    Property,
    TypeAlias,
}

/// Platform enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Platform {
    Android,
    IOS,
}

impl Platform {
    pub fn name(&self) -> &str {
        match self {
            Platform::Android => "Android",
            Platform::IOS => "iOS",
        }
    }
}

/// Source file entity
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: String,
    #[allow(dead_code)]
    pub platform: Platform,
    pub language: Language,
    pub content: String,
}

/// Programming language
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Language {
    Kotlin,
    Java,
    Swift,
    ObjectiveC,
}

/// Symbol usage in a specific location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolUsage {
    pub symbol_name: String,
    pub file_path: String,
    pub line_number: usize,
    pub context: String,
}

/// Impact analysis result - aggregated domain entity
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImpactAnalysis {
    pub total_symbols: usize,
    pub total_app_files: usize,
    pub total_app_lines: usize,
    pub affected_files: HashSet<String>,
    pub affected_lines: usize,
    pub impact_ratio: f64,
    pub platform_impacts: HashMap<String, PlatformImpact>,
    pub symbol_usages: HashMap<String, Vec<SymbolUsage>>,
}

/// Platform-specific impact
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlatformImpact {
    pub platform_name: String,
    pub total_files: usize,
    pub total_lines: usize,
    pub affected_files: HashSet<String>,
    pub affected_lines: usize,
    pub impact_ratio: f64,
    pub top_symbols: Vec<(String, usize)>,
}

impl ImpactAnalysis {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn calculate_impact_ratio(&mut self) {
        if self.total_app_lines > 0 {
            self.impact_ratio = self.affected_lines as f64 / self.total_app_lines as f64;
        }
    }
}

impl PlatformImpact {
    pub fn new(platform_name: String) -> Self {
        Self {
            platform_name,
            ..Default::default()
        }
    }

    pub fn calculate_impact_ratio(&mut self) {
        if self.total_lines > 0 {
            self.impact_ratio = self.affected_lines as f64 / self.total_lines as f64;
        }
    }
}
