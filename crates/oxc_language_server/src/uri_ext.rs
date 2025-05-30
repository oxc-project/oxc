use std::{path::{Path, PathBuf}, str::FromStr};

use percent_encoding::AsciiSet;
use tower_lsp_server::lsp_types::Uri;

fn path_to_uri(path: &PathBuf) -> Uri {
    let path_str = normalize_path_with_utf8_percent_encode(path);
    Uri::from_str(&format!("file://{}", path_str.to_string_lossy())).expect("Failed to create URI from path")
}

const ASCII_SET: AsciiSet = percent_encoding::NON_ALPHANUMERIC.remove(b'.');

/// Normalize a path by removing `.` and resolving `..` components,
/// without touching the filesystem.
pub fn normalize_path_with_utf8_percent_encode<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut result = PathBuf::new();
    let components = path.as_ref().components();

    for component in components {
        match component {
            std::path::Component::Prefix(_) => {
                // Keep the prefix (e.g., drive letter on Windows)
                result.push(component.as_os_str());
            }
            std::path::Component::RootDir => {
                // Keep the root directory
                result.push(component.as_os_str());
            }
            std::path::Component::Normal(part) => {
                // Normal components are added to the path
                result.push(percent_encoding::utf8_percent_encode(&part.to_str().unwrap(), &ASCII_SET).to_string());
            }
            _ => {}
        }
    }

    result
}
#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use crate::uri_ext::path_to_uri;


    #[test]
    fn test_path_to_uri() {
        let path = PathBuf::from("/some/path/to/file.txt");
        let uri = path_to_uri(&path);
        assert_eq!(uri.to_string(), "file:///some/path/to/file.txt");
    }

    #[test]
    fn test_path_to_uri_with_spaces() {
        let path = PathBuf::from("/some/path/to/file with spaces.txt");
        let uri = path_to_uri(&path);
        assert_eq!(uri.to_string(), "file:///some/path/to/file%20with%20spaces.txt");
    }

    #[test]

    fn test_path_to_uri_with_special_characters() {
        let path = PathBuf::from("/some/path/[[...rest]]/file.txt");
        let uri = path_to_uri(&path);
        assert_eq!(uri.to_string(), "file:///some/path/%5B%5B...rest%5D%5D/file.txt");
    }
}
