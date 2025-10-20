use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::Path;

use super::models::{KmpSymbol, SymbolType};

/// Extracts public symbols from KMP source code
pub struct SymbolExtractor {
    class_regex: Regex,
    interface_regex: Regex,
    object_regex: Regex,
    function_regex: Regex,
    property_regex: Regex,
    typealias_regex: Regex,
}

impl SymbolExtractor {
    /// Creates a new SymbolExtractor instance
    pub fn new() -> Self {
        Self {
            // Match: public class ClassName, class ClassName (public by default in Kotlin)
            class_regex: Regex::new(r"(?m)^\s*(?:public\s+)?class\s+([A-Z][a-zA-Z0-9_]*)").unwrap(),
            // Match: public interface InterfaceName
            interface_regex: Regex::new(r"(?m)^\s*(?:public\s+)?interface\s+([A-Z][a-zA-Z0-9_]*)").unwrap(),
            // Match: public object ObjectName
            object_regex: Regex::new(r"(?m)^\s*(?:public\s+)?object\s+([A-Z][a-zA-Z0-9_]*)").unwrap(),
            // Match: public fun functionName, fun functionName
            function_regex: Regex::new(r"(?m)^\s*(?:public\s+)?fun\s+([a-z][a-zA-Z0-9_]*)\s*\(").unwrap(),
            // Match: public val/var propertyName
            property_regex: Regex::new(r"(?m)^\s*(?:public\s+)?(?:val|var)\s+([a-z][a-zA-Z0-9_]*)\s*[:=]").unwrap(),
            // Match: public typealias AliasName
            typealias_regex: Regex::new(r"(?m)^\s*(?:public\s+)?typealias\s+([A-Z][a-zA-Z0-9_]*)").unwrap(),
        }
    }

    /// Extracts all public symbols from a Kotlin file
    pub fn extract_symbols(&self, file_path: &Path, module: &str) -> Result<Vec<KmpSymbol>> {
        let content = fs::read_to_string(file_path)?;
        let mut symbols = Vec::new();

        // Skip if file is private or internal
        if self.is_private_file(&content) {
            return Ok(symbols);
        }

        // Extract classes
        for cap in self.class_regex.captures_iter(&content) {
            if let Some(name) = cap.get(1) {
                symbols.push(KmpSymbol {
                    name: name.as_str().to_string(),
                    symbol_type: SymbolType::Class,
                    module: module.to_string(),
                    file_path: file_path.to_string_lossy().to_string(),
                    is_public: true,
                });
            }
        }

        // Extract interfaces
        for cap in self.interface_regex.captures_iter(&content) {
            if let Some(name) = cap.get(1) {
                symbols.push(KmpSymbol {
                    name: name.as_str().to_string(),
                    symbol_type: SymbolType::Interface,
                    module: module.to_string(),
                    file_path: file_path.to_string_lossy().to_string(),
                    is_public: true,
                });
            }
        }

        // Extract objects
        for cap in self.object_regex.captures_iter(&content) {
            if let Some(name) = cap.get(1) {
                symbols.push(KmpSymbol {
                    name: name.as_str().to_string(),
                    symbol_type: SymbolType::Object,
                    module: module.to_string(),
                    file_path: file_path.to_string_lossy().to_string(),
                    is_public: true,
                });
            }
        }

        // Extract functions
        for cap in self.function_regex.captures_iter(&content) {
            if let Some(name) = cap.get(1) {
                symbols.push(KmpSymbol {
                    name: name.as_str().to_string(),
                    symbol_type: SymbolType::Function,
                    module: module.to_string(),
                    file_path: file_path.to_string_lossy().to_string(),
                    is_public: true,
                });
            }
        }

        // Extract properties
        for cap in self.property_regex.captures_iter(&content) {
            if let Some(name) = cap.get(1) {
                symbols.push(KmpSymbol {
                    name: name.as_str().to_string(),
                    symbol_type: SymbolType::Property,
                    module: module.to_string(),
                    file_path: file_path.to_string_lossy().to_string(),
                    is_public: true,
                });
            }
        }

        // Extract type aliases
        for cap in self.typealias_regex.captures_iter(&content) {
            if let Some(name) = cap.get(1) {
                symbols.push(KmpSymbol {
                    name: name.as_str().to_string(),
                    symbol_type: SymbolType::TypeAlias,
                    module: module.to_string(),
                    file_path: file_path.to_string_lossy().to_string(),
                    is_public: true,
                });
            }
        }

        Ok(symbols)
    }

    /// Checks if the file contains private or internal markers
    fn is_private_file(&self, content: &str) -> bool {
        // Simple heuristic: if file starts with private/internal package
        content.contains("internal ") || content.starts_with("private ")
    }
}

impl Default for SymbolExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_extract_class() {
        let extractor = SymbolExtractor::new();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "class UserRepository {{}}").unwrap();

        let symbols = extractor.extract_symbols(file.path(), "test").unwrap();
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "UserRepository");
        assert_eq!(symbols[0].symbol_type, SymbolType::Class);
    }

    #[test]
    fn test_extract_function() {
        let extractor = SymbolExtractor::new();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "fun getUserData(): User {{}}").unwrap();

        let symbols = extractor.extract_symbols(file.path(), "test").unwrap();
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "getUserData");
        assert_eq!(symbols[0].symbol_type, SymbolType::Function);
    }
}
