use std::{fs, io, path::Path};

use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter, stderr, stdout};

/// To debug `oxc_formatter`:
/// `OXC_LOG=oxc_formatter oxfmt`
/// # Panics
pub fn init_tracing() {
    use tracing_subscriber::{filter::Targets, prelude::*};

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

/// Prints a string to stdout with buffering.
pub async fn print_stdout(message: &str) {
    write_buffered(stdout(), message.as_bytes()).await;
}

/// Prints a string to stderr with buffering.
pub async fn print_stderr(message: &str) {
    write_buffered(stderr(), message.as_bytes()).await;
}

/// Writes bytes to stderr with buffering.
pub async fn write_stderr(message: &[u8]) {
    write_buffered(stderr(), message).await;
}

async fn write_buffered<W: AsyncWrite + Unpin>(writer: W, message: &[u8]) {
    // stdio is blocked by `LineWriter`, use a `BufWriter` to reduce syscalls.
    // See https://github.com/rust-lang/rust/issues/60673
    let mut writer = BufWriter::new(writer);
    let _ = writer.write_all(message).await;
    let _ = writer.flush().await;
}
