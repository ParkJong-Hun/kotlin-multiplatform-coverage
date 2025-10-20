use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Analysis result structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisResult {
    /// Total number of projects
    pub total_projects: usize,
    /// Number of projects using KMP
    pub kmp_projects: usize,
    /// Coverage information per module
    pub modules: Vec<ModuleCoverage>,
    /// Overall coverage ratio (0.0 ~ 1.0)
    pub overall_coverage: f64,
    /// Impact coverage analysis
    pub impact_coverage: ImpactCoverage,
}

/// Impact coverage analysis result
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImpactCoverage {
    /// KMP symbols (classes, functions, etc.)
    pub kmp_symbols: Vec<KmpSymbol>,
    /// Direct impact: files that directly use KMP symbols
    pub direct_impact_files: HashSet<String>,
    /// Transitive impact: files that depend on KMP-using files
    pub transitive_impact_files: HashSet<String>,
    /// Total lines affected by KMP (direct + transitive)
    pub affected_lines: usize,
    /// Total lines in app code
    pub total_app_lines: usize,
    /// Impact coverage ratio (0.0 ~ 1.0)
    pub impact_ratio: f64,
    /// Symbol usage statistics
    pub symbol_usage: HashMap<String, SymbolUsage>,
    /// Per-platform impact breakdown
    pub platform_impact: HashMap<String, PlatformImpact>,
}

/// Platform-specific impact statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlatformImpact {
    /// Platform name (Android, iOS, etc.)
    pub platform_name: String,
    /// Number of app files for this platform
    pub total_files: usize,
    /// Files directly impacted by KMP
    pub direct_impact_files: HashSet<String>,
    /// Files transitively impacted by KMP
    pub transitive_impact_files: usize,
    /// Total lines in platform app code
    pub total_lines: usize,
    /// Lines affected by KMP
    pub affected_lines: usize,
    /// Impact ratio for this platform
    pub impact_ratio: f64,
    /// Most used symbols on this platform
    pub top_symbols: Vec<(String, usize)>,
}

/// KMP symbol information (class, function, property, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KmpSymbol {
    /// Symbol name
    pub name: String,
    /// Symbol type (class, function, property, etc.)
    pub symbol_type: SymbolType,
    /// Module where the symbol is defined
    pub module: String,
    /// File path where the symbol is defined
    pub file_path: String,
    /// Whether the symbol is public
    pub is_public: bool,
}

/// Symbol type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SymbolType {
    Class,
    Interface,
    Object,
    Function,
    Property,
    TypeAlias,
}

/// Symbol usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SymbolUsage {
    /// Symbol name
    pub symbol_name: String,
    /// Number of times the symbol is referenced
    pub reference_count: usize,
    /// Files that use this symbol
    pub used_in_files: HashSet<String>,
    /// Lines where the symbol is used
    pub usage_lines: Vec<UsageLocation>,
}

/// Usage location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageLocation {
    /// File path
    pub file: String,
    /// Line number
    pub line: usize,
    /// Context (surrounding code)
    pub context: String,
}

/// Module coverage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleCoverage {
    /// Module name
    pub name: String,
    /// Module path
    pub path: String,
    /// Whether this is a KMP module
    pub is_kmp: bool,
    /// Number of KMP code lines
    pub kmp_lines: usize,
    /// Total number of code lines
    pub total_lines: usize,
    /// Other modules this module depends on
    pub dependencies: Vec<String>,
    /// Other modules that depend on this module
    pub dependents: Vec<String>,
    /// Coverage ratio (0.0 ~ 1.0)
    pub coverage: f64,
}

impl ModuleCoverage {
    /// Creates a new ModuleCoverage instance
    #[allow(dead_code)]
    pub fn new(name: String, path: String) -> Self {
        Self {
            name,
            path,
            is_kmp: false,
            kmp_lines: 0,
            total_lines: 0,
            dependencies: Vec::new(),
            dependents: Vec::new(),
            coverage: 0.0,
        }
    }

    /// Calculates the coverage ratio
    #[allow(dead_code)]
    pub fn calculate_coverage(&mut self) {
        if self.total_lines > 0 {
            self.coverage = self.kmp_lines as f64 / self.total_lines as f64;
        }
    }
}
