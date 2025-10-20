/// Use Cases layer - Application business rules
/// Orchestrates domain entities and repository interfaces

pub mod analyze_impact;
pub mod extract_symbols;
pub mod detect_usage;
pub mod calculate_dependencies;

pub use analyze_impact::AnalyzeImpactUseCase;
pub use extract_symbols::ExtractSymbolsUseCase;
pub use detect_usage::DetectUsageUseCase;
pub use calculate_dependencies::CalculateDependenciesUseCase;
