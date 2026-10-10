/// This file is mostly copied from `tower-lsp-server` crate.
/// https://github.com/tower-lsp-community/ls-types/blob/a6feea6c723b3314d80c69e66b452978f62f6afb/src/uri.rs
/// The current version does not support this util functions anymore.
use percent_encoding::AsciiSet;
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    str::FromStr,
};
use tower_lsp_server::gen_lsp_types::Uri;

#[cfg(not(windows))]
pub use std::fs::canonicalize as strict_canonicalize;

const ASCII_SET: AsciiSet =
    // RFC3986 allows only alphanumeric characters, `-`, `.`, `_`, and `~` in the path.
    percent_encoding::NON_ALPHANUMERIC
        .remove(b'-')
        .remove(b'.')
        .remove(b'_')
        .remove(b'~')
        // we do not want path separators to be percent-encoded
        .remove(b'/');

/// On Windows, rewrites the wide path prefix `\\?\C:` to `C:`
/// Source: https://stackoverflow.com/a/70970317
#[inline]
#[cfg(windows)]
fn strict_canonicalize<P: AsRef<Path>>(path: P) -> std::io::Result<PathBuf> {
    use std::io;

    fn impl_(path: PathBuf) -> std::io::Result<PathBuf> {
        let head = path.components().next().ok_or(io::Error::other("empty path"))?;
        let disk_;
        let head = if let std::path::Component::Prefix(prefix) = head {
            if let std::path::Prefix::VerbatimDisk(disk) = prefix.kind() {
                disk_ = format!("{}:", disk as char);
                Path::new(&disk_)
                    .components()
                    .next()
                    .ok_or(io::Error::other("failed to parse disk component"))?
            } else {
                head
            }
        } else {
            head
        };
        Ok(std::iter::once(head).chain(path.components().skip(1)).collect())
    }

    let canon = std::fs::canonicalize(path)?;
    impl_(canon)
}

#[cfg(windows)]
fn capitalize_drive_letter(path: &str) -> String {
    // Check if it's a Windows path starting with a drive letter like "c:/"
    if path.len() >= 2 && path.chars().nth(1) == Some(':') {
        let mut chars = path.chars();
        let drive_letter = chars.next().unwrap().to_ascii_uppercase();
        let rest: String = chars.collect();
        format!("{}{}", drive_letter, rest)
    } else {
        path.to_string()
    }
}

/// Assuming the URL is in the `file` scheme or similar,
/// convert its path to an absolute `std::path::Path`.
///
/// **Note:** This does not actually check the URL’s `scheme`, and may
/// give nonsensical results for other schemes. It is the user’s
/// responsibility to check the URL’s scheme before calling this.
///
/// e.g. `Uri("file:///etc/passwd")` becomes `PathBuf("/etc/passwd")`
pub fn uri_to_file_path(uri: &Uri) -> Option<Cow<'_, Path>> {
    let path_str = uri.path().decode().to_string_lossy();
    if path_str.is_empty() {
        return None;
    }

    let path = match path_str {
        Cow::Borrowed(ref_) => Cow::Borrowed(Path::new(ref_)),
        Cow::Owned(owned) => Cow::Owned(PathBuf::from(owned)),
    };

    if cfg!(windows) {
        let auth_host = uri.authority().map(|auth| auth.host()).unwrap_or_default();

        if auth_host.is_empty() {
            // very high chance this is a `file:///c:/...` uri
            // in which case the path will include a leading slash we
            // need to remove to get `c:/...`
            let host = path.to_string_lossy();
            let host = host.get(1..)?;
            return Some(Cow::Owned(PathBuf::from(host)));
        }

        Some(Cow::Owned(
            // `file://server/...` becomes `server:/`
            Path::new(&format!("{auth_host}:")).components().chain(path.components()).collect(),
        ))
    } else {
        Some(path)
    }
}

/// Convert a file path to a [`Uri`].
///
/// Returns `None` if a relative path cannot be canonicalized.
pub fn file_path_to_uri<A: AsRef<Path>>(path: A) -> Option<Uri> {
    let path = path.as_ref();

    let fragment = if path.is_absolute() {
        Cow::Borrowed(path)
    } else {
        match strict_canonicalize(path) {
            Ok(path) => Cow::Owned(path),
            Err(_) => return None,
        }
    };

    #[cfg(windows)]
    let raw_uri = {
        // we want to parse a triple-slash path for Windows paths
        // it's a shorthand for `file://localhost/C:/Windows` with the `localhost` omitted.
        // We encode the driver Letter `C:` as well. LSP Specification allows it.
        // https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#uri
        format!(
            "file:///{}",
            percent_encoding::utf8_percent_encode(
                &capitalize_drive_letter(&fragment.to_string_lossy().replace('\\', "/")),
                &ASCII_SET
            )
        )
    };

    #[cfg(not(windows))]
    let raw_uri = {
        format!(
            "file://{}",
            percent_encoding::utf8_percent_encode(&fragment.to_string_lossy(), &ASCII_SET)
        )
    };

    Uri::from_str(&raw_uri).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    fn with_schema(path: &str) -> String {
        const EXPECTED_SCHEMA: &str = if cfg!(windows) { "file:///" } else { "file://" };
        format!("{EXPECTED_SCHEMA}{path}")
    }

    #[test]
    #[cfg(unix)]
    fn test_path_roundtrip_conversion() {
        let sources = [
            PathBuf::from("/some/path/to/file.txt"),
            PathBuf::from("/some/path/to/file with spaces.txt"),
            PathBuf::from("/some/path/[[...rest]]/file.txt"),
            PathBuf::from("/some/path/to/файл.txt"),
            PathBuf::from("/some/path/to/文件.txt"),
        ];

        for source in sources {
            let conv = file_path_to_uri(&source).unwrap();
            let roundtrip = uri_to_file_path(&conv).unwrap();
            assert_eq!(source, roundtrip, "conv={conv:?}");
        }
    }

    #[test]
    #[cfg(windows)]
    fn test_path_roundtrip_conversion() {
        let sources = [
            PathBuf::from("C:\\some\\path\\to\\file.txt"),
            PathBuf::from("C:\\some\\path\\to\\file with spaces.txt"),
            PathBuf::from("C:\\some\\path\\[[...rest]]\\file.txt"),
            PathBuf::from("C:\\some\\path\\to\\файл.txt"),
            PathBuf::from("C:\\some\\path\\to\\文件.txt"),
        ];

        for source in sources {
            let conv = file_path_to_uri(&source).unwrap();
            let roundtrip = uri_to_file_path(&conv).unwrap();
            assert_eq!(source, roundtrip, "conv={conv:?}");
        }
    }

    #[test]
    #[cfg(windows)]
    fn test_windows_uri_roundtrip_conversion() {
        use std::str::FromStr;

        let uris = [
            Uri::from_str("file:///C:/some/path/to/file.txt").unwrap(),
            Uri::from_str("file:///c:/some/path/to/file.txt").unwrap(),
            Uri::from_str("file:///c%3A/some/path/to/file.txt").unwrap(),
        ];

        let final_uri = Uri::from_str("file:///C%3A/some/path/to/file.txt").unwrap();

        for uri in uris {
            let path = uri_to_file_path(&uri).unwrap();
            assert_eq!(&path, Path::new("C:\\some\\path\\to\\file.txt"), "uri={uri:?}");

            let conv = file_path_to_uri(&path).unwrap();

            assert_eq!(final_uri, conv, "path={path:?} left={final_uri} right={conv}",);
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_path_to_uri() {
        let paths = [
            PathBuf::from("/some/path/to/file.txt"),
            PathBuf::from("/some/path/to/file with spaces.txt"),
            PathBuf::from("/some/path/[[...rest]]/file.txt"),
            PathBuf::from("/some/path/to/файл.txt"),
            PathBuf::from("/some/path/to/文件.txt"),
        ];

        let expected = [
            with_schema("/some/path/to/file.txt"),
            with_schema("/some/path/to/file%20with%20spaces.txt"),
            with_schema("/some/path/%5B%5B...rest%5D%5D/file.txt"),
            with_schema("/some/path/to/%D1%84%D0%B0%D0%B9%D0%BB.txt"),
            with_schema("/some/path/to/%E6%96%87%E4%BB%B6.txt"),
        ];

        for (path, expected) in paths.iter().zip(expected) {
            let uri = file_path_to_uri(path).unwrap();
            assert_eq!(uri.to_string(), expected);
        }
    }

    #[test]
    #[cfg(windows)]
    fn test_path_to_uri_windows() {
        let paths = [
            PathBuf::from("C:\\some\\path\\to\\file.txt"),
            PathBuf::from("C:\\some\\path\\to\\file with spaces.txt"),
            PathBuf::from("C:\\some\\path\\[[...rest]]\\file.txt"),
            PathBuf::from("C:\\some\\path\\to\\файл.txt"),
            PathBuf::from("C:\\some\\path\\to\\文件.txt"),
        ];

        // yes we encode `:` too, LSP allows it
        // https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#uri
        let expected = [
            with_schema("C%3A/some/path/to/file.txt"),
            with_schema("C%3A/some/path/to/file%20with%20spaces.txt"),
            with_schema("C%3A/some/path/%5B%5B...rest%5D%5D/file.txt"),
            with_schema("C%3A/some/path/to/%D1%84%D0%B0%D0%B9%D0%BB.txt"),
            with_schema("C%3A/some/path/to/%E6%96%87%E4%BB%B6.txt"),
        ];

        for (path, expected) in paths.iter().zip(expected) {
            let uri = file_path_to_uri(path).unwrap();
            assert_eq!(uri.to_string(), expected);
        }
    }

    #[test]
    fn test_invalid_uri_on_windows() {
        let uri = Uri::from_str("file://").unwrap();
        let path = uri_to_file_path(&uri);
        assert!(path.is_none());
    }
}
