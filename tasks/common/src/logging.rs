//! Common logging and console output utilities
//!
//! This module provides standardized logging patterns used across task binaries.

use console::Style;

/// Colors for consistent output styling
pub struct Colors {
    pub success: Style,
    pub error: Style,
    pub warning: Style,
    pub info: Style,
    pub dim: Style,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            success: Style::new().green(),
            error: Style::new().red(),
            warning: Style::new().yellow(),
            info: Style::new().blue(),
            dim: Style::new().dim(),
        }
    }
}

/// Global colors instance
pub static COLORS: std::sync::LazyLock<Colors> = std::sync::LazyLock::new(Colors::default);

/// Print a success message with green coloring
pub fn print_success(message: &str) {
    println!("{}", COLORS.success.apply_to(message));
}

/// Print an error message with red coloring
pub fn print_error(message: &str) {
    eprintln!("{}", COLORS.error.apply_to(message));
}

/// Print a warning message with yellow coloring
pub fn print_warning(message: &str) {
    println!("{}", COLORS.warning.apply_to(message));
}

/// Print an info message with blue coloring
pub fn print_info(message: &str) {
    println!("{}", COLORS.info.apply_to(message));
}

/// Print a dimmed message
pub fn print_dim(message: &str) {
    println!("{}", COLORS.dim.apply_to(message));
}

/// Print a progress message with optional success/error indication
pub fn print_progress(message: &str, success: Option<bool>) {
    match success {
        Some(true) => print_success(&format!("✓ {message}")),
        Some(false) => print_error(&format!("✗ {message}")),
        None => print_info(&format!("• {message}")),
    }
}

/// Print a file operation message
pub fn print_file_operation(operation: &str, path: &str) {
    print_dim(&format!("{operation}: {path}"));
}

/// Print download progress
pub fn print_download(filename: &str, url: &str, destination: &str) {
    print_info(&format!("[{filename}] - Downloading [{url}] to [{destination}]"));
}