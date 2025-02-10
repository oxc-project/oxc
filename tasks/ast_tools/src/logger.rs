use std::sync::OnceLock;

static LOG: OnceLock<bool> = OnceLock::new();

/// Disable logging.
pub(super) fn quiet() {
    LOG.set(false).expect("Failed to disable logger");
}

pub(super) fn __internal_log_enable() -> bool {
    *LOG.get_or_init(|| true)
}

/// Log a message to stdout.
///
/// Does not include a trailing newline.
macro_rules! log {
    ($fmt:literal $(, $args:expr)*) => {
        if $crate::logger::__internal_log_enable() {
            print!($fmt$(, $args)*);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }
}
pub(crate) use log;

/// Log a message to stdout.
///
/// Includes a trailing newline.
macro_rules! logln {
    ($fmt:literal $(, $args:expr)*) => {
        if $crate::logger::__internal_log_enable() {
            println!($fmt$(, $args)*);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }
}
pub(crate) use logln;

/// Log "Success".
macro_rules! log_success {
    () => {
        $crate::log!("Done!\n");
    };
}
pub(crate) use log_success;

/// Log "FAILED".
macro_rules! log_failed {
    () => {
        $crate::log!("FAILED\n");
    };
}
pub(crate) use log_failed;

/// Log a [`Result`].
macro_rules! log_result {
    ($result:expr) => {
        match &($result) {
            Ok(_) => {
                $crate::log_success!();
            }
            Err(_) => {
                $crate::log_failed!();
            }
        }
    };
}
pub(crate) use log_result;
