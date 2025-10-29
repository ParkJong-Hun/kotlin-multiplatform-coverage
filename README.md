# Kotlin Multiplatform Coverage

A tool to analyze the **impact coverage** of Kotlin Multiplatform (KMP) code in monorepos, similar to how test coverage measures code execution.

## What is Impact Coverage?

Unlike simple line-counting metrics, **Impact Coverage** measures how much of your app code is actually affected by KMP code:

- **Direct Impact**: Code that directly calls/uses KMP symbols (classes, functions, properties)
- **Transitive Impact**: Code that depends on KMP-using code (through dependency chains)
- **Impact Ratio**: `(Affected Lines) / (Total App Lines)` - shows the real reach of your KMP code

Think of it like test coverage, but for measuring how much your KMP code influences the entire codebase.

## Features

- 🎯 **Impact Coverage Analysis**: Measure real KMP influence, not just code percentage
- 🔍 **Symbol Extraction**: Identify all public KMP classes, functions, and properties
- 🤖 **Dynamic Project Detection**: Automatically discovers KMP, Android, and iOS projects
  - No hardcoded paths - analyzes project structure intelligently
  - Detects projects by build files (build.gradle.kts, AndroidManifest.xml, .xcodeproj)
  - Finds source directories automatically
  - Fallback to pattern matching for edge cases
- 📊 **Multi-Platform Support**:
  - **Android**: Kotlin + Java
  - **iOS**: Swift + Objective-C
  - Extensible architecture for adding more platforms
- 🌐 **Per-Platform Impact**: Separate analysis for each platform
- 📈 **Usage Detection**: Find where KMP symbols are used across all platforms
- 🔗 **Dependency Graph**: Track direct and transitive code dependencies
- 📋 **Multiple Output Formats**: Table, JSON, and Markdown reports
- 🏆 **Top Symbols Ranking**: See which KMP symbols are most heavily used

## Installation

```bash
cargo build --release
```

## Usage

### Basic Usage

```bash
# Analyze current directory
kotlin-multiplatform-coverage

# Analyze specific path
kotlin-multiplatform-coverage -p /path/to/project

# Enable verbose logging
kotlin-multiplatform-coverage -v

# Output results in JSON format
kotlin-multiplatform-coverage -f json

# Save results to file
kotlin-multiplatform-coverage -f json -o result.json
```

### Command Options

- `-p, --path <PATH>`: Project path to analyze (default: current directory)
- `-f, --format <FORMAT>`: Output format - table, json, markdown (default: table)
- `-v, --verbose`: Enable verbose logging
- `-o, --output <FILE>`: Output file path to save results

## How It Works

1. **Dynamic Project Detection**:
   - Scans for build.gradle.kts with kotlin("multiplatform") plugin
   - Finds AndroidManifest.xml and build.gradle with Android plugin
   - Detects .xcodeproj or .xcworkspace for iOS projects
   - Locates source directories automatically (no hardcoded paths!)
2. **Symbol Extraction**: Scans KMP modules to find all public symbols (classes, functions, properties)
3. **Usage Detection**: Searches app code for references to these KMP symbols using regex patterns
4. **Dependency Graph**: Builds a graph of file dependencies to track transitive impact
5. **Impact Calculation**: Computes affected lines and impact ratio

```
Detect Projects → Extract Symbols → Find Usage → Build Dep Graph → Calculate Impact
```

### Project Detection Examples

**KMP Project Detection:**
- Looks for `kotlin("multiplatform")` in build.gradle.kts
- Finds commonMain, androidMain, iosMain source sets
- Detects "shared" module with KMP structure

**Android Project Detection:**
- Finds AndroidManifest.xml
- Checks build.gradle for Android plugin
- Locates src/main/java or src/main/kotlin directories

**iOS Project Detection:**
- Finds .xcodeproj or .xcworkspace
- Scans for Swift and Objective-C source files
- Detects iosApp, iOS, ios directories

## Output Example

```
=== KMP Impact Coverage Report ===

📊 Impact Coverage: 45.23%
   Affected Lines: 1,234 / 2,729

🎯 Direct Impact: 23 files
🔗 Transitive Impact: 15 files
📦 KMP Symbols: 87

=== Platform Impact Breakdown ===
+----------+-----------+----------------+----------------+-------------+
| Platform | Impact %  | Affected Files | Affected Lines | Total Lines |
+----------+-----------+----------------+----------------+-------------+
| Android  | 52.30%    | 15             | 892            | 1,705       |
| iOS      | 35.60%    | 8              | 342            | 1,024       |
+----------+-----------+----------------+----------------+-------------+

=== Top 10 Used KMP Symbols ===
+------------------+------------+----------------+
| Symbol           | References | Used in Files  |
+------------------+------------+----------------+
| UserRepository   | 45         | 12             |
| User             | 38         | 15             |
| fetchUserData    | 23         | 8              |
+------------------+------------+----------------+
```

## Supported Platforms

### Android
- **Languages**: Kotlin (.kt, .kts) + Java (.java)
- **Detection**:
  - Automatically finds projects with AndroidManifest.xml
  - Detects build.gradle with Android plugin
  - No hardcoded paths required!
- **Analyzes**: Both Kotlin and Java files for KMP symbol usage

### iOS
- **Languages**: Swift (.swift) + Objective-C (.m, .mm, .h)
- **Detection**:
  - Automatically finds .xcodeproj or .xcworkspace
  - Scans for Swift/Objective-C source files
  - Works with any iOS project structure!
- **Analyzes**: KMP framework imports and symbol usage in Swift/Objective-C code

### Kotlin Multiplatform
- **Detection**:
  - Finds build.gradle.kts with kotlin("multiplatform")
  - Detects commonMain, androidMain, iosMain source sets
  - Automatically discovers "shared" modules
  - No manual configuration needed!

### Extensible Architecture
The platform system is designed to be easily extended. To add a new platform:
1. Implement the `Platform` trait in `src/adapters/platforms/`
2. Add detection logic in `src/adapters/project_detector.rs`
3. Register it in `PlatformRegistry::new()`
4. Done! The analyzer automatically supports the new platform
