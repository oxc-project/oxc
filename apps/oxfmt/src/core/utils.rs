use std::{
    fs, io,
    io::Write,
    path::{Component, Path, PathBuf},
};

/// Returns the value of the `VP_VERSION` environment variable, if set.
pub fn vp_version() -> Option<std::ffi::OsString> {
    std::env::var_os("VP_VERSION")
}

/// Initialize global tracing subscriber for `oxfmt`.
///
/// Safe to call multiple times from different NAPI entry points
/// (`run_cli()` or `format()` -> `js_text_to_doc()`) and worker processes.
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

/// Run a closure that blocks on JS callbacks (`block_on` in `external_formatter.rs`)
/// from within an async (NAPI) context.
///
/// On native targets, wrap with `block_in_place` so the multi-threaded tokio runtime
/// can keep scheduling other tasks while this thread blocks.
/// On wasm, tokio only provides the current-thread runtime and `block_in_place` does not exist;
/// JS callback bridging there does not rely on the tokio runtime (see `block_on` below),
/// so a direct call is fine.
#[cfg(feature = "napi")]
pub fn run_blocking<T>(f: impl FnOnce() -> T) -> T {
    #[cfg(not(target_family = "wasm"))]
    {
        tokio::task::block_in_place(f)
    }
    #[cfg(target_family = "wasm")]
    {
        f()
    }
}

// `block_on` for JS (TSFN) futures.
// On native targets, NAPI-RS's implementation drives the future on its multi-threaded
// tokio runtime. On wasm that runtime is current-thread and its core is held by the
// thread driving the NAPI async fn for its entire duration, so napi's `block_on` would
// deadlock when called concurrently from rayon threads. TSFN futures are channel-based
// and do not need a tokio driver, so a plain thread-parking executor is enough there.
#[cfg(all(feature = "napi", target_family = "wasm"))]
pub use futures::executor::block_on;
#[cfg(all(feature = "napi", not(target_family = "wasm")))]
pub use napi::bindgen_prelude::block_on;

/// WASI has no process-level cwd; wasi-libc emulates one starting at `/`.
/// Sync it once from the host `PWD` env var (snapshotted by the napi wasi glue at
/// instantiation; shells — including WebContainers' — set it) so that relative path
/// normalization in the `format()` API behaves like on native.
/// No-op on native targets.
#[cfg(all(feature = "napi", target_family = "wasm"))]
pub fn init_wasi_cwd() {
    use std::sync::OnceLock;

    static CWD_INIT: OnceLock<()> = OnceLock::new();
    CWD_INIT.get_or_init(|| {
        if let Ok(pwd) = std::env::var("PWD") {
            let _ = std::env::set_current_dir(pwd);
        }
    });
}
#[cfg(all(feature = "napi", not(target_family = "wasm")))]
pub fn init_wasi_cwd() {}

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
        if matches!(
            error.kind(),
            ErrorKind::Interrupted | ErrorKind::BrokenPipe | ErrorKind::WouldBlock
        ) {
            Ok(())
        } else {
            Err(error)
        }
    }

    writer.write_all(message.as_bytes()).or_else(check_for_writer_error).unwrap();
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
