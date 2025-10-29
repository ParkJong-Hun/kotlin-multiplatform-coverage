/// Kotlin Multiplatform Coverage - Library Interface
///
/// This module exposes the library's public API for use in tests and as a library.

// Re-export public modules for library usage
pub mod domain;
pub mod use_cases;
pub mod adapters;
pub mod infrastructure;
pub mod utils;
pub mod analyzer;

// Re-export commonly used types for convenience
pub use domain::{
    DependencyRepository, ImpactAnalysis, Platform, PlatformImpact,
    SourceFile, SourceFileRepository, Symbol, SymbolRepository,
    SymbolType, SymbolUsage, SymbolUsageRepository,
};
