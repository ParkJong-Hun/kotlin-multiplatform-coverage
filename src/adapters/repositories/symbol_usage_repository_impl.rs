use anyhow::Result;

use crate::domain::{SourceFile, Symbol, SymbolUsage, SymbolUsageRepository};
use crate::adapters::platforms::detect_usage_with_patterns;

/// Adapter implementation of SymbolUsageRepository
pub struct SymbolUsageRepositoryImpl;

impl SymbolUsageRepositoryImpl {
    pub fn new() -> Self {
        Self
    }

    fn get_comment_prefixes(source_file: &SourceFile) -> Vec<&'static str> {
        match source_file.language {
            crate::domain::Language::Kotlin | crate::domain::Language::Java => {
                vec!["//", "/*", "*", "import "]
            }
            crate::domain::Language::Swift | crate::domain::Language::ObjectiveC => {
                vec!["//", "/*", "*", "import ", "#import"]
            }
        }
    }
}

impl Default for SymbolUsageRepositoryImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolUsageRepository for SymbolUsageRepositoryImpl {
    fn detect_symbol_usage(
        &self,
        source_file: &SourceFile,
        symbols: &[Symbol],
    ) -> Result<Vec<SymbolUsage>> {
        let symbol_names: Vec<String> = symbols.iter().map(|s| s.name.clone()).collect();
        let comment_prefixes = Self::get_comment_prefixes(source_file);

        let path = std::path::Path::new(&source_file.path);
        let usages_map = detect_usage_with_patterns(
            &source_file.content,
            path,
            &symbol_names,
            &comment_prefixes,
        );

        let mut usages = Vec::new();
        for (symbol_name, symbol_usage) in usages_map {
            for usage_location in symbol_usage.usage_lines {
                usages.push(SymbolUsage {
                    symbol_name: symbol_name.clone(),
                    file_path: usage_location.file,
                    line_number: usage_location.line,
                    context: usage_location.context,
                });
            }
        }

        Ok(usages)
    }
}
