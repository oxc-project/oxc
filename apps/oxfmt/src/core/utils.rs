use std::{
    fs, io,
    io::Write,
    path::{Component, Path, PathBuf},
};

use oxc_allocator::Allocator;
use oxc_parser::Source;
use oxc_span::SourceType;

/// Initialize global tracing subscriber for `oxfmt`.
///
/// Safe to call multiple times from different NAPI entry points
/// (`run_cli()` or `format()` -> `js_text_to_doc()`) and worker threads.
///
/// To debug `oxc_formatter`:
/// `OXC_LOG=oxc_formatter oxfmt`
///
/// # Panics
/// Panics when `OXC_LOG` is set but cannot be parsed as a valid
/// `tracing_subscriber::filter::Targets` expression.
pub fn init_tracing() {
    use std::sync::OnceLock;
    use tracing_subscriber::{filter::Targets, fmt::format::FmtSpan, prelude::*};

    static TRACING_INIT: OnceLock<()> = OnceLock::new();
    TRACING_INIT.get_or_init(|| {
        // Usage without the `regex` feature.
        // <https://github.com/tokio-rs/tracing/issues/1436#issuecomment-918528013>
        let _ = tracing_subscriber::registry()
            .with(std::env::var("OXC_LOG").map_or_else(
                |_| Targets::new(),
                |env_var| {
                    use std::str::FromStr;
                    Targets::from_str(&env_var).unwrap()
                },
            ))
            .with(
                tracing_subscriber::fmt::layer()
                    .with_thread_names(true)
                    .with_span_events(FmtSpan::CLOSE)
                    // https://github.com/tokio-rs/tracing/issues/2492
                    .with_writer(std::io::stderr),
            )
            .try_init();
    });
}

pub fn read_to_string(path: &Path) -> io::Result<String> {
    // `simdutf8` is faster than `std::str::from_utf8` which `fs::read_to_string` uses internally
    let bytes = fs::read(path)?;
    if simdutf8::basic::from_utf8(&bytes).is_err() {
        return Err(invalid_utf8_error());
    }
    // SAFETY: `simdutf8` has ensured it's a valid UTF-8 string
    Ok(unsafe { String::from_utf8_unchecked(bytes) })
}

/// `SourceType` to create a [`Source`] with, when it's going to be passed to
/// [`SourceFormatter::format`].
///
/// It's a placeholder. [`SourceFormatter::format`] does not use the `Source`'s `SourceType`.
/// It gets the language of the file, and for JS/TS its `SourceType`, from the `FormatPlan`.
///
/// [`SourceFormatter::format`]: super::SourceFormatter::format
pub const PLACEHOLDER_SOURCE_TYPE: SourceType = SourceType::mjs();

/// Read the file at `path` into a [`Source`], to pass to [`SourceFormatter::format`].
///
/// The file is read directly into `allocator`, without copying it again.
///
/// # Errors
/// Returns an error if the file cannot be read, or is not valid UTF-8.
///
/// [`SourceFormatter::format`]: super::SourceFormatter::format
pub fn read_source<'a>(path: &Path, allocator: &'a Allocator) -> io::Result<Source<'a>> {
    let source = Source::read_from_file_with_source_type(path, PLACEHOLDER_SOURCE_TYPE, allocator)?;
    ensure_utf8(source)
}

/// Check that `source` is valid UTF-8.
///
/// The `Source` records that it is, so the check is not repeated when it's parsed.
///
/// # Errors
/// Returns an error if `source` is not valid UTF-8.
pub fn ensure_utf8(mut source: Source<'_>) -> io::Result<Source<'_>> {
    if source.as_str().is_err() {
        return Err(invalid_utf8_error());
    }
    Ok(source)
}

/// Same error as `fs::read_to_string` produces (using `io::ErrorKind::InvalidData`).
fn invalid_utf8_error() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "stream did not contain valid UTF-8")
}

pub fn print_and_flush(writer: &mut dyn Write, message: &str) {
    print_bytes_and_flush(writer, message.as_bytes());
}

/// Same as [`print_and_flush`], but for bytes which are not necessarily valid UTF-8.
pub fn print_bytes_and_flush(writer: &mut dyn Write, bytes: &[u8]) {
    use std::io::{Error, ErrorKind};
    fn check_for_writer_error(error: Error) -> Result<(), Error> {
        // Do not panic when the process is killed (e.g. piping into `less`).
        if matches!(
            error.kind(),
            ErrorKind::Interrupted | ErrorKind::BrokenPipe | ErrorKind::WouldBlock
        ) {
            Ok(())
        } else {
            Err(error)
        }
    }

    writer.write_all(bytes).or_else(check_for_writer_error).unwrap();
    writer.flush().or_else(check_for_writer_error).unwrap();
}

/// Normalize a relative path by:
/// - stripping `./` prefix,
/// - joining with `cwd`,
/// - and resolving `.` and `..` components logically (without fs access)
///
/// This ensures consistent absolute path format,
/// which is required for gitignore-based pattern matching
/// (e.g., `ignorePatterns` resolution).
///
/// Unlike `fs::canonicalize()`,
/// this does not resolve symlinks and does not produce `\\?\` prefixed paths on Windows.
pub fn normalize_relative_path(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }

    let joined = if let Ok(stripped) = path.strip_prefix("./") {
        cwd.join(stripped)
    } else {
        cwd.join(path)
    };

    let mut result = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {}
            _ => {
                result.push(component.as_os_str());
            }
        }
    }

    result
}
