use anyhow::Result;

use crate::analyzer::symbol_extractor::SymbolExtractor;
use crate::domain::{Symbol, SymbolRepository, SymbolType};

/// Adapter implementation of SymbolRepository
/// Uses the existing SymbolExtractor from analyzer layer
pub struct SymbolRepositoryImpl {
    extractor: SymbolExtractor,
}

impl SymbolRepositoryImpl {
    pub fn new() -> Self {
        Self {
            extractor: SymbolExtractor::new(),
        }
    }

    fn determine_module_name(file_path: &str) -> String {
        if let Some(idx) = file_path.find("/src/") {
            let before_src = &file_path[..idx];
            if let Some(last_slash) = before_src.rfind('/') {
                return before_src[last_slash + 1..].to_string();
            }
            return before_src.to_string();
        }
        "unknown".to_string()
    }

    fn convert_symbol_type(old_type: &crate::analyzer::models::SymbolType) -> SymbolType {
        match old_type {
            crate::analyzer::models::SymbolType::Class => SymbolType::Class,
            crate::analyzer::models::SymbolType::Interface => SymbolType::Interface,
            crate::analyzer::models::SymbolType::Object => SymbolType::Object,
            crate::analyzer::models::SymbolType::Function => SymbolType::Function,
            crate::analyzer::models::SymbolType::Property => SymbolType::Property,
            crate::analyzer::models::SymbolType::TypeAlias => SymbolType::TypeAlias,
        }
    }
}

impl Default for SymbolRepositoryImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolRepository for SymbolRepositoryImpl {
    fn extract_kmp_symbols(&self, kmp_file_paths: &[String]) -> Result<Vec<Symbol>> {
        let mut symbols = Vec::new();

        for file_path in kmp_file_paths {
            let module = Self::determine_module_name(file_path);
            let path = std::path::Path::new(file_path);

            let extracted = self.extractor.extract_symbols(path, &module)?;

            for old_symbol in extracted {
                symbols.push(Symbol {
                    name: old_symbol.name,
                    symbol_type: Self::convert_symbol_type(&old_symbol.symbol_type),
                    module: old_symbol.module,
                    file_path: old_symbol.file_path,
                    is_public: old_symbol.is_public,
                });
            }
        }

        Ok(symbols)
    }
}
