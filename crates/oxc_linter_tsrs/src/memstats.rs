//! `TSRSLINT_MEM_STATS=1`: one stderr line per phase boundary (config assignment, each program built and linted)
//! with the process's memory now and at its peak, user/sys CPU, minor page faults and, on macOS, retired
//! instructions. Attributes peak memory and CPU to phases; instruction deltas of the lint phase stay precise when
//! the machine is loaded, unlike wall time.

use std::sync::OnceLock;
use std::time::Instant;

fn enabled() -> bool {
    static S: OnceLock<bool> = OnceLock::new();
    *S.get_or_init(|| std::env::var("TSRSLINT_MEM_STATS").is_ok_and(|v| v == "1"))
}

pub(crate) fn mark(phase: &str) {
    if !enabled() {
        return;
    }
    static T0: OnceLock<Instant> = OnceLock::new();
    let t = T0.get_or_init(Instant::now).elapsed().as_secs_f64();
    let gib = |b: u64| b as f64 / (1u64 << 30) as f64;
    let (user, sys, minflt) = rusage();
    let (now, peak, instr) = memory();
    let instr = instr.map_or(String::new(), |i| format!("  instr {:7.1} G", i as f64 / 1e9));
    eprintln!(
        "tsrslint mem [{t:7.2}s] {phase:<28} now {:6.3} GiB  peak {:6.3} GiB  user {user:6.2}s  sys {sys:6.2}s  minflt {minflt:>8}{instr}",
        gib(now),
        gib(peak),
    );
}

#[repr(C)]
#[derive(Default)]
struct Timeval {
    sec: i64,
    #[cfg(target_os = "macos")]
    usec: i32,
    #[cfg(target_os = "macos")]
    _pad: i32,
    #[cfg(not(target_os = "macos"))]
    usec: i64,
}

#[repr(C)]
#[derive(Default)]
struct Rusage {
    utime: Timeval,
    stime: Timeval,
    // ru_maxrss, ru_ixrss, ru_idrss, ru_isrss, ru_minflt, ...
    rest: [i64; 14],
}

/// (user s, sys s, minor faults) of the process.
#[cfg(unix)]
fn rusage() -> (f64, f64, i64) {
    extern "C" {
        fn getrusage(who: i32, usage: *mut Rusage) -> i32;
    }
    let mut ru = Rusage::default();
    // SAFETY: getrusage(RUSAGE_SELF) fills the struct, laid out as `struct rusage` on 64-bit macOS and Linux.
    if unsafe { getrusage(0, &raw mut ru) } != 0 {
        return (0.0, 0.0, 0);
    }
    let s = |t: &Timeval| t.sec as f64 + t.usec as f64 * 1e-6;
    (s(&ru.utime), s(&ru.stime), ru.rest[4])
}

#[cfg(not(unix))]
fn rusage() -> (f64, f64, i64) {
    (0.0, 0.0, 0)
}

/// (physical footprint now, lifetime peak footprint, retired instructions). The peak is what `/usr/bin/time -l`
/// reports as "peak memory footprint".
#[cfg(target_os = "macos")]
fn memory() -> (u64, u64, Option<u64>) {
    extern "C" {
        fn proc_pid_rusage(pid: i32, flavor: i32, buffer: *mut u64) -> i32;
        fn getpid() -> i32;
    }
    // struct rusage_info_v4: a 16-byte uuid, then u64 fields; ri_phys_footprint is field 7,
    // ri_lifetime_max_phys_footprint 28, ri_instructions 29.
    let mut buf = [0u64; 2 + 40];
    // SAFETY: RUSAGE_INFO_V4 (4) writes a rusage_info_v4, which is smaller than `buf`.
    if unsafe { proc_pid_rusage(getpid(), 4, buf.as_mut_ptr()) } != 0 {
        return (0, 0, None);
    }
    let f = &buf[2..];
    (f[7], f[28], Some(f[29]))
}

/// (resident now, peak resident) from /proc/self/status.
#[cfg(not(target_os = "macos"))]
fn memory() -> (u64, u64, Option<u64>) {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let kib = |key: &str| {
        status
            .lines()
            .find_map(|l| l.strip_prefix(key))
            .and_then(|v| v.trim().trim_end_matches("kB").trim().parse::<u64>().ok())
            .unwrap_or(0)
            * 1024
    };
    (kib("VmRSS:"), kib("VmHWM:"), None)
}
