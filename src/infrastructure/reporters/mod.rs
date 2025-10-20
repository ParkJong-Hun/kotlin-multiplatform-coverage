use anyhow::Result;
use prettytable::{Cell, Row, Table};
use std::fs;

use crate::analyzer::models::AnalysisResult;
use crate::domain::ImpactAnalysis;

/// Reporter for outputting analysis results in various formats
pub struct Reporter {
    format: ReportFormat,
}

/// Report output format
#[derive(Debug, Clone, PartialEq)]
pub enum ReportFormat {
    Table,
    Json,
    Markdown,
}

impl Reporter {
    /// Creates a new Reporter instance
    pub fn new(format: &str) -> Result<Self> {
        let format = match format.to_lowercase().as_str() {
            "table" => ReportFormat::Table,
            "json" => ReportFormat::Json,
            "markdown" | "md" => ReportFormat::Markdown,
            _ => anyhow::bail!("Unsupported output format: {}", format),
        };

        Ok(Self { format })
    }

    /// Outputs the analysis results as a report
    #[allow(dead_code)]
    pub fn report(&self, result: &AnalysisResult, output_path: Option<&str>) -> Result<()> {
        let content = match self.format {
            ReportFormat::Table => self.format_as_table(result),
            ReportFormat::Json => self.format_as_json(result)?,
            ReportFormat::Markdown => self.format_as_markdown(result),
        };

        // Save to file or print to console
        if let Some(path) = output_path {
            fs::write(path, content)?;
            println!("Results saved to file: {}", path);
        } else {
            println!("{}", content);
        }

        Ok(())
    }

    /// Formats as a table
    #[allow(dead_code)]
    fn format_as_table(&self, result: &AnalysisResult) -> String {
        let mut output = String::new();

        // Impact Coverage Summary
        output.push_str("=== KMP Impact Coverage Report ===\n\n");

        let impact = &result.impact_coverage;
        output.push_str(&format!("📊 Impact Coverage: {:.2}%\n", impact.impact_ratio * 100.0));
        output.push_str(&format!("   Affected Lines: {} / {}\n\n", impact.affected_lines, impact.total_app_lines));

        output.push_str(&format!("🎯 Direct Impact: {} files\n", impact.direct_impact_files.len()));
        output.push_str(&format!("🔗 Transitive Impact: {} files\n", impact.transitive_impact_files.len()));
        output.push_str(&format!("📦 KMP Symbols: {}\n\n", impact.kmp_symbols.len()));

        // Per-platform impact
        if !impact.platform_impact.is_empty() {
            output.push_str("=== Platform Impact Breakdown ===\n\n");
            let mut platform_table = Table::new();
            platform_table.add_row(Row::new(vec![
                Cell::new("Platform"),
                Cell::new("Impact %"),
                Cell::new("Affected Files"),
                Cell::new("Affected Lines"),
                Cell::new("Total Lines"),
            ]));

            for (platform_name, platform_impact) in &impact.platform_impact {
                platform_table.add_row(Row::new(vec![
                    Cell::new(platform_name),
                    Cell::new(&format!("{:.2}%", platform_impact.impact_ratio * 100.0)),
                    Cell::new(&platform_impact.direct_impact_files.len().to_string()),
                    Cell::new(&platform_impact.affected_lines.to_string()),
                    Cell::new(&platform_impact.total_lines.to_string()),
                ]));
            }

            output.push_str(&platform_table.to_string());
            output.push_str("\n");
        }

        // Top used symbols
        if !impact.symbol_usage.is_empty() {
            output.push_str("=== Top 10 Used KMP Symbols ===\n\n");
            let mut symbols: Vec<_> = impact.symbol_usage.iter().collect();
            symbols.sort_by(|a, b| b.1.reference_count.cmp(&a.1.reference_count));

            let mut symbol_table = Table::new();
            symbol_table.add_row(Row::new(vec![
                Cell::new("Symbol"),
                Cell::new("References"),
                Cell::new("Used in Files"),
            ]));

            for (symbol_name, usage) in symbols.iter().take(10) {
                symbol_table.add_row(Row::new(vec![
                    Cell::new(symbol_name),
                    Cell::new(&usage.reference_count.to_string()),
                    Cell::new(&usage.used_in_files.len().to_string()),
                ]));
            }

            output.push_str(&symbol_table.to_string());
            output.push_str("\n");
        }

        // Module details (if any)
        if !result.modules.is_empty() {
            output.push_str("\n=== Module Details ===\n\n");
            let mut table = Table::new();

            table.add_row(Row::new(vec![
                Cell::new("Module"),
                Cell::new("KMP"),
                Cell::new("KMP Lines"),
                Cell::new("Total Lines"),
                Cell::new("Coverage"),
            ]));

            for module in &result.modules {
                table.add_row(Row::new(vec![
                    Cell::new(&module.name),
                    Cell::new(if module.is_kmp { "O" } else { "X" }),
                    Cell::new(&module.kmp_lines.to_string()),
                    Cell::new(&module.total_lines.to_string()),
                    Cell::new(&format!("{:.2}%", module.coverage * 100.0)),
                ]));
            }

            output.push_str(&table.to_string());
        }

        output
    }

    /// Formats as JSON
    #[allow(dead_code)]
    fn format_as_json(&self, result: &AnalysisResult) -> Result<String> {
        Ok(serde_json::to_string_pretty(result)?)
    }

    /// Formats as Markdown
    #[allow(dead_code)]
    fn format_as_markdown(&self, result: &AnalysisResult) -> String {
        let mut md = String::from("# Kotlin Multiplatform Impact Coverage Report\n\n");

        let impact = &result.impact_coverage;

        // Summary
        md.push_str("## 📊 Impact Summary\n\n");
        md.push_str(&format!("- **Impact Coverage**: {:.2}%\n", impact.impact_ratio * 100.0));
        md.push_str(&format!("- **Affected Lines**: {} / {}\n", impact.affected_lines, impact.total_app_lines));
        md.push_str(&format!("- **Direct Impact Files**: {}\n", impact.direct_impact_files.len()));
        md.push_str(&format!("- **Transitive Impact Files**: {}\n", impact.transitive_impact_files.len()));
        md.push_str(&format!("- **Total KMP Symbols**: {}\n\n", impact.kmp_symbols.len()));

        // Per-platform impact
        if !impact.platform_impact.is_empty() {
            md.push_str("## 📱 Platform Impact Breakdown\n\n");
            md.push_str("| Platform | Impact % | Affected Files | Affected Lines | Total Lines |\n");
            md.push_str("|----------|----------|----------------|----------------|-------------|\n");

            for (platform_name, platform_impact) in &impact.platform_impact {
                md.push_str(&format!(
                    "| {} | {:.2}% | {} | {} | {} |\n",
                    platform_name,
                    platform_impact.impact_ratio * 100.0,
                    platform_impact.direct_impact_files.len(),
                    platform_impact.affected_lines,
                    platform_impact.total_lines
                ));
            }
            md.push_str("\n");
        }

        // Top used symbols
        if !impact.symbol_usage.is_empty() {
            md.push_str("## 🎯 Top Used KMP Symbols\n\n");
            md.push_str("| Symbol | References | Used in Files |\n");
            md.push_str("|--------|------------|---------------|\n");

            let mut symbols: Vec<_> = impact.symbol_usage.iter().collect();
            symbols.sort_by(|a, b| b.1.reference_count.cmp(&a.1.reference_count));

            for (symbol_name, usage) in symbols.iter().take(10) {
                md.push_str(&format!(
                    "| {} | {} | {} |\n",
                    symbol_name,
                    usage.reference_count,
                    usage.used_in_files.len()
                ));
            }
            md.push_str("\n");
        }

        // Symbol type breakdown
        md.push_str("## 📦 KMP Symbol Breakdown\n\n");
        let mut class_count = 0;
        let mut function_count = 0;
        let mut property_count = 0;
        let mut other_count = 0;

        for symbol in &impact.kmp_symbols {
            match symbol.symbol_type {
                crate::analyzer::models::SymbolType::Class => class_count += 1,
                crate::analyzer::models::SymbolType::Function => function_count += 1,
                crate::analyzer::models::SymbolType::Property => property_count += 1,
                _ => other_count += 1,
            }
        }

        md.push_str(&format!("- **Classes**: {}\n", class_count));
        md.push_str(&format!("- **Functions**: {}\n", function_count));
        md.push_str(&format!("- **Properties**: {}\n", property_count));
        md.push_str(&format!("- **Others**: {}\n\n", other_count));

        // Module details (if any)
        if !result.modules.is_empty() {
            md.push_str("## Module Details\n\n");
            md.push_str("| Module | KMP | KMP Lines | Total Lines | Coverage |\n");
            md.push_str("|--------|-----|-----------|-------------|----------|\n");

            for module in &result.modules {
                md.push_str(&format!(
                    "| {} | {} | {} | {} | {:.2}% |\n",
                    module.name,
                    if module.is_kmp { "✓" } else { "✗" },
                    module.kmp_lines,
                    module.total_lines,
                    module.coverage * 100.0
                ));
            }
        }

        md
    }

    /// New method for Clean Architecture: Report ImpactAnalysis
    pub fn report_impact_analysis(
        &self,
        analysis: &ImpactAnalysis,
        output_path: Option<&str>,
    ) -> Result<()> {
        let content = match self.format {
            ReportFormat::Table => self.format_impact_as_table(analysis),
            ReportFormat::Json => serde_json::to_string_pretty(analysis)?,
            ReportFormat::Markdown => self.format_impact_as_markdown(analysis),
        };

        if let Some(path) = output_path {
            fs::write(path, content)?;
            println!("Results saved to file: {}", path);
        } else {
            println!("{}", content);
        }

        Ok(())
    }

    fn format_impact_as_table(&self, analysis: &ImpactAnalysis) -> String {
        let mut output = String::new();

        output.push_str("=== KMP Impact Coverage Report (Clean Architecture) ===\n\n");

        output.push_str(&format!("📊 Impact Coverage: {:.2}%\n", analysis.impact_ratio * 100.0));
        output.push_str(&format!(
            "   Affected Lines: {} / {}\n\n",
            analysis.affected_lines, analysis.total_app_lines
        ));

        output.push_str(&format!("🎯 Direct Impact: {} files\n", analysis.affected_files.len()));
        output.push_str(&format!("📦 KMP Symbols: {}\n", analysis.total_symbols));
        output.push_str(&format!("📁 Total App Files: {}\n\n", analysis.total_app_files));

        // Platform breakdown
        if !analysis.platform_impacts.is_empty() {
            output.push_str("=== Platform Impact Breakdown ===\n\n");
            let mut platform_table = Table::new();
            platform_table.add_row(Row::new(vec![
                Cell::new("Platform"),
                Cell::new("Impact %"),
                Cell::new("Affected Files"),
                Cell::new("Affected Lines"),
                Cell::new("Total Lines"),
            ]));

            for (platform_name, impact) in &analysis.platform_impacts {
                platform_table.add_row(Row::new(vec![
                    Cell::new(platform_name),
                    Cell::new(&format!("{:.2}%", impact.impact_ratio * 100.0)),
                    Cell::new(&impact.affected_files.len().to_string()),
                    Cell::new(&impact.affected_lines.to_string()),
                    Cell::new(&impact.total_lines.to_string()),
                ]));
            }

            output.push_str(&platform_table.to_string());
            output.push_str("\n");
        }

        output
    }

    fn format_impact_as_markdown(&self, analysis: &ImpactAnalysis) -> String {
        let mut md = String::from("# Kotlin Multiplatform Impact Coverage Report\n\n");

        md.push_str("## 📊 Impact Summary\n\n");
        md.push_str(&format!("- **Impact Coverage**: {:.2}%\n", analysis.impact_ratio * 100.0));
        md.push_str(&format!(
            "- **Affected Lines**: {} / {}\n",
            analysis.affected_lines, analysis.total_app_lines
        ));
        md.push_str(&format!("- **Direct Impact Files**: {}\n", analysis.affected_files.len()));
        md.push_str(&format!("- **Total KMP Symbols**: {}\n\n", analysis.total_symbols));

        // Platform breakdown
        if !analysis.platform_impacts.is_empty() {
            md.push_str("## 📱 Platform Impact Breakdown\n\n");
            md.push_str("| Platform | Impact % | Affected Files | Affected Lines | Total Lines |\n");
            md.push_str("|----------|----------|----------------|----------------|-------------|\n");

            for (platform_name, impact) in &analysis.platform_impacts {
                md.push_str(&format!(
                    "| {} | {:.2}% | {} | {} | {} |\n",
                    platform_name,
                    impact.impact_ratio * 100.0,
                    impact.affected_files.len(),
                    impact.affected_lines,
                    impact.total_lines
                ));
            }
            md.push_str("\n");
        }

        md
    }
}
