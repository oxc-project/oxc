//! Enhanced HTTP utilities for remote resource handling
//!
//! This module extends the basic request functionality with caching,
//! retry logic, and better error handling.

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use ureq::Agent;

use crate::{
    fs::FileOperations,
    logging::{print_download, print_error, print_success},
    project_root,
    request::agent,
};

/// Configuration for HTTP operations
#[derive(Clone)]
pub struct HttpConfig {
    /// Cache directory for downloaded files
    pub cache_dir: PathBuf,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Timeout for requests
    pub timeout: Duration,
    /// Whether to use cache
    pub use_cache: bool,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            cache_dir: project_root().join("target").join("http_cache"),
            max_retries: 3,
            timeout: Duration::from_secs(30),
            use_cache: true,
        }
    }
}

/// Enhanced HTTP client with caching and retry logic
pub struct HttpClient {
    agent: Agent,
    config: HttpConfig,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new(HttpConfig::default())
    }
}

impl HttpClient {
    /// Create a new HTTP client with the given configuration
    pub fn new(config: HttpConfig) -> Self {
        Self { agent: agent(), config }
    }

    /// Download a file from URL with caching and retry logic
    pub fn download_file(&self, url: &str) -> Result<(String, String), String> {
        let filename = self.extract_filename(url)?;
        let cache_path = self.config.cache_dir.join(&filename);

        // Try to use cached version first
        if self.config.use_cache && cache_path.exists() {
            match FileOperations::read_to_string(&cache_path) {
                Ok(content) => {
                    print_success(&format!("Using cached file: {}", cache_path.display()));
                    return Ok((filename, content));
                }
                Err(e) => {
                    print_error(&format!("Failed to read cached file: {e}"));
                }
            }
        }

        // Download with retry logic
        self.download_with_retry(url, &filename, &cache_path)
    }

    /// Download file with retry logic
    fn download_with_retry(
        &self,
        url: &str,
        filename: &str,
        cache_path: &Path,
    ) -> Result<(String, String), String> {
        let mut last_error = String::new();

        for attempt in 1..=self.config.max_retries {
            print_download(filename, url, &cache_path.display().to_string());

            match self.agent.get(url).call() {
                Ok(mut response) => {
                    match self.save_response_to_cache(response.body_mut().as_reader(), cache_path) {
                        Ok(content) => {
                            print_success(&format!("Downloaded {filename} (attempt {attempt})"));
                            return Ok((filename.to_string(), content));
                        }
                        Err(e) => {
                            last_error = format!("Failed to save response: {e}");
                            print_error(&last_error);
                        }
                    }
                }
                Err(e) => {
                    last_error = format!("HTTP request failed: {e}");
                    if attempt < self.config.max_retries {
                        print_error(&format!("{last_error} (attempt {attempt}, retrying...)"));
                        std::thread::sleep(Duration::from_secs(attempt as u64));
                    }
                }
            }
        }

        Err(format!(
            "Failed to download {url} after {} attempts. Last error: {last_error}",
            self.config.max_retries
        ))
    }

    /// Save HTTP response to cache file - using same pattern as test_file.rs
    fn save_response_to_cache(
        &self,
        mut reader: impl std::io::Read,
        cache_path: &Path,
    ) -> Result<String, String> {
        // Ensure cache directory exists
        FileOperations::create_dir_all(&self.config.cache_dir)
            .map_err(|e| format!("Failed to create cache directory: {e}"))?;

        // Save response to file first, then read it back (same pattern as test_file.rs)
        let mut writer = std::fs::File::create(cache_path)
            .map_err(|e| format!("Failed to create cache file: {e}"))?;

        std::io::copy(&mut reader, &mut writer)
            .map_err(|e| format!("Failed to write response to cache: {e}"))?;

        // Read the content back as string
        let content = std::fs::read_to_string(cache_path)
            .map_err(|e| format!("Failed to read cached file: {e}"))?;

        Ok(content)
    }

    /// Extract filename from URL
    fn extract_filename(&self, url: &str) -> Result<String, String> {
        if !url.starts_with("https://") && !url.starts_with("http://") {
            return Err(format!("Invalid URL: {url}"));
        }

        url.split('/')
            .last()
            .filter(|name| !name.is_empty())
            .map(|name| {
                // Handle query parameters
                name.split('?').next().unwrap_or(name).to_string()
            })
            .ok_or_else(|| format!("Could not extract filename from URL: {url}"))
    }

    /// Clear the cache directory
    pub fn clear_cache(&self) -> Result<(), String> {
        if self.config.cache_dir.exists() {
            FileOperations::remove_dir_all(&self.config.cache_dir)
                .map_err(|e| format!("Failed to clear cache: {e}"))?;
        }
        Ok(())
    }
}

/// Convenience function to download a file with default settings
pub fn download_file(url: &str) -> Result<(String, String), String> {
    HttpClient::default().download_file(url)
}

/// Download multiple files concurrently
pub fn download_files(urls: &[&str]) -> Vec<Result<(String, String), String>> {
    use rayon::prelude::*;

    urls.par_iter().map(|url| download_file(url)).collect()
}
