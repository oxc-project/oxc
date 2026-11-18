use std::{
    fs, io,
    io::Write,
    path::{Component, Path, PathBuf},
};

/// Make stdio (stdin, stdout, stderr) blocking.
///
/// JS runtimes set non-TTY stdio to non-blocking mode:
///
/// - Node.js provides `process.std*._handle.setBlocking(true)` to undo it,
///   which the JS CLI entry point (`src-js/cli.ts`) calls before this runs.
/// - bun does not provide such an API (`process.std*._handle` is `undefined`)
///   and switches piped stdio to non-blocking once `process.stdout` is
///   accessed. Writes larger than the pipe buffer then return `EAGAIN` after a
///   partial write, which silently truncated `--stdin-filepath` output.
///
/// This is the runtime-agnostic equivalent of `setBlocking(true)`, so large
/// stdio writes never hit `EAGAIN` regardless of the JS runtime.
///
/// <https://github.com/oxc-project/oxc/issues/29846>
#[cfg(unix)]
pub fn set_blocking_stdio() {
    // Discard the return values: whether stdio was already blocking does not matter here.
    let _ = set_fd_blocking(libc::STDIN_FILENO);
    let _ = set_fd_blocking(libc::STDOUT_FILENO);
    let _ = set_fd_blocking(libc::STDERR_FILENO);
}

#[cfg(not(unix))]
pub fn set_blocking_stdio() {}

/// Restore blocking mode on a file descriptor if it is non-blocking.
///
/// Returns `true` when the descriptor was non-blocking and was switched to blocking mode.
#[cfg(unix)]
fn set_fd_blocking(fd: std::os::fd::RawFd) -> bool {
    // SAFETY: `F_GETFL`/`F_SETFL` have no preconditions beyond `fd` being open.
    // `-1` means the descriptor is closed (`EBADF`), leave it alone.
    unsafe {
        let flags = libc::fcntl(fd, libc::F_GETFL);
        if flags == -1 || flags & libc::O_NONBLOCK == 0 {
            return false;
        }
        libc::fcntl(fd, libc::F_SETFL, flags & !libc::O_NONBLOCK);
    }
    true
}

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

#[cfg(all(test, unix))]
mod tests {
    use std::os::fd::AsRawFd;

    use super::set_fd_blocking;

    /// `set_fd_blocking()` must clear `O_NONBLOCK` and be idempotent.
    ///
    /// `set_blocking_stdio()` applies it to stdio, so large stdio writes never
    /// return `EAGAIN` after a partial write.
    /// <https://github.com/oxc-project/oxc/issues/29846>
    #[test]
    fn set_fd_blocking_clears_non_blocking_mode() {
        let file = tempfile::tempfile().expect("create temp file");
        let fd = file.as_raw_fd();

        // SAFETY: `F_GETFL`/`F_SETFL` on an open file descriptor.
        unsafe {
            assert_eq!(libc::fcntl(fd, libc::F_SETFL, libc::O_NONBLOCK), 0);
            assert_ne!(libc::fcntl(fd, libc::F_GETFL) & libc::O_NONBLOCK, 0);
        }

        assert!(set_fd_blocking(fd), "non-blocking fd must be switched to blocking mode");

        // SAFETY: Same as above.
        unsafe {
            assert_eq!(libc::fcntl(fd, libc::F_GETFL) & libc::O_NONBLOCK, 0);
        }

        // Already blocking: no change reported, still blocking.
        assert!(!set_fd_blocking(fd));

        // Closed descriptors are left alone without panicking.
        drop(file);
        assert!(!set_fd_blocking(fd));
    }
}
