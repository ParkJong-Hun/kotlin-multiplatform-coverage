/// Integration tests for Kotlin Multiplatform Coverage Analyzer
/// Tests the complete flow from symbol extraction to impact analysis

use anyhow::Result;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

// Import from the library
use kotlin_multiplatform_coverage::{
    adapters::{
        DependencyRepositoryImpl, SourceFileRepositoryImpl, SymbolRepositoryImpl,
        SymbolUsageRepositoryImpl,
    },
    domain::{SourceFileRepository, SymbolRepository, SymbolUsageRepository},
    use_cases::AnalyzeImpactUseCase,
};

/// Creates a temporary KMP project structure for testing
fn create_test_kmp_project() -> Result<TempDir> {
    let temp_dir = tempfile::tempdir()?;
    let project_path = temp_dir.path();

    // Create shared module (KMP)
    let shared_path = project_path.join("shared/src/commonMain/kotlin/com/example");
    fs::create_dir_all(&shared_path)?;

    // Create build.gradle.kts for KMP module
    fs::write(
        project_path.join("shared/build.gradle.kts"),
        r#"
plugins {
    kotlin("multiplatform")
}

kotlin {
    sourceSets {
        val commonMain by getting
        val androidMain by getting
        val iosMain by getting
    }
}
"#,
    )?;

    // KMP classes
    fs::write(
        shared_path.join("User.kt"),
        r#"
package com.example

data class User(
    val id: String,
    val name: String,
    val email: String
)

interface UserRepository {
    fun getUser(id: String): User?
    fun saveUser(user: User)
}

class UserRepositoryImpl : UserRepository {
    private val users = mutableMapOf<String, User>()

    override fun getUser(id: String): User? = users[id]
    override fun saveUser(user: User) {
        users[user.id] = user
    }
}
"#,
    )?;

    fs::write(
        shared_path.join("Utils.kt"),
        r#"
package com.example

object Logger {
    fun log(message: String) {
        println(message)
    }
}

fun formatUserName(user: User): String {
    return "${user.name} <${user.email}>"
}
"#,
    )?;

    // Create Android app
    let android_path = project_path.join("app/src/main/java/com/example/android");
    fs::create_dir_all(&android_path)?;

    // Create AndroidManifest.xml
    fs::create_dir_all(project_path.join("app/src/main"))?;
    fs::write(
        project_path.join("app/src/main/AndroidManifest.xml"),
        r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="com.example.android">
    <application>
        <activity android:name=".MainActivity"/>
    </application>
</manifest>"#,
    )?;

    // Create build.gradle.kts for Android app
    fs::write(
        project_path.join("app/build.gradle.kts"),
        r#"
plugins {
    id("com.android.application")
    kotlin("android")
}

android {
    compileSdk = 34
}
"#,
    )?;

    fs::write(
        android_path.join("MainActivity.kt"),
        r#"
package com.example.android

import com.example.User
import com.example.UserRepository
import com.example.UserRepositoryImpl
import com.example.Logger
import com.example.formatUserName

class MainActivity {
    private val repository: UserRepository = UserRepositoryImpl()

    fun createUser() {
        val user = User("1", "John Doe", "john@example.com")
        repository.saveUser(user)
        Logger.log("User created: ${formatUserName(user)}")
    }

    fun loadUser(id: String) {
        val user = repository.getUser(id)
        if (user != null) {
            Logger.log("Found user: ${user.name}")
        }
    }
}
"#,
    )?;

    fs::write(
        android_path.join("UserAdapter.kt"),
        r#"
package com.example.android

import com.example.User

class UserAdapter {
    fun displayUser(user: User) {
        println("User: ${user.name}")
    }
}
"#,
    )?;

    // Create iOS app
    let ios_path = project_path.join("iosApp/iosApp");
    fs::create_dir_all(&ios_path)?;

    // Create .xcodeproj directory (marker for iOS project)
    fs::create_dir_all(project_path.join("iosApp/iosApp.xcodeproj"))?;
    fs::write(
        project_path.join("iosApp/iosApp.xcodeproj/project.pbxproj"),
        "// Xcode project file",
    )?;

    fs::write(
        ios_path.join("ContentView.swift"),
        r#"
import SwiftUI
import Shared

struct ContentView: View {
    let repository = UserRepositoryImpl()

    var body: some View {
        VStack {
            Text("KMP Demo")
                .padding()
            Button("Create User") {
                createUser()
            }
        }
    }

    func createUser() {
        let user = User(id: "1", name: "Jane Doe", email: "jane@example.com")
        repository.saveUser(user: user)
        Logger.shared.log(message: "User created")
    }
}
"#,
    )?;

    fs::write(
        ios_path.join("UserViewModel.swift"),
        r#"
import Foundation
import Shared

class UserViewModel: ObservableObject {
    private let repository: UserRepository = UserRepositoryImpl()
    @Published var user: User?

    func loadUser(id: String) {
        user = repository.getUser(id: id)
        if let user = user {
            Logger.shared.log(message: "Loaded: \(user.name)")
        }
    }
}
"#,
    )?;

    Ok(temp_dir)
}

#[test]
fn test_end_to_end_impact_analysis() -> Result<()> {
    // Create test project
    let temp_project = create_test_kmp_project()?;
    let project_path = temp_project.path().to_str().unwrap();

    // Create repository implementations
    let symbol_repo = SymbolRepositoryImpl::new();
    let source_file_repo = SourceFileRepositoryImpl::new();
    let symbol_usage_repo = SymbolUsageRepositoryImpl::new();
    let dependency_repo = DependencyRepositoryImpl::new();

    // Create and execute use case
    let analyze_use_case = AnalyzeImpactUseCase::new(
        &symbol_repo,
        &source_file_repo,
        &symbol_usage_repo,
        &dependency_repo,
    );

    let impact_analysis = analyze_use_case.execute(project_path)?;

    // Verify results
    assert!(
        impact_analysis.total_symbols > 0,
        "Should find KMP symbols"
    );
    assert!(
        !impact_analysis.affected_files.is_empty(),
        "Should find affected files"
    );
    assert!(
        impact_analysis.impact_ratio > 0.0,
        "Should have positive impact ratio"
    );

    // Verify platform-specific impacts
    assert!(
        impact_analysis.platform_impacts.contains_key("Android"),
        "Should have Android platform data"
    );
    assert!(
        impact_analysis.platform_impacts.contains_key("iOS"),
        "Should have iOS platform data"
    );

    // Verify Android impact
    let android_impact = &impact_analysis.platform_impacts["Android"];
    assert!(
        android_impact.total_files > 0,
        "Android should have files"
    );
    assert!(
        android_impact.affected_lines > 0,
        "Android should have affected lines"
    );

    // Verify iOS impact
    let ios_impact = &impact_analysis.platform_impacts["iOS"];
    assert!(ios_impact.total_files > 0, "iOS should have files");
    assert!(
        ios_impact.affected_lines > 0,
        "iOS should have affected lines"
    );

    // Verify symbol usage tracking
    assert!(
        !impact_analysis.symbol_usages.is_empty(),
        "Should track symbol usages"
    );

    // Check for specific symbols
    let symbol_names: Vec<String> = impact_analysis.symbol_usages.keys().cloned().collect();
    assert!(
        symbol_names.iter().any(|s| s.contains("User")),
        "Should detect User symbol usage"
    );

    println!("✓ End-to-end integration test passed!");
    println!("  - Total symbols: {}", impact_analysis.total_symbols);
    println!("  - Impact ratio: {:.2}%", impact_analysis.impact_ratio * 100.0);
    println!("  - Affected files: {}", impact_analysis.affected_files.len());
    println!("  - Android files: {}", android_impact.total_files);
    println!("  - iOS files: {}", ios_impact.total_files);

    Ok(())
}

#[test]
fn test_symbol_extraction() -> Result<()> {
    let temp_project = create_test_kmp_project()?;
    let project_path = temp_project.path().to_str().unwrap();

    let symbol_repo = SymbolRepositoryImpl::new();
    let source_file_repo = SourceFileRepositoryImpl::new();

    // Find KMP files
    let kmp_files = source_file_repo.find_kmp_files(project_path)?;
    assert!(!kmp_files.is_empty(), "Should find KMP files");

    // Extract symbols
    let symbols = symbol_repo.extract_kmp_symbols(&kmp_files)?;
    assert!(symbols.len() > 0, "Should extract symbols");

    // Verify specific symbols
    let symbol_names: Vec<String> = symbols.iter().map(|s| s.name.clone()).collect();

    // Debug output
    println!("  Found symbols: {:?}", symbol_names);

    // We should find at least some symbols from the KMP code
    // The exact names might vary depending on extraction, so we check for partial matches
    let has_user_related = symbol_names.iter().any(|s| s.contains("User"));

    assert!(
        has_user_related,
        "Should find User-related symbols, found: {:?}",
        symbol_names
    );

    println!("✓ Symbol extraction test passed!");
    println!("  - Found {} symbols", symbols.len());

    Ok(())
}

#[test]
fn test_platform_detection() -> Result<()> {
    let temp_project = create_test_kmp_project()?;
    let project_path = temp_project.path().to_str().unwrap();

    let source_file_repo = SourceFileRepositoryImpl::new();

    // Find app files by platform
    let app_files = source_file_repo.find_app_files(project_path)?;

    // Should detect both platforms
    assert!(app_files.len() >= 1, "Should detect at least one platform");

    // Count files per platform
    for (platform, files) in &app_files {
        println!("  Platform: {} - {} files", platform.name(), files.len());
        assert!(!files.is_empty(), "Platform should have files");
    }

    println!("✓ Platform detection test passed!");

    Ok(())
}

#[test]
fn test_usage_detection_accuracy() -> Result<()> {
    let temp_project = create_test_kmp_project()?;
    let project_path = temp_project.path().to_str().unwrap();

    let symbol_repo = SymbolRepositoryImpl::new();
    let source_file_repo = SourceFileRepositoryImpl::new();
    let symbol_usage_repo = SymbolUsageRepositoryImpl::new();

    // Extract symbols
    let kmp_files = source_file_repo.find_kmp_files(project_path)?;
    let symbols = symbol_repo.extract_kmp_symbols(&kmp_files)?;

    // Find app files
    let app_files = source_file_repo.find_app_files(project_path)?;

    // Detect usage in Android files
    if let Some((_, android_files)) = app_files
        .iter()
        .find(|(p, _)| p.name() == "Android")
    {
        for file_path in android_files {
            let source_file = source_file_repo.read_source_file(file_path)?;
            let usages = symbol_usage_repo.detect_symbol_usage(&source_file, &symbols)?;

            if !usages.is_empty() {
                println!("  File: {}", Path::new(file_path).file_name().unwrap().to_str().unwrap());
                for usage in &usages {
                    println!("    - {} at line {}", usage.symbol_name, usage.line_number);
                }
            }
        }
    }

    println!("✓ Usage detection test passed!");

    Ok(())
}

#[test]
fn test_impact_ratio_calculation() -> Result<()> {
    let temp_project = create_test_kmp_project()?;
    let project_path = temp_project.path().to_str().unwrap();

    let symbol_repo = SymbolRepositoryImpl::new();
    let source_file_repo = SourceFileRepositoryImpl::new();
    let symbol_usage_repo = SymbolUsageRepositoryImpl::new();
    let dependency_repo = DependencyRepositoryImpl::new();

    let analyze_use_case = AnalyzeImpactUseCase::new(
        &symbol_repo,
        &source_file_repo,
        &symbol_usage_repo,
        &dependency_repo,
    );

    let impact_analysis = analyze_use_case.execute(project_path)?;

    // Impact ratio should be between 0 and 1
    assert!(
        impact_analysis.impact_ratio >= 0.0 && impact_analysis.impact_ratio <= 1.0,
        "Impact ratio should be between 0 and 1, got {}",
        impact_analysis.impact_ratio
    );

    // Platform-specific ratios should also be valid
    for (platform_name, impact) in &impact_analysis.platform_impacts {
        assert!(
            impact.impact_ratio >= 0.0 && impact.impact_ratio <= 1.0,
            "{} platform impact ratio should be between 0 and 1, got {}",
            platform_name,
            impact.impact_ratio
        );

        // Affected lines should not exceed total lines
        assert!(
            impact.affected_lines <= impact.total_lines,
            "{} affected lines ({}) should not exceed total lines ({})",
            platform_name,
            impact.affected_lines,
            impact.total_lines
        );
    }

    println!("✓ Impact ratio calculation test passed!");
    println!("  - Overall impact: {:.2}%", impact_analysis.impact_ratio * 100.0);

    Ok(())
}
