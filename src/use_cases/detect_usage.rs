use anyhow::Result;
use log::info;
use std::collections::HashMap;

use crate::domain::{Platform, SourceFileRepository, Symbol, SymbolUsage, SymbolUsageRepository};

/// Use Case: Detect Symbol Usage
///
/// Responsibility: Find where KMP symbols are used across all platforms
pub struct DetectUsageUseCase<'a> {
    source_file_repository: &'a dyn SourceFileRepository,
    symbol_usage_repository: &'a dyn SymbolUsageRepository,
}

impl<'a> DetectUsageUseCase<'a> {
    pub fn new(
        source_file_repository: &'a dyn SourceFileRepository,
        symbol_usage_repository: &'a dyn SymbolUsageRepository,
    ) -> Self {
        Self {
            source_file_repository,
            symbol_usage_repository,
        }
    }

    /// Execute the use case
    pub fn execute(
        &self,
        app_files_by_platform: &HashMap<Platform, Vec<String>>,
        symbols: &[Symbol],
    ) -> Result<HashMap<String, Vec<SymbolUsage>>> {
        info!("Detecting symbol usage across platforms");

        let mut all_usages: HashMap<String, Vec<SymbolUsage>> = HashMap::new();

        for (platform, file_paths) in app_files_by_platform {
            info!("Analyzing {} {} files", file_paths.len(), platform.name());

            for file_path in file_paths {
                // Read source file
                let source_file = self.source_file_repository.read_source_file(file_path)?;

                // Detect symbol usage
                let usages = self.symbol_usage_repository.detect_symbol_usage(&source_file, symbols)?;

                // Aggregate usages by symbol name
                for usage in usages {
                    all_usages
                        .entry(usage.symbol_name.clone())
                        .or_insert_with(Vec::new)
                        .push(usage);
                }
            }
        }

        let total_usages: usize = all_usages.values().map(|v| v.len()).sum();
        info!("Found {} total symbol usages", total_usages);

        Ok(all_usages)
    }

    /// Get files that directly use symbols
    pub fn get_affected_files(
        &self,
        symbol_usages: &HashMap<String, Vec<SymbolUsage>>,
    ) -> Vec<String> {
        let mut affected_files = std::collections::HashSet::new();

        for usages in symbol_usages.values() {
            for usage in usages {
                affected_files.insert(usage.file_path.clone());
            }
        }

        affected_files.into_iter().collect()
    }
}
