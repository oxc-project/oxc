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
                print!("{}", format!($fmt$(, $args)*));
            }
        }
    }

macro_rules! log_success {
    () => {
        $crate::log!("Done!\n");
    };
}

macro_rules! log_failed {
    () => {
        $crate::log!("FAILED\n");
    };
}

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

pub(crate) use {log, log_failed, log_result, log_success};
