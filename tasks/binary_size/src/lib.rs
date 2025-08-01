#![expect(clippy::print_stdout)]

use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    fs,
};

use chrono::Utc;
use humansize::{format_size, DECIMAL};
use miette::{Context, IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use pico_args::Arguments;
use regex::Regex;
use serde::{Deserialize, Serialize};

use oxc_tasks_common::project_root;

#[derive(Debug, Clone)]
struct Options {
    /// Generate detailed analysis report
    detailed: bool,
    /// Compare with baseline if available
    compare: bool,
    /// Save results as baseline
    save_baseline: bool,
    /// Target binary to analyze
    target: String,
    /// Features to enable
    features: Vec<String>,
    /// Output format (human, json)
    format: OutputFormat,
}

#[derive(Debug, Clone)]
enum OutputFormat {
    Human,
    Json,
}

#[derive(Debug, Serialize, Deserialize)]
struct BinarySizeReport {
    binary_path: String,
    file_size: u64,
    stripped_size: u64,
    text_section_size: u64,
    bloat_analysis: BloatAnalysis,
    llvm_lines_analysis: LlvmLinesAnalysis,
    dependency_analysis: DependencyAnalysis,
    features_enabled: Vec<String>,
    build_info: BuildInfo,
    timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BloatAnalysis {
    largest_functions: Vec<BloatItem>,
    largest_crates: Vec<BloatItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BloatItem {
    name: String,
    size: u64,
    percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct LlvmLinesAnalysis {
    total_lines: u64,
    top_functions: Vec<LlvmLinesItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LlvmLinesItem {
    name: String,
    lines: u64,
    copies: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct DependencyAnalysis {
    total_dependencies: usize,
    largest_dependencies: Vec<DependencyItem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DependencyItem {
    name: String,
    size_bytes: u64,
    features: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BuildInfo {
    target_triple: String,
    opt_level: String,
    debug_info: bool,
    lto: String,
    codegen_units: String,
    panic_strategy: String,
}

/// Analyze binary size of oxc tools
/// 
/// # Errors
/// 
/// Returns error if binary analysis fails
pub fn run() -> Result<()> {
    let mut args = Arguments::from_env();
    
    let options = Options {
        detailed: args.contains(["-d", "--detailed"]),
        compare: args.contains(["-c", "--compare"]),
        save_baseline: args.contains(["-s", "--save-baseline"]),
        target: args.value_from_str(["-t", "--target"]).unwrap_or_else(|_| "oxlint".to_string()),
        features: args
            .values_from_str::<_, String>(["-f", "--features"])
            .unwrap_or_default(),
        format: if args.contains("--json") {
            OutputFormat::Json
        } else {
            OutputFormat::Human
        },
    };

    let report = analyze_binary(&options)?;
    
    match options.format {
        OutputFormat::Human => print_human_report(&report, &options),
        OutputFormat::Json => print_json_report(&report)?,
    }
    
    if options.save_baseline {
        save_baseline(&report, &options)?;
    }
    
    if options.compare {
        compare_with_baseline(&report, &options)?;
    }
    
    Ok(())
}

fn analyze_binary(options: &Options) -> Result<BinarySizeReport> {
    let root = project_root();
    
    // Build the binary first
    println!("{}", "Building binary...".bold().cyan());
    build_binary(options)?;
    
    let binary_path = get_binary_path(&root, &options.target);
    
    println!("{}", "Analyzing binary size...".bold().cyan());
    
    // Get basic file info
    let metadata = fs::metadata(&binary_path)
        .into_diagnostic()
        .with_context(|| format!("Failed to get metadata for {}", binary_path.display()))?;
    
    let file_size = metadata.len();
    
    // Get stripped size
    let stripped_size = get_stripped_size(&binary_path)?;
    
    // Get text section size using objdump or similar
    let text_section_size = get_text_section_size(&binary_path)?;
    
    // Run cargo-bloat analysis
    let bloat_analysis = run_bloat_analysis(options)?;
    
    // Run cargo-llvm-lines analysis  
    let llvm_lines_analysis = run_llvm_lines_analysis(options)?;
    
    // Analyze dependencies
    let dependency_analysis = analyze_dependencies(options)?;
    
    // Get build info
    let build_info = get_build_info()?;
    
    Ok(BinarySizeReport {
        binary_path: binary_path.to_string_lossy().to_string(),
        file_size,
        stripped_size,
        text_section_size,
        bloat_analysis,
        llvm_lines_analysis,
        dependency_analysis,
        features_enabled: options.features.clone(),
        build_info,
        timestamp: Utc::now().to_rfc3339(),
    })
}

fn build_binary(options: &Options) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args(["build", "--release"]);
    
    if !options.target.is_empty() {
        cmd.args(["-p", &options.target]);
    }
    
    if !options.features.is_empty() {
        cmd.arg("--features");
        cmd.arg(options.features.join(","));
    }
    
    // Add debug symbols for analysis
    cmd.env("RUSTFLAGS", "-C debuginfo=2 -C strip=none");
    
    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .into_diagnostic()
        .context("Failed to execute cargo build")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(miette::miette!("Build failed: {}", stderr));
    }
    
    Ok(())
}

fn get_binary_path(root: &Path, target: &str) -> PathBuf {
    root.join("target/release").join(target)
}

fn get_stripped_size(binary_path: &Path) -> Result<u64> {
    // Create a temporary stripped copy
    let temp_dir = tempfile::tempdir().into_diagnostic()?;
    let stripped_path = temp_dir.path().join("stripped");
    
    // Copy original binary
    fs::copy(binary_path, &stripped_path).into_diagnostic()?;
    
    // Strip it
    let output = Command::new("strip")
        .arg(&stripped_path)
        .output()
        .into_diagnostic()
        .context("Failed to run strip command")?;
    
    if !output.status.success() {
        return Err(miette::miette!("Strip command failed"));
    }
    
    let metadata = fs::metadata(&stripped_path).into_diagnostic()?;
    Ok(metadata.len())
}

fn get_text_section_size(binary_path: &Path) -> Result<u64> {
    let output = Command::new("objdump")
        .args(["-h", &binary_path.to_string_lossy()])
        .output()
        .into_diagnostic()
        .context("Failed to run objdump")?;
    
    if !output.status.success() {
        return Err(miette::miette!("objdump failed"));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Parse .text section size
    let re = Regex::new(r"\s+\d+\s+\.text\s+([0-9a-fA-F]+)").unwrap();
    if let Some(captures) = re.captures(&stdout) {
        if let Some(size_hex) = captures.get(1) {
            return u64::from_str_radix(size_hex.as_str(), 16)
                .into_diagnostic()
                .context("Failed to parse text section size");
        }
    }
    
    Err(miette::miette!("Could not find .text section size"))
}

fn run_bloat_analysis(options: &Options) -> Result<BloatAnalysis> {
    // Check if cargo-bloat is available
    if Command::new("cargo").arg("bloat").arg("--help").output().is_err() {
        return Err(miette::miette!("cargo-bloat is not installed. Install with: cargo install cargo-bloat"));
    }
    
    let root = project_root();
    let target_dir = if options.target == "oxlint" {
        root.join("apps/oxlint")
    } else {
        root.clone()
    };
    
    // Get largest functions
    let mut cmd = Command::new("cargo");
    cmd.current_dir(&target_dir);
    cmd.args(["bloat", "--release", "-n", "15"]);
    
    if !options.target.is_empty() && options.target != "oxlint" {
        cmd.args(["-p", &options.target]);
    }
    
    if !options.features.is_empty() {
        cmd.arg("--features");
        cmd.arg(options.features.join(","));
    }
    
    cmd.env("RUSTFLAGS", "-C debuginfo=2 -C strip=none");
    
    let output = cmd.output().into_diagnostic()?;
    let functions_output = String::from_utf8_lossy(&output.stdout);
    
    // Get largest crates
    let mut cmd = Command::new("cargo");
    cmd.current_dir(&target_dir);
    cmd.args(["bloat", "--release", "--crates", "-n", "15"]);
    
    if !options.target.is_empty() && options.target != "oxlint" {
        cmd.args(["-p", &options.target]);
    }
    
    if !options.features.is_empty() {
        cmd.arg("--features");
        cmd.arg(options.features.join(","));
    }
    
    cmd.env("RUSTFLAGS", "-C debuginfo=2 -C strip=none");
    
    let output = cmd.output().into_diagnostic()?;
    let crates_output = String::from_utf8_lossy(&output.stdout);
    
    Ok(BloatAnalysis {
        largest_functions: parse_bloat_output(&functions_output, false),
        largest_crates: parse_bloat_output(&crates_output, true),
    })
}

fn parse_bloat_output(output: &str, is_crates: bool) -> Vec<BloatItem> {
    let mut items = Vec::new();
    
    for line in output.lines() {
        // Skip header and footer lines
        if line.contains("File") || line.contains("---") || line.trim().is_empty() {
            continue;
        }
        
        // Parse lines like: "0.1%   1.7% 104.6KiB regex_automata regex_automata::meta::strategy::new"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            if let Ok(percentage) = parts[1].trim_end_matches('%').parse::<f64>() {
                let size_str = parts[2];
                let size = parse_size_string(size_str);
                let name = if is_crates {
                    parts[3].to_string()
                } else {
                    parts[4..].join(" ")
                };
                
                items.push(BloatItem {
                    name,
                    size,
                    percentage,
                });
            }
        }
    }
    
    items
}

fn parse_size_string(size_str: &str) -> u64 {
    let size_str = size_str.to_lowercase();
    let (number_str, unit) = if size_str.ends_with("kib") {
        (size_str.trim_end_matches("kib"), 1024)
    } else if size_str.ends_with("mib") {
        (size_str.trim_end_matches("mib"), 1024 * 1024)
    } else if size_str.ends_with("gib") {
        (size_str.trim_end_matches("gib"), 1024 * 1024 * 1024)
    } else if size_str.ends_with("kb") {
        (size_str.trim_end_matches("kb"), 1000)
    } else if size_str.ends_with("mb") {
        (size_str.trim_end_matches("mb"), 1000 * 1000)
    } else if size_str.ends_with("gb") {
        (size_str.trim_end_matches("gb"), 1000 * 1000 * 1000)
    } else {
        (size_str.as_str(), 1)
    };
    
    number_str.parse::<f64>().unwrap_or(0.0) as u64 * unit
}

fn run_llvm_lines_analysis(options: &Options) -> Result<LlvmLinesAnalysis> {
    // Check if cargo-llvm-lines is available
    if Command::new("cargo").arg("llvm-lines").arg("--help").output().is_err() {
        return Err(miette::miette!("cargo-llvm-lines is not installed. Install with: cargo install cargo-llvm-lines"));
    }
    
    let root = project_root();
    let target_dir = if options.target == "oxlint" {
        root.join("apps/oxlint")
    } else {
        root.clone()
    };
    
    let mut cmd = Command::new("cargo");
    cmd.current_dir(&target_dir);
    cmd.args(["llvm-lines", "--release"]);
    
    if options.target == "oxlint" {
        cmd.args(["--bin", "oxlint"]);
    } else if !options.target.is_empty() {
        cmd.args(["-p", &options.target]);
    }
    
    if !options.features.is_empty() {
        cmd.arg("--features");
        cmd.arg(options.features.join(","));
    }
    
    let output = cmd.output().into_diagnostic()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(miette::miette!("cargo-llvm-lines failed: {}", stderr));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    Ok(parse_llvm_lines_output(&stdout))
}

fn parse_llvm_lines_output(output: &str) -> LlvmLinesAnalysis {
    let mut total_lines = 0;
    let mut top_functions = Vec::new();
    
    for line in output.lines() {
        if line.contains("TOTAL") {
            // Parse total line: "1867069                18001                (TOTAL)"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                total_lines = parts[0].parse().unwrap_or(0);
            }
            continue;
        }
        
        // Skip header lines
        if line.contains("Lines") || line.contains("-----") || line.trim().is_empty() {
            continue;
        }
        
        // Parse lines like: "30061 (1.6%,  1.6%)    119 (0.7%,  0.7%)  hashbrown::raw::RawTable<T,A>::reserve_rehash"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            if let (Ok(lines), Ok(copies)) = (parts[0].parse::<u64>(), parts[3].parse::<u64>()) {
                let name = parts[6..].join(" ");
                top_functions.push(LlvmLinesItem {
                    name,
                    lines,
                    copies,
                });
            }
        }
    }
    
    LlvmLinesAnalysis {
        total_lines,
        top_functions,
    }
}

fn analyze_dependencies(_options: &Options) -> Result<DependencyAnalysis> {
    // This is a simplified dependency analysis
    // In a real implementation, you might parse Cargo.lock and correlate with bloat analysis
    Ok(DependencyAnalysis {
        total_dependencies: 0,
        largest_dependencies: Vec::new(),
    })
}

fn get_build_info() -> Result<BuildInfo> {
    // This would normally parse the actual build configuration
    // For now, return the release profile settings from Cargo.toml
    Ok(BuildInfo {
        target_triple: std::env::consts::ARCH.to_string(),
        opt_level: "3".to_string(),
        debug_info: true,
        lto: "fat".to_string(),
        codegen_units: "1".to_string(),
        panic_strategy: "abort".to_string(),
    })
}

fn print_human_report(report: &BinarySizeReport, options: &Options) {
    println!();
    println!("{}", "=== Binary Size Analysis Report ===".bold().green());
    println!();
    
    // Basic size information
    println!("{}", "📊 Size Overview".bold().blue());
    println!("  Binary: {}", report.binary_path);
    println!("  File size: {}", format_size(report.file_size, DECIMAL).bold());
    println!("  Stripped size: {}", format_size(report.stripped_size, DECIMAL).bold());
    println!("  Text section: {}", format_size(report.text_section_size, DECIMAL).bold());
    
    let debug_overhead = report.file_size.saturating_sub(report.stripped_size);
    if debug_overhead > 0 {
        println!("  Debug overhead: {} ({:.1}%)", 
                format_size(debug_overhead, DECIMAL).red(),
                (debug_overhead as f64 / report.file_size as f64) * 100.0);
    }
    println!();
    
    // Build info
    println!("{}", "🔧 Build Configuration".bold().blue());
    println!("  Target: {}", report.build_info.target_triple);
    println!("  Optimization: Level {}", report.build_info.opt_level);
    println!("  LTO: {}", report.build_info.lto);
    println!("  Codegen units: {}", report.build_info.codegen_units);
    println!("  Panic strategy: {}", report.build_info.panic_strategy);
    
    if !report.features_enabled.is_empty() {
        println!("  Features: {}", report.features_enabled.join(", "));
    }
    println!();
    
    // Largest crates
    println!("{}", "📦 Largest Crates".bold().blue());
    for (i, item) in report.bloat_analysis.largest_crates.iter().take(10).enumerate() {
        let color = match i {
            0 => owo_colors::AnsiColors::Red,
            1..=2 => owo_colors::AnsiColors::Yellow,
            _ => owo_colors::AnsiColors::White,
        };
        println!("  {:2}. {} ({:.1}%) - {}", 
                i + 1, 
                item.name.color(color),
                item.percentage,
                format_size(item.size, DECIMAL));
    }
    println!();
    
    if options.detailed {
        // Largest functions
        println!("{}", "🔍 Largest Functions".bold().blue());
        for (i, item) in report.bloat_analysis.largest_functions.iter().take(15).enumerate() {
            println!("  {:2}. {} ({:.1}%) - {}", 
                    i + 1, 
                    truncate_function_name(&item.name, 60),
                    item.percentage,
                    format_size(item.size, DECIMAL));
        }
        println!();
        
        // LLVM Lines analysis
        println!("{}", "⚡ Generic Function Bloat (LLVM Lines)".bold().blue());
        println!("  Total LLVM lines: {}", report.llvm_lines_analysis.total_lines.to_string().bold());
        for (i, item) in report.llvm_lines_analysis.top_functions.iter().take(10).enumerate() {
            println!("  {:2}. {} ({} lines, {} copies)", 
                    i + 1,
                    truncate_function_name(&item.name, 50),
                    item.lines,
                    item.copies);
        }
        println!();
    }
    
    // Recommendations
    print_recommendations(report);
}

fn truncate_function_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        name.to_string()
    } else {
        format!("{}...", &name[..max_len.saturating_sub(3)])
    }
}

fn print_recommendations(report: &BinarySizeReport) {
    println!("{}", "💡 Size Reduction Recommendations".bold().yellow());
    
    let debug_overhead = report.file_size.saturating_sub(report.stripped_size);
    let debug_percentage = (debug_overhead as f64 / report.file_size as f64) * 100.0;
    
    if debug_percentage > 50.0 {
        println!("  🎯 Strip debug symbols: Could save {} ({:.1}%)", 
                format_size(debug_overhead, DECIMAL).bold().green(),
                debug_percentage);
    }
    
    // Analyze largest crates for specific recommendations
    for item in &report.bloat_analysis.largest_crates {
        match item.name.as_str() {
            "regex_automata" | "regex_syntax" if item.percentage > 3.0 => {
                println!("  🎯 Consider regex optimization: {} from regex crates ({:.1}%)", 
                        format_size(item.size, DECIMAL), item.percentage);
            }
            "miette" if item.percentage > 1.0 => {
                println!("  🎯 Consider simpler error handling: {} from miette ({:.1}%)", 
                        format_size(item.size, DECIMAL), item.percentage);
            }
            _ => {}
        }
    }
    
    // Check for generic bloat
    if report.llvm_lines_analysis.total_lines > 1000000 {
        println!("  🎯 Generic function bloat detected: {} LLVM lines generated", 
                report.llvm_lines_analysis.total_lines);
        
        for item in report.llvm_lines_analysis.top_functions.iter().take(3) {
            if item.copies > 50 {
                println!("    - {} instantiated {} times", 
                        truncate_function_name(&item.name, 40), item.copies);
            }
        }
    }
    
    println!("  🎯 Run 'cargo +nightly build -Z build-std=std,panic_abort --target x86_64-unknown-linux-gnu' for smaller std");
    println!("  🎯 Consider feature flags to conditionally compile large components");
    println!("  🎯 Profile-guided optimization (PGO) can help reduce size");
    println!();
}

fn print_json_report(report: &BinarySizeReport) -> Result<()> {
    let json = serde_json::to_string_pretty(report)
        .into_diagnostic()
        .context("Failed to serialize report to JSON")?;
    println!("{json}");
    Ok(())
}

fn save_baseline(report: &BinarySizeReport, options: &Options) -> Result<()> {
    let path = project_root().join("tasks/binary_size").join(format!("{}_baseline.json", options.target));
    let json = serde_json::to_string_pretty(report)
        .into_diagnostic()
        .context("Failed to serialize baseline")?;
    
    fs::write(&path, json)
        .into_diagnostic()
        .with_context(|| format!("Failed to write baseline to {}", path.display()))?;
    
    println!("{} Baseline saved to {}", "✅".green(), path.display());
    Ok(())
}

fn compare_with_baseline(report: &BinarySizeReport, options: &Options) -> Result<()> {
    let path = project_root().join("tasks/binary_size").join(format!("{}_baseline.json", options.target));
    
    if !path.exists() {
        println!("{} No baseline found at {}", "⚠️".yellow(), path.display());
        return Ok(());
    }
    
    let baseline_json = fs::read_to_string(&path)
        .into_diagnostic()
        .with_context(|| format!("Failed to read baseline from {}", path.display()))?;
    
    let baseline: BinarySizeReport = serde_json::from_str(&baseline_json)
        .into_diagnostic()
        .context("Failed to parse baseline JSON")?;
    
    println!();
    println!("{}", "📈 Comparison with Baseline".bold().cyan());
    println!();
    
    compare_sizes("File size", baseline.file_size, report.file_size);
    compare_sizes("Stripped size", baseline.stripped_size, report.stripped_size);
    compare_sizes("Text section", baseline.text_section_size, report.text_section_size);
    
    println!();
    Ok(())
}

fn compare_sizes(label: &str, baseline: u64, current: u64) {
    let diff = current as i64 - baseline as i64;
    let diff_percent = if baseline > 0 {
        (diff as f64 / baseline as f64) * 100.0
    } else {
        0.0
    };
    
    let (color, symbol) = if diff > 0 {
        (owo_colors::AnsiColors::Red, "📈")
    } else if diff < 0 {
        (owo_colors::AnsiColors::Green, "📉")
    } else {
        (owo_colors::AnsiColors::White, "📊")
    };
    
    println!("  {} {}: {} → {} ({:+} bytes, {:+.1}%)", 
            symbol,
            label,
            format_size(baseline, DECIMAL),
            format_size(current, DECIMAL).color(color),
            diff,
            diff_percent);
}