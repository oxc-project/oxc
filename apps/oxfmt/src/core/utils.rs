use std::{
    fs, io,
    io::Write,
    path::{Path, PathBuf},
};

/// To debug `oxc_formatter`:
/// `OXC_LOG=oxc_formatter oxfmt`
/// # Panics
pub fn init_tracing() {
    use tracing_subscriber::{filter::Targets, fmt::format::FmtSpan, prelude::*};

    // Usage without the `regex` feature.
    // <https://github.com/tokio-rs/tracing/issues/1436#issuecomment-918528013>
    tracing_subscriber::registry()
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
        .init();
}

pub fn read_to_string(path: &Path) -> io::Result<String> {
    // `simdutf8` is faster than `std::str::from_utf8` which `fs::read_to_string` uses internally
    let bytes = fs::read(path)?;
    if simdutf8::basic::from_utf8(&bytes).is_err() {
        // Same error as `fs::read_to_string` produces (using `io::ErrorKind::InvalidData`)
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "stream did not contain valid UTF-8",
        ));
    }
    // SAFETY: `simdutf8` has ensured it's a valid UTF-8 string
    Ok(unsafe { String::from_utf8_unchecked(bytes) })
}

pub fn print_and_flush(writer: &mut dyn Write, message: &str) {
    use std::io::{Error, ErrorKind};
    fn check_for_writer_error(error: Error) -> Result<(), Error> {
        // Do not panic when the process is killed (e.g. piping into `less`).
        if matches!(error.kind(), ErrorKind::Interrupted | ErrorKind::BrokenPipe) {
            Ok(())
        } else {
            Err(error)
        }
    }

    writer.write_all(message.as_bytes()).or_else(check_for_writer_error).unwrap();
    writer.flush().unwrap();
}

/// Normalize a relative path by stripping `./` prefix and joining with `cwd`.
/// This ensures consistent path format and avoids issues with relative paths.
pub fn normalize_relative_path(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }

    if let Ok(stripped) = path.strip_prefix("./") { cwd.join(stripped) } else { cwd.join(path) }
}
