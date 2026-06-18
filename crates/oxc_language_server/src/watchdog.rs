/// Spawn a background thread that checks every 10s whether the parent process
/// is still alive. If the parent has exited (editor crash / force quit without
/// sending LSP `shutdown`), the child is re-parented to PID 1. The watchdog
/// detects this and terminates the entire process group.
///
/// See <https://github.com/microsoft/typescript-go/issues/2478>.
#[cfg(unix)]
pub(crate) fn spawn() {
    let initial_ppid = std::os::unix::process::parent_id();
    std::thread::Builder::new()
        .name("oxlint-watchdog".into())
        .spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(10));
            let current_ppid = std::os::unix::process::parent_id();
            if current_ppid != initial_ppid {
                tracing::warn!(
                    "LSP parent process {initial_ppid} no longer alive \
                     (re-parented to {current_ppid}); \
                     terminating process group to stop orphaned CPU loop",
                );
                // kill(0, SIGKILL) terminates every process in the caller's process
                // group: the LSP itself and all its children (e.g. tsgolint).
                // SIGKILL cannot be caught or ignored.
                unsafe { kill(0, SIGKILL) };
                std::process::exit(0);
            }
        })
        .expect("failed to spawn LSP parent-process watchdog thread");
}

#[cfg(not(unix))]
pub(crate) fn spawn() {}

#[cfg(unix)]
extern "C" {
    fn kill(pid: i32, sig: i32) -> i32;
}

#[cfg(unix)]
const SIGKILL: i32 = 9;
