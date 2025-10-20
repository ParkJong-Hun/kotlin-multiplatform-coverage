use anyhow::Result;
use clap::Parser;
use log::info;

// Clean Architecture Layers
mod domain;
mod use_cases;
mod adapters;
mod infrastructure;

// Supporting utilities (cross-cutting concerns)
mod utils;

// Legacy modules (to be phased out)
mod analyzer;

use adapters::{
    DependencyRepositoryImpl, SourceFileRepositoryImpl, SymbolRepositoryImpl,
    SymbolUsageRepositoryImpl,
};
use infrastructure::Reporter;
use use_cases::AnalyzeImpactUseCase;

/// Kotlin Multiplatform Coverage Analyzer
///
/// Analyzes the impact of Kotlin Multiplatform code in a monorepo.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Project path to analyze
    #[arg(short, long, default_value = ".")]
    path: String,

    /// Output format (json, table, markdown)
    #[arg(short, long, default_value = "table")]
    format: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Output file path to save results
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logger
    if args.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();
    }

    info!("Starting Kotlin Multiplatform Coverage Analyzer (Clean Architecture)");
    info!("Analysis path: {}", args.path);

    // Clean Architecture: Dependency Injection
    // Create repository implementations (adapters)
    let symbol_repo = SymbolRepositoryImpl::new();
    let source_file_repo = SourceFileRepositoryImpl::new();
    let symbol_usage_repo = SymbolUsageRepositoryImpl::new();
    let dependency_repo = DependencyRepositoryImpl::new();

    // Create use case with injected dependencies
    let analyze_use_case = AnalyzeImpactUseCase::new(
        &symbol_repo,
        &source_file_repo,
        &symbol_usage_repo,
        &dependency_repo,
    );

    // Execute use case
    let impact_analysis = analyze_use_case.execute(&args.path)?;

    // Report results (infrastructure layer)
    let reporter = Reporter::new(&args.format)?;
    reporter.report_impact_analysis(&impact_analysis, args.output.as_deref())?;

    info!("Analysis completed");
    Ok(())
}
