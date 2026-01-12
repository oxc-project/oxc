use std::{fs, io, io::Write, path::Path};

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

pub fn print_and_flush(writer: &mut dyn Write, message: &str) {
    use std::io::ErrorKind;

    const MAX_RETRIES: u32 = 1000;

    let mut bytes = message.as_bytes();
    let mut retries = 0;
    while !bytes.is_empty() {
        match writer.write(bytes) {
            Ok(0) => break, // EOF
            Ok(n) => {
                bytes = &bytes[n..];
                retries = 0; // Reset on progress
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                retries += 1;
                assert!(retries < MAX_RETRIES, "Failed to write: too many WouldBlock retries");
                // Buffer full, yield and retry
                std::thread::yield_now();
            }
            Err(e) if e.kind() == ErrorKind::Interrupted => {}
            Err(e) if e.kind() == ErrorKind::BrokenPipe => break,
            Err(e) => panic!("Failed to write: {e} bytes: {}", bytes.len()),
        }
    }

    let mut retries = 0;
    loop {
        match writer.flush() {
            Ok(()) => break,
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                retries += 1;
                assert!(retries < MAX_RETRIES, "Failed to flush: too many WouldBlock retries");
                std::thread::yield_now();
            }
            Err(e) if e.kind() == ErrorKind::Interrupted => {}
            Err(e) if e.kind() == ErrorKind::BrokenPipe => break,
            Err(e) => panic!("Failed to flush: {e}"),
        }
    }
}
