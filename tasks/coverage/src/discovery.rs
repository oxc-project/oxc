use std::{
    cell::OnceCell,
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use encoding_rs::UTF_16LE;
use encoding_rs_io::DecodeReaderBytesBuilder;
use walkdir::WalkDir;

use crate::workspace_root;

/// Discovers test files from a directory
pub struct FileDiscovery {
    test_root: PathBuf,
}

impl FileDiscovery {
    pub fn new(test_root: PathBuf) -> Self {
        Self { test_root }
    }

    /// Discover test file paths without reading file contents
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the test suite (for submodule initialization message)
    /// * `filter` - Optional filter string to match against file paths
    /// * `skip_path` - Function to determine if a path should be skipped
    ///
    /// # Returns
    ///
    /// Vector of PathBuf containing discovered file paths (relative to workspace root)
    pub fn discover_paths<F>(&self, name: &str, filter: Option<&str>, skip_path: &F) -> Vec<PathBuf>
    where
        F: Fn(&Path) -> bool,
    {
        let workspace = workspace_root();
        let cases_path = workspace.join(&self.test_root);

        let get_paths = || {
            WalkDir::new(&cases_path)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| !e.file_type().is_dir())
                .filter(|e| e.file_name() != ".DS_Store")
                .map(|e| e.path().to_owned())
                .filter(|path| !skip_path(path))
                .filter(|path| filter.is_none_or(|query| path.to_string_lossy().contains(query)))
                .map(|path| path.strip_prefix(&workspace).unwrap().to_owned())
                .collect::<Vec<_>>()
        };

        let mut paths = get_paths();

        // Initialize git submodule if it is empty and no filter is provided
        if paths.is_empty() && filter.is_none() {
            println!("-------------------------------------------------------");
            println!("git submodule is empty for {name}");
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
    }

    /// Read a file with support for UTF-16 encoding (TypeScript tests)
    ///
    /// # Arguments
    ///
    /// * `path` - Relative path from workspace root
    ///
    /// # Returns
    ///
    /// The file contents as a String
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read
    pub fn read_file(&self, path: &Path) -> std::io::Result<String> {
        let workspace = workspace_root();
        let full_path = workspace.join(path);

        // Try UTF-8 first
        let mut code = fs::read_to_string(&full_path).or_else(|_| -> std::io::Result<String> {
            // TypeScript tests may contain UTF-16 encoding files
            let file = fs::File::open(&full_path)?;
            let mut content = String::new();
            DecodeReaderBytesBuilder::new()
                .encoding(Some(UTF_16LE))
                .build(file)
                .read_to_string(&mut content)?;
            Ok(content)
        })?;

        // Remove the Byte Order Mark in some of the TypeScript files
        if code.starts_with('\u{feff}') {
            code.remove(0);
        }

        Ok(code)
    }
}

/// Cache for discovered test file paths to avoid redundant directory walks
///
/// This cache stores only file paths (not contents) to minimize memory usage.
/// Files are read on-demand when creating test cases.
#[derive(Debug, Default)]
pub struct DiscoveryCache {
    test262_paths: OnceCell<Vec<PathBuf>>,
    babel_paths: OnceCell<Vec<PathBuf>>,
    typescript_paths: OnceCell<Vec<PathBuf>>,
    misc_paths: OnceCell<Vec<PathBuf>>,
}

impl DiscoveryCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get cached test262 file paths, discovering them on first access
    pub fn get_test262_paths<F>(&self, filter: Option<&str>, skip_path: &F) -> &[PathBuf]
    where
        F: Fn(&Path) -> bool,
    {
        self.test262_paths.get_or_init(|| {
            FileDiscovery::new(PathBuf::from("test262/test"))
                .discover_paths("test262", filter, skip_path)
        })
    }

    /// Get cached babel file paths, discovering them on first access
    pub fn get_babel_paths<F>(&self, filter: Option<&str>, skip_path: &F) -> &[PathBuf]
    where
        F: Fn(&Path) -> bool,
    {
        self.babel_paths.get_or_init(|| {
            FileDiscovery::new(PathBuf::from("babel/packages/babel-parser/test/fixtures"))
                .discover_paths("babel", filter, skip_path)
        })
    }

    /// Get cached typescript file paths, discovering them on first access
    pub fn get_typescript_paths<F>(&self, filter: Option<&str>, skip_path: &F) -> &[PathBuf]
    where
        F: Fn(&Path) -> bool,
    {
        self.typescript_paths.get_or_init(|| {
            FileDiscovery::new(PathBuf::from("typescript/tests/cases")).discover_paths(
                "typescript",
                filter,
                skip_path,
            )
        })
    }

    /// Get cached misc file paths, discovering them on first access
    pub fn get_misc_paths<F>(&self, filter: Option<&str>, skip_path: &F) -> &[PathBuf]
    where
        F: Fn(&Path) -> bool,
    {
        self.misc_paths.get_or_init(|| {
            FileDiscovery::new(PathBuf::from("misc")).discover_paths("misc", filter, skip_path)
        })
    }
}
