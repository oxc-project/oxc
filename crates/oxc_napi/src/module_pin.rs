//! Process-lifetime pinning of a native addon's code image.
//!
//! napi-rs (as of 3.11) shuts its built-in Tokio runtime down with
//! `Runtime::shutdown_background()` when the last Node environment (e.g. a
//! `worker_threads` worker) is destroyed. That returns before runtime threads have
//! stopped, so Node can unload the addon while a thread can still execute its code —
//! an execute-after-unload fault (`0xC0000005` on Windows). Pinning the image keeps it
//! mapped for the rest of the process: bounded state (one image per process), not a
//! per-worker leak; per-environment N-API cleanup still runs normally.
//!
//! Remove once napi-rs guarantees runtime quiescence before addon unload.

use std::sync::Once;

/// Install a load-time hook that pins the addon's code image via
/// [`pin_module_image`](crate::pin_module_image).
///
/// Invoke once at crate level in every addon that enables napi's `async` feature (i.e.
/// runs exported functions on napi-rs's Tokio runtime). Expands in the calling crate so
/// the constructor is linked into the addon's own `cdylib`.
#[macro_export]
macro_rules! pin_addon_image {
    () => {
        #[cfg(not(target_family = "wasm"))]
        #[::napi_derive::module_init]
        fn __oxc_pin_addon_image() {
            $crate::pin_module_image();
        }
    };
}

/// Pin the code image containing this (statically linked) function so the OS loader
/// never unmaps it.
///
/// No-op when the code is linked into an executable rather than a dynamic library —
/// an executable's image can never be unloaded.
///
/// Prefer [`pin_addon_image!`](crate::pin_addon_image) over calling this directly.
pub fn pin_module_image() {
    static PIN: Once = Once::new();
    PIN.call_once(pin_impl);
}

/// Pinning from DLL-load (ctor) context is the documented use of
/// `GET_MODULE_HANDLE_EX_FLAG_PIN`. Pinning an executable's own image trivially
/// succeeds, so this is safe in non-addon binaries (CLI, test runners) that link this
/// crate.
///
/// Aborts on failure: continuing would allow a Tokio thread to execute unmapped code
/// after a worker exits, which is memory-unsafe.
#[cfg(windows)]
#[expect(clippy::print_stderr)]
fn pin_impl() {
    use windows_sys::Win32::System::LibraryLoader::{
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_PIN, GetModuleHandleExW,
    };

    let mut handle = std::ptr::null_mut();
    // SAFETY: `pin_module_image` is an address inside this module's image and `handle`
    // is a valid out-pointer.
    let pinned = unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN,
            pin_module_image as *const u16,
            &raw mut handle,
        ) != 0
    };
    if !pinned {
        eprintln!(
            "oxc: failed to pin native addon image in memory; \
             aborting instead of risking execution of unloaded code during worker teardown"
        );
        std::process::abort();
    }
}

/// Take a never-released `dlopen` reference to our own already-loaded image, which
/// keeps it mapped for the process lifetime.
///
/// Best-effort by design: this code is also linked into plain executables (CLI
/// binaries, test runners), where `dladdr` may fail (static linking) or the image is
/// the main executable and not `dlopen`-able. Neither case needs pinning — an
/// executable's image is never unloaded — and `dlopen` gives no way to distinguish
/// them from a genuine pin failure, so failures here must not abort.
#[cfg(unix)]
fn pin_impl() {
    // SAFETY: `pin_module_image` is an address inside this module's image; `info` is a
    // valid out-pointer; `dli_fname` remains valid while the image is loaded.
    // `RTLD_NOLOAD` pins the already-loaded image only — it can never map and pin a
    // second copy under a mismatched path. The returned handle is deliberately never
    // `dlclose`d.
    unsafe {
        let mut info = std::mem::zeroed::<libc::Dl_info>();
        if libc::dladdr(pin_module_image as *const libc::c_void, &raw mut info) != 0
            && !info.dli_fname.is_null()
        {
            libc::dlopen(info.dli_fname, libc::RTLD_NOW | libc::RTLD_NOLOAD);
        }
    }
}

/// Other targets (e.g. WASM) have no dynamic unload of native addon images.
#[cfg(not(any(windows, unix)))]
fn pin_impl() {}
