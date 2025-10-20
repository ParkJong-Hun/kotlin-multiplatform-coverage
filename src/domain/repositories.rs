use anyhow::Result;
use std::collections::HashMap;

use super::entities::{Platform, SourceFile, Symbol, SymbolUsage};

/// Repository interface for symbol extraction
/// Implemented by adapters layer
pub trait SymbolRepository: Send + Sync {
    /// Extract all public symbols from KMP source files
    fn extract_kmp_symbols(&self, kmp_file_paths: &[String]) -> Result<Vec<Symbol>>;
}

/// Repository interface for source file operations
/// Implemented by adapters layer
pub trait SourceFileRepository: Send + Sync {
    /// Find all KMP source files in the project
    fn find_kmp_files(&self, project_path: &str) -> Result<Vec<String>>;

    /// Find all app source files grouped by platform
    fn find_app_files(&self, project_path: &str) -> Result<HashMap<Platform, Vec<String>>>;

    /// Read and parse a source file
    fn read_source_file(&self, file_path: &str) -> Result<SourceFile>;

    /// Count code lines in content (excluding comments/empty lines)
    fn count_code_lines(&self, content: &str, platform: Platform) -> usize;
}

/// Repository interface for symbol usage detection
/// Implemented by adapters layer
pub trait SymbolUsageRepository: Send + Sync {
    /// Detect where symbols are used in a source file
    fn detect_symbol_usage(
        &self,
        source_file: &SourceFile,
        symbols: &[Symbol],
    ) -> Result<Vec<SymbolUsage>>;
}

/// Repository interface for dependency analysis
/// Implemented by adapters layer
pub trait DependencyRepository: Send + Sync {
    /// Build dependency graph from source files
    fn build_dependency_graph(&self, file_paths: &[String]) -> Result<()>;

    /// Calculate transitive dependencies for given files
    fn calculate_transitive_dependencies(&self, direct_files: &[String]) -> Result<Vec<String>>;

    /// Extract imports from a source file
    #[allow(dead_code)]
    fn extract_imports(&self, source_file: &SourceFile) -> Result<Vec<String>>;
}
