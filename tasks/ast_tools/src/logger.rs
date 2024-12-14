use std::sync::OnceLock;

static LOG: OnceLock<bool> = OnceLock::new();

pub(super) fn quiet() -> Result<(), bool> {
    LOG.set(false)
}

pub(super) fn __internal_log_enable() -> bool {
    *LOG.get_or_init(|| true)
}

macro_rules! log {
    ($fmt:literal $(, $args:expr)*) => {
        if $crate::logger::__internal_log_enable() {
            print!($fmt$(, $args)*);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }
}
pub(crate) use log;

macro_rules! log_success {
    () => {
        $crate::log!("Done!\n");
    };
}
pub(crate) use log_success;

macro_rules! log_failed {
    () => {
        $crate::log!("FAILED\n");
    };
}
pub(crate) use log_failed;

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
