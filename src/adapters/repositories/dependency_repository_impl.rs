use anyhow::Result;
use std::sync::Mutex;

use crate::analyzer::dependency_graph::DependencyGraph;
use crate::domain::{DependencyRepository, SourceFile};

/// Adapter implementation of DependencyRepository
pub struct DependencyRepositoryImpl {
    graph: Mutex<DependencyGraph>,
}

impl DependencyRepositoryImpl {
    pub fn new() -> Self {
        Self {
            graph: Mutex::new(DependencyGraph::new()),
        }
    }
}

impl Default for DependencyRepositoryImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyRepository for DependencyRepositoryImpl {
    fn build_dependency_graph(&self, file_paths: &[String]) -> Result<()> {
        let paths: Vec<std::path::PathBuf> = file_paths
            .iter()
            .map(|s| std::path::PathBuf::from(s))
            .collect();

        self.graph.lock().unwrap().build(&paths)?;
        Ok(())
    }

    fn calculate_transitive_dependencies(&self, direct_files: &[String]) -> Result<Vec<String>> {
        let direct_set: std::collections::HashSet<String> =
            direct_files.iter().cloned().collect();

        let transitive_set = self.graph.lock().unwrap().compute_transitive_impact(&direct_set);

        Ok(transitive_set.into_iter().collect())
    }

    fn extract_imports(&self, source_file: &SourceFile) -> Result<Vec<String>> {
        use regex::Regex;

        let mut imports = Vec::new();

        // For Kotlin/Java
        let import_regex = Regex::new(r"(?m)^import\s+([a-zA-Z0-9_.]+)").unwrap();
        for cap in import_regex.captures_iter(&source_file.content) {
            if let Some(import) = cap.get(1) {
                imports.push(import.as_str().to_string());
            }
        }

        // For Swift
        let swift_import_regex = Regex::new(r"(?m)^import\s+([A-Za-z0-9_]+)").unwrap();
        for cap in swift_import_regex.captures_iter(&source_file.content) {
            if let Some(import) = cap.get(1) {
                imports.push(import.as_str().to_string());
            }
        }

        // For Objective-C
        let objc_import_regex = Regex::new(r#"(?m)^#import\s+[<"]([A-Za-z0-9_/]+)[>"]"#).unwrap();
        for cap in objc_import_regex.captures_iter(&source_file.content) {
            if let Some(import) = cap.get(1) {
                imports.push(import.as_str().to_string());
            }
        }

        Ok(imports)
    }
}
