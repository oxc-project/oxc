use std::{path::Path, str::FromStr};

use cow_utils::CowUtils;
use percent_encoding::AsciiSet;
use tower_lsp_server::lsp_types::Uri;

pub fn path_to_uri(path: &Path) -> Uri {
    let path = if cfg!(target_os = "windows") {
        // On Windows, we need to replace backslashes with forward slashes.
        // Tripleslash is a shorthand for `file://localhost/C:/Windows` with the `localhost` omitted.
        // We encode the driver Letter `C:` as well. LSP Specification allows it.
        // https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#uri
        format!(
            "file:///{}",
            percent_encoding::utf8_percent_encode(
                &path.to_string_lossy().cow_replace('\\', "/"),
                &ASCII_SET
            )
        )
    } else {
        // For Unix-like systems, just convert to a file URI directly
        format!(
            "file://{}",
            percent_encoding::utf8_percent_encode(&path.to_string_lossy(), &ASCII_SET)
        )
    };
    Uri::from_str(&path).expect("Failed to create URI from path")
}

const ASCII_SET: AsciiSet =
    // RFC3986 allows only alphanumeric characters, `-`, `.`, `_`, and `~` in the path.
    percent_encoding::NON_ALPHANUMERIC
        .remove(b'-')
        .remove(b'.')
        .remove(b'_')
        .remove(b'~')
        // we do not want path separators to be percent-encoded
        .remove(b'/');

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use crate::uri_ext::path_to_uri;

    const EXPECTED_SCHEMA: &str = if cfg!(target_os = "windows") { "file:///" } else { "file://" };

    fn with_schema(path: &str) -> String {
        format!("{EXPECTED_SCHEMA}{path}")
    }

    #[test]
    fn test_path_to_uri() {
        let path = PathBuf::from("/some/path/to/file.txt");
        let uri = path_to_uri(&path);
        assert_eq!(uri.to_string(), with_schema("/some/path/to/file.txt"));
    }

    #[test]
    fn test_path_to_uri_with_spaces() {
        let path = PathBuf::from("/some/path/to/file with spaces.txt");
        let uri = path_to_uri(&path);
        assert_eq!(uri.to_string(), with_schema("/some/path/to/file%20with%20spaces.txt"));
    }

    #[test]
    fn test_path_to_uri_with_special_characters() {
        let path = PathBuf::from("/some/path/[[...rest]]/file.txt");
        let uri = path_to_uri(&path);
        assert_eq!(uri.to_string(), with_schema("/some/path/%5B%5B...rest%5D%5D/file.txt"));
    }

    #[test]
    fn test_path_to_uri_non_ascii() {
        let path = PathBuf::from("/some/path/to/файл.txt");
        let uri = path_to_uri(&path);
        assert_eq!(uri.to_string(), with_schema("/some/path/to/%D1%84%D0%B0%D0%B9%D0%BB.txt"));
    }

    #[test]
    fn test_path_to_uri_with_unicode() {
        let path = PathBuf::from("/some/path/to/文件.txt");
        let uri = path_to_uri(&path);
        assert_eq!(uri.to_string(), with_schema("/some/path/to/%E6%96%87%E4%BB%B6.txt"));
    }

    #[cfg(all(test, target_os = "windows"))]
    #[test]
    fn test_path_to_uri_windows() {
        let path = PathBuf::from("C:\\some\\path\\to\\file.txt");
        let uri = path_to_uri(&path);
        // yes we encode `:` too, LSP allows it
        // https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#uri
        assert_eq!(uri.to_string(), with_schema("C%3A/some/path/to/file.txt"));
    }
}
