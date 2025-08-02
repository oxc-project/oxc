//! Common file and path utilities
//!
//! This module provides enhanced file operations and path utilities
//! used across different task binaries.

use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use crate::logging::print_file_operation;

/// Enhanced file operations with consistent error handling and logging
pub struct FileOperations;

impl FileOperations {
    /// Read a file to string with error handling
    pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String, io::Error> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)?;
        print_file_operation("Read", &path.display().to_string());
        Ok(content)
    }

    /// Write string to file with error handling and logging
    pub fn write_string<P: AsRef<Path>>(path: P, content: &str) -> Result<(), io::Error> {
        let path = path.as_ref();
        fs::write(path, content)?;
        print_file_operation("Wrote", &path.display().to_string());
        Ok(())
    }

    /// Create directory with parents if it doesn't exist
    pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<(), io::Error> {
        let path = path.as_ref();
        if !path.exists() {
            fs::create_dir_all(path)?;
            print_file_operation("Created dir", &path.display().to_string());
        }
        Ok(())
    }

    /// Copy file with logging
    pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<u64, io::Error> {
        let from = from.as_ref();
        let to = to.as_ref();
        let bytes = fs::copy(from, to)?;
        print_file_operation("Copied", &format!("{} -> {}", from.display(), to.display()));
        Ok(bytes)
    }

    /// Remove file with logging
    pub fn remove_file<P: AsRef<Path>>(path: P) -> Result<(), io::Error> {
        let path = path.as_ref();
        fs::remove_file(path)?;
        print_file_operation("Removed", &path.display().to_string());
        Ok(())
    }

    /// Remove directory recursively with logging
    pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> Result<(), io::Error> {
        let path = path.as_ref();
        fs::remove_dir_all(path)?;
        print_file_operation("Removed dir", &path.display().to_string());
        Ok(())
    }

    /// Check if file exists
    pub fn exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists()
    }
}

/// Common path operations
pub struct PathOperations;

impl PathOperations {
    /// Get relative path from project root
    pub fn relative_to_project<P: AsRef<Path>>(path: P) -> PathBuf {
        let path = path.as_ref();
        let project_root = crate::project_root();

        path.strip_prefix(&project_root).unwrap_or(path).to_path_buf()
    }

    /// Join paths safely
    pub fn join_safely<P: AsRef<Path>, Q: AsRef<Path>>(base: P, path: Q) -> PathBuf {
        base.as_ref().join(path)
    }

    /// Get parent directory, creating it if it doesn't exist
    pub fn ensure_parent_dir<P: AsRef<Path>>(path: P) -> Result<(), io::Error> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            FileOperations::create_dir_all(parent)?;
        }
        Ok(())
    }
}

/// Create a safe file writer that ensures parent directories exist
pub struct SafeFileWriter<W: Write> {
    writer: W,
}

impl<W: Write> SafeFileWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<W: Write> Write for SafeFileWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

/// Create a safe file writer for a path
pub fn create_safe_file<P: AsRef<Path>>(path: P) -> Result<SafeFileWriter<fs::File>, io::Error> {
    let path = path.as_ref();
    PathOperations::ensure_parent_dir(path)?;
    let file = fs::File::create(path)?;
    Ok(SafeFileWriter::new(file))
}
