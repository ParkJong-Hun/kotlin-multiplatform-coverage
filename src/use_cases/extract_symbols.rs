use anyhow::Result;
use log::info;

use crate::domain::{Symbol, SymbolRepository};

/// Use Case: Extract KMP Symbols
///
/// Responsibility: Extract all public symbols from KMP source files
pub struct ExtractSymbolsUseCase<'a> {
    symbol_repository: &'a dyn SymbolRepository,
}

impl<'a> ExtractSymbolsUseCase<'a> {
    pub fn new(symbol_repository: &'a dyn SymbolRepository) -> Self {
        Self { symbol_repository }
    }

    /// Execute the use case
    pub fn execute(&self, kmp_file_paths: &[String]) -> Result<Vec<Symbol>> {
        info!("Extracting symbols from {} KMP files", kmp_file_paths.len());

        let symbols = self.symbol_repository.extract_kmp_symbols(kmp_file_paths)?;

        info!("Extracted {} symbols", symbols.len());
        Ok(symbols)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{SymbolType};

    struct MockSymbolRepository;

    impl SymbolRepository for MockSymbolRepository {
        fn extract_kmp_symbols(&self, _paths: &[String]) -> Result<Vec<Symbol>> {
            Ok(vec![
                Symbol {
                    name: "UserRepository".to_string(),
                    symbol_type: SymbolType::Class,
                    module: "shared".to_string(),
                    file_path: "shared/src/User.kt".to_string(),
                    is_public: true,
                }
            ])
        }
    }

    #[test]
    fn test_extract_symbols() {
        let repo = MockSymbolRepository;
        let use_case = ExtractSymbolsUseCase::new(&repo);

        let symbols = use_case.execute(&vec!["test.kt".to_string()]).unwrap();
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "UserRepository");
    }
}
