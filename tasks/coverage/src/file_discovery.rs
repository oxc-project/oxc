//! File discovery layer for conformance test suites
//!
//! # Overview
//!
//! This module provides centralized file discovery to avoid redundant directory traversals.
//! Instead of each Suite walking the directory tree independently, file discovery is performed
//! once per test root directory and the results are shared across multiple tool runs.
//!
//! # Architecture
//!
//! The file discovery system consists of three main components:
//!
//! - [`DiscoveredFiles`]: Container holding all discovered test files from a single test root
//! - [`DiscoveredFile`]: Individual test file with path and pre-loaded source code
//! - [`FileDiscoveryConfig`]: Configuration for customizing the discovery process
//!
//! # Performance Benefits
//!
//! Before this optimization, the conformance runner would:
//! 1. Walk directory tree 7 times per test suite (once per tool)
//! 2. Read each file 7 times (once per tool)
//!
//! With centralized file discovery:
//! 1. Walk directory tree 1 time per test suite
//! 2. Read each file 1 time per test suite
//! 3. Share discovered files across all tools (parser, semantic, codegen, etc.)
//!
//! For the Test262 suite with ~40,000 files, this eliminates:
//! - 6 redundant directory walks (saving ~240,000 stat calls)
//! - 6 redundant file reads (saving ~240,000 I/O operations)
//!
//! # Usage Example
//!
//! ```rust,ignore
//! use crate::file_discovery::{DiscoveredFiles, FileDiscoveryConfig};
//!
//! // Discover files once
//! let files = DiscoveredFiles::discover(&FileDiscoveryConfig {
//!     test_root: Path::new("tasks/coverage/test262"),
//!     filter: None,
//!     skip_test_path: Box::new(|path| path.extension() != Some("js")),
//!     skip_test_crawl: false,
//!     suite_name: "test262",
//! });
//!
//! // Share with multiple tools
//! ParserSuite::new().run_with_discovered_files("parser", &args, &files);
//! SemanticSuite::new().run_with_discovered_files("semantic", &args, &files);
//! CodegenSuite::new().run_with_discovered_files("codegen", &args, &files);
//! ```
//!
//! # Memory Management
//!
//! Files are discovered and stored in memory for the duration of a single suite's processing.
//! The `DiscoveredFiles` struct is dropped after all tools for that suite complete, allowing
//! the memory to be reclaimed before processing the next suite. This sequential processing
//! pattern keeps peak memory usage low while still eliminating redundant I/O.

use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use encoding_rs::UTF_16LE;
use encoding_rs_io::DecodeReaderBytesBuilder;
use walkdir::WalkDir;

use crate::workspace_root;

/// Discovered test files from a single test root directory.
///
/// This structure contains all test files discovered from a single test suite directory
/// (e.g., Test262, Babel, TypeScript). The files are stored with their paths and pre-loaded
/// source code, ready to be shared across multiple tool runs.
///
/// # Lifecycle
///
/// 1. Created via [`DiscoveredFiles::discover`] at the start of processing a suite
/// 2. Passed to multiple tools via `run_with_discovered_files`
/// 3. Dropped after all tools complete, reclaiming memory
#[derive(Debug, Clone)]
pub struct DiscoveredFiles {
    /// Root directory that was scanned
    #[expect(dead_code)]
    pub test_root: PathBuf,
    /// List of discovered file paths (relative to workspace root)
    pub files: Vec<DiscoveredFile>,
}

/// A single discovered test file with pre-loaded content.
///
/// Each file is discovered once and its content is read into memory. The file path
/// is stored relative to the workspace root for consistency across tools.
///
/// # Encoding Handling
///
/// - UTF-8 files are read directly
/// - UTF-16LE files (common in TypeScript tests) are automatically detected and decoded
/// - Byte Order Marks (BOM) are stripped if present
#[derive(Debug, Clone)]
pub struct DiscoveredFile {
    /// Path relative to workspace root
    pub path: PathBuf,
    /// Source code content (UTF-8)
    pub code: String,
}

/// Configuration for file discovery process.
///
/// This configuration allows customizing the file discovery behavior for each test suite,
/// including filtering, path skipping, and submodule initialization.
///
/// # Examples
///
/// ```rust,ignore
/// let config = FileDiscoveryConfig {
///     test_root: Path::new("tasks/coverage/test262"),
///     filter: Some("class-methods"),  // Only files matching this substring
///     skip_test_path: Box::new(|path| {
///         // Skip minified files
///         path.to_string_lossy().contains(".min.")
///     }),
///     skip_test_crawl: false,
///     suite_name: "test262",
/// };
/// ```
pub struct FileDiscoveryConfig<'a> {
    /// Test root directory relative to workspace
    pub test_root: &'a Path,
    /// Optional filter string to match against file paths
    pub filter: Option<&'a str>,
    /// Function to determine if a path should be skipped
    pub skip_test_path: Box<dyn Fn(&Path) -> bool + 'a>,
    /// Whether to skip directory crawl entirely (for empty suites)
    pub skip_test_crawl: bool,
    /// Name of the test suite (for submodule initialization messages)
    pub suite_name: &'a str,
}

impl DiscoveredFiles {
    /// Discover files from a test root directory.
    ///
    /// This method performs the following steps:
    /// 1. Walk the directory tree at `config.test_root`
    /// 2. Filter files based on `skip_test_path` and `filter`
    /// 3. Read all file contents (handling UTF-8 and UTF-16LE)
    /// 4. Initialize git submodules if directory is empty
    ///
    /// # Returns
    ///
    /// A [`DiscoveredFiles`] structure containing all discovered files with pre-loaded content.
    ///
    /// # Submodule Initialization
    ///
    /// If no files are found and no filter is applied, this method assumes the test suite
    /// git submodule is not initialized and automatically runs `just submodules` to clone it.
    ///
    /// # Performance
    ///
    /// This method is I/O intensive as it:
    /// - Walks the entire directory tree
    /// - Reads all discovered files into memory
    ///
    /// However, it only runs once per test suite, with results shared across all tools.
    pub fn discover(config: &FileDiscoveryConfig<'_>) -> Self {
        let test_path = workspace_root();
        let test_root = config.test_root.to_path_buf();

        let paths = if config.skip_test_crawl {
            vec![]
        } else {
            let cases_path = test_path.join(&test_root);

            let get_paths = || {
                WalkDir::new(&cases_path)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|e| !e.file_type().is_dir())
                    .filter(|e| e.file_name() != ".DS_Store")
                    .map(|e| e.path().to_owned())
                    .filter(|path| !(config.skip_test_path)(path))
                    .filter(|path| {
                        config.filter.is_none_or(|query| path.to_string_lossy().contains(query))
                    })
                    .collect::<Vec<_>>()
            };

            let mut paths = get_paths();

            // Initialize git submodule if it is empty and no filter is provided
            if paths.is_empty() && config.filter.is_none() {
                println!("-------------------------------------------------------");
                println!("git submodule is empty for {}", config.suite_name);
                println!("Running `just submodules` to clone the submodules");
                println!("This may take a while.");
                println!("-------------------------------------------------------");
                Command::new("just")
                    .args(["submodules"])
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .output()
                    .expect("failed to execute `just submodules`");
                paths = get_paths();
            }
            paths
        };

        // Read all files in parallel
        let files = paths
            .into_iter()
            .map(|path| {
                let mut code = fs::read_to_string(&path).unwrap_or_else(|_| {
                    // TypeScript tests may contain utf_16 encoding files
                    let file = fs::File::open(&path).unwrap();
                    let mut content = String::new();
                    DecodeReaderBytesBuilder::new()
                        .encoding(Some(UTF_16LE))
                        .build(file)
                        .read_to_string(&mut content)
                        .unwrap();
                    content
                });

                let path = path.strip_prefix(&test_path).unwrap().to_owned();
                // Remove the Byte Order Mark in some of the TypeScript files
                if code.starts_with('\u{feff}') {
                    code.remove(0);
                }

                DiscoveredFile { path, code }
            })
            .collect();

        Self { test_root, files }
    }
}
