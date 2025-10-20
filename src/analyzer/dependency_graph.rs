use anyhow::Result;
use regex::Regex;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

/// Builds and analyzes dependency graph between files
pub struct DependencyGraph {
    /// Maps file path to its dependencies (files it imports/uses)
    dependencies: HashMap<String, HashSet<String>>,
    /// Maps file path to files that depend on it
    reverse_dependencies: HashMap<String, HashSet<String>>,
    /// Package to file mapping (for resolving imports)
    package_map: HashMap<String, String>,
}

impl DependencyGraph {
    /// Creates a new empty DependencyGraph
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            reverse_dependencies: HashMap::new(),
            package_map: HashMap::new(),
        }
    }

    /// Builds the dependency graph from the given files
    pub fn build(&mut self, files: &[PathBuf]) -> Result<()> {
        // First pass: build package map
        for file in files {
            if let Ok(package_name) = self.extract_package_name(file) {
                if let Some(class_name) = self.extract_primary_class_name(file) {
                    let full_name = format!("{}.{}", package_name, class_name);
                    self.package_map.insert(full_name, file.to_string_lossy().to_string());
                }
            }
        }

        // Second pass: build dependency graph
        for file in files {
            let file_path = file.to_string_lossy().to_string();
            let imports = self.extract_imports(file)?;

            let mut deps = HashSet::new();
            for import in imports {
                // Try to resolve import to file path
                if let Some(dep_file) = self.resolve_import(&import) {
                    deps.insert(dep_file.clone());

                    // Update reverse dependencies
                    self.reverse_dependencies
                        .entry(dep_file)
                        .or_insert_with(HashSet::new)
                        .insert(file_path.clone());
                }
            }

            self.dependencies.insert(file_path, deps);
        }

        Ok(())
    }

    /// Extracts package name from a Kotlin file
    fn extract_package_name(&self, file: &Path) -> Result<String> {
        let content = fs::read_to_string(file)?;
        let package_regex = Regex::new(r"(?m)^package\s+([a-zA-Z0-9_.]+)").unwrap();

        if let Some(cap) = package_regex.captures(&content) {
            if let Some(package) = cap.get(1) {
                return Ok(package.as_str().to_string());
            }
        }

        Ok(String::new())
    }

    /// Extracts primary class/interface/object name from a Kotlin file
    fn extract_primary_class_name(&self, file: &Path) -> Option<String> {
        let content = fs::read_to_string(file).ok()?;
        let class_regex = Regex::new(r"(?m)^\s*(?:public\s+)?(?:class|interface|object)\s+([A-Z][a-zA-Z0-9_]*)").unwrap();

        if let Some(cap) = class_regex.captures(&content) {
            if let Some(name) = cap.get(1) {
                return Some(name.as_str().to_string());
            }
        }

        None
    }

    /// Extracts import statements from a Kotlin file
    fn extract_imports(&self, file: &Path) -> Result<Vec<String>> {
        let content = fs::read_to_string(file)?;
        let import_regex = Regex::new(r"(?m)^import\s+([a-zA-Z0-9_.]+)").unwrap();

        let mut imports = Vec::new();
        for cap in import_regex.captures_iter(&content) {
            if let Some(import) = cap.get(1) {
                imports.push(import.as_str().to_string());
            }
        }

        Ok(imports)
    }

    /// Resolves an import statement to a file path
    fn resolve_import(&self, import: &str) -> Option<String> {
        // Try exact match first
        if let Some(file) = self.package_map.get(import) {
            return Some(file.clone());
        }

        // Try wildcard imports
        for (package, file) in &self.package_map {
            if package.starts_with(import) {
                return Some(file.clone());
            }
        }

        None
    }

    /// Computes transitive dependencies (all files that transitively depend on the given files)
    pub fn compute_transitive_impact(&self, direct_impact_files: &HashSet<String>) -> HashSet<String> {
        let mut transitive = HashSet::new();
        let mut queue = VecDeque::new();

        // Start with direct impact files
        for file in direct_impact_files {
            queue.push_back(file.clone());
        }

        // BFS to find all files that depend on these files
        while let Some(file) = queue.pop_front() {
            if transitive.contains(&file) {
                continue;
            }

            transitive.insert(file.clone());

            // Add all files that depend on this file
            if let Some(dependents) = self.reverse_dependencies.get(&file) {
                for dependent in dependents {
                    if !transitive.contains(dependent) {
                        queue.push_back(dependent.clone());
                    }
                }
            }
        }

        // Remove direct impact files from transitive (we want only indirect impact)
        for file in direct_impact_files {
            transitive.remove(file);
        }

        transitive
    }

    /// Gets all dependencies of a file (direct and transitive)
    #[allow(dead_code)]
    pub fn get_all_dependencies(&self, file: &str) -> HashSet<String> {
        let mut all_deps = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(file.to_string());

        while let Some(current) = queue.pop_front() {
            if all_deps.contains(&current) {
                continue;
            }

            all_deps.insert(current.clone());

            if let Some(deps) = self.dependencies.get(&current) {
                for dep in deps {
                    if !all_deps.contains(dep) {
                        queue.push_back(dep.clone());
                    }
                }
            }
        }

        all_deps.remove(file);
        all_deps
    }

    /// Gets statistics about the dependency graph
    #[allow(dead_code)]
    pub fn get_stats(&self) -> DependencyStats {
        DependencyStats {
            total_files: self.dependencies.len(),
            total_edges: self.dependencies.values().map(|deps| deps.len()).sum(),
            max_dependencies: self.dependencies.values().map(|deps| deps.len()).max().unwrap_or(0),
            max_dependents: self.reverse_dependencies.values().map(|deps| deps.len()).max().unwrap_or(0),
        }
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the dependency graph
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DependencyStats {
    pub total_files: usize,
    pub total_edges: usize,
    pub max_dependencies: usize,
    pub max_dependents: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_extract_package_name() {
        let graph = DependencyGraph::new();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "package com.example.app").unwrap();

        let package = graph.extract_package_name(file.path()).unwrap();
        assert_eq!(package, "com.example.app");
    }

    #[test]
    fn test_extract_imports() {
        let graph = DependencyGraph::new();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "import com.example.UserRepository").unwrap();
        writeln!(file, "import com.example.User").unwrap();

        let imports = graph.extract_imports(file.path()).unwrap();
        assert_eq!(imports.len(), 2);
        assert!(imports.contains(&"com.example.UserRepository".to_string()));
    }
}
