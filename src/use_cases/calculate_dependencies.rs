use anyhow::Result;
use log::info;

use crate::domain::DependencyRepository;

/// Use Case: Calculate Dependencies
///
/// Responsibility: Build dependency graph and find transitive dependencies
pub struct CalculateDependenciesUseCase<'a> {
    dependency_repository: &'a dyn DependencyRepository,
}

impl<'a> CalculateDependenciesUseCase<'a> {
    pub fn new(dependency_repository: &'a dyn DependencyRepository) -> Self {
        Self {
            dependency_repository,
        }
    }

    /// Build dependency graph for all files
    pub fn build_graph(&self, all_files: &[String]) -> Result<()> {
        info!("Building dependency graph for {} files", all_files.len());
        self.dependency_repository.build_dependency_graph(all_files)?;
        info!("Dependency graph built successfully");
        Ok(())
    }

    /// Calculate transitive dependencies (files that depend on the given files)
    pub fn calculate_transitive(&self, direct_files: &[String]) -> Result<Vec<String>> {
        info!("Calculating transitive dependencies for {} files", direct_files.len());

        let transitive = self.dependency_repository.calculate_transitive_dependencies(direct_files)?;

        info!("Found {} transitive dependencies", transitive.len());
        Ok(transitive)
    }
}
