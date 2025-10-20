use anyhow::Result;
use log::info;
use std::collections::{HashMap, HashSet};

use crate::domain::{
    DependencyRepository, ImpactAnalysis, Platform, PlatformImpact, SourceFileRepository,
    SymbolRepository, SymbolUsageRepository,
};

use super::{CalculateDependenciesUseCase, DetectUsageUseCase, ExtractSymbolsUseCase};

/// Use Case: Analyze KMP Impact
///
/// Main orchestrator use case that coordinates all other use cases
/// to produce a complete impact analysis
pub struct AnalyzeImpactUseCase<'a> {
    symbol_repository: &'a dyn SymbolRepository,
    source_file_repository: &'a dyn SourceFileRepository,
    symbol_usage_repository: &'a dyn SymbolUsageRepository,
    dependency_repository: &'a dyn DependencyRepository,
}

impl<'a> AnalyzeImpactUseCase<'a> {
    pub fn new(
        symbol_repository: &'a dyn SymbolRepository,
        source_file_repository: &'a dyn SourceFileRepository,
        symbol_usage_repository: &'a dyn SymbolUsageRepository,
        dependency_repository: &'a dyn DependencyRepository,
    ) -> Self {
        Self {
            symbol_repository,
            source_file_repository,
            symbol_usage_repository,
            dependency_repository,
        }
    }

    /// Execute the complete impact analysis
    pub fn execute(&self, project_path: &str) -> Result<ImpactAnalysis> {
        info!("Starting impact analysis for project: {}", project_path);

        // Step 1: Find all source files
        let kmp_files = self.source_file_repository.find_kmp_files(project_path)?;
        let app_files = self.source_file_repository.find_app_files(project_path)?;

        info!("Found {} KMP files", kmp_files.len());
        info!("Found {} platforms with app files", app_files.len());

        // Step 2: Extract KMP symbols
        let extract_use_case = ExtractSymbolsUseCase::new(self.symbol_repository);
        let symbols = extract_use_case.execute(&kmp_files)?;

        // Step 3: Detect symbol usage across all platforms
        let detect_use_case = DetectUsageUseCase::new(
            self.source_file_repository,
            self.symbol_usage_repository,
        );
        let symbol_usages = detect_use_case.execute(&app_files, &symbols)?;
        let direct_affected_files = detect_use_case.get_affected_files(&symbol_usages);

        // Step 4: Build dependency graph and calculate transitive impact
        let dep_use_case = CalculateDependenciesUseCase::new(self.dependency_repository);
        let mut all_files: Vec<String> = kmp_files.clone();
        for files in app_files.values() {
            all_files.extend(files.clone());
        }
        dep_use_case.build_graph(&all_files)?;

        let transitive_files = dep_use_case.calculate_transitive(&direct_affected_files)?;

        // Step 5: Calculate metrics per platform
        let platform_impacts = self.calculate_platform_impacts(
            &app_files,
            &symbol_usages,
            &direct_affected_files,
            &transitive_files,
        )?;

        // Step 6: Aggregate overall metrics
        let mut impact_analysis = ImpactAnalysis {
            total_symbols: symbols.len(),
            total_app_files: app_files.values().map(|v| v.len()).sum(),
            total_app_lines: platform_impacts.values().map(|p| p.total_lines).sum(),
            affected_files: direct_affected_files.iter().cloned().collect(),
            affected_lines: platform_impacts.values().map(|p| p.affected_lines).sum(),
            impact_ratio: 0.0,
            platform_impacts: platform_impacts
                .into_iter()
                .map(|(k, v)| (k.name().to_string(), v))
                .collect(),
            symbol_usages,
        };

        impact_analysis.calculate_impact_ratio();

        info!(
            "Impact analysis complete: {:.2}% impact coverage",
            impact_analysis.impact_ratio * 100.0
        );

        Ok(impact_analysis)
    }

    /// Calculate platform-specific impacts
    fn calculate_platform_impacts(
        &self,
        app_files: &HashMap<Platform, Vec<String>>,
        symbol_usages: &HashMap<String, Vec<crate::domain::SymbolUsage>>,
        direct_files: &[String],
        transitive_files: &[String],
    ) -> Result<HashMap<Platform, PlatformImpact>> {
        let mut platform_impacts = HashMap::new();

        for (platform, files) in app_files {
            let mut impact = PlatformImpact::new(platform.name().to_string());
            impact.total_files = files.len();

            // Calculate total lines
            for file_path in files {
                if let Ok(file) = self.source_file_repository.read_source_file(file_path) {
                    impact.total_lines += self
                        .source_file_repository
                        .count_code_lines(&file.content, platform.clone());
                }
            }

            // Find affected files for this platform
            let platform_direct: HashSet<String> = direct_files
                .iter()
                .filter(|f| files.contains(f))
                .cloned()
                .collect();

            let platform_transitive: HashSet<String> = transitive_files
                .iter()
                .filter(|f| files.contains(f))
                .cloned()
                .collect();

            impact.affected_files = platform_direct.clone();

            // Calculate affected lines
            for file_path in platform_direct.iter().chain(platform_transitive.iter()) {
                if let Ok(file) = self.source_file_repository.read_source_file(file_path) {
                    impact.affected_lines += self
                        .source_file_repository
                        .count_code_lines(&file.content, platform.clone());
                }
            }

            // Calculate top symbols for this platform
            impact.top_symbols = self.calculate_top_symbols(symbol_usages, files);

            impact.calculate_impact_ratio();

            platform_impacts.insert(platform.clone(), impact);
        }

        Ok(platform_impacts)
    }

    /// Calculate top used symbols for a platform
    fn calculate_top_symbols(
        &self,
        symbol_usages: &HashMap<String, Vec<crate::domain::SymbolUsage>>,
        platform_files: &[String],
    ) -> Vec<(String, usize)> {
        let mut symbol_counts: HashMap<String, usize> = HashMap::new();

        for (symbol_name, usages) in symbol_usages {
            let count = usages
                .iter()
                .filter(|u| platform_files.contains(&u.file_path))
                .count();

            if count > 0 {
                symbol_counts.insert(symbol_name.clone(), count);
            }
        }

        let mut top_symbols: Vec<(String, usize)> = symbol_counts.into_iter().collect();
        top_symbols.sort_by(|a, b| b.1.cmp(&a.1));
        top_symbols.truncate(10);

        top_symbols
    }
}
