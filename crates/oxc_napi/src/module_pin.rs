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

/// Pin the addon's code image via [`pin_module_image`](crate::pin_module_image) during
/// N-API module registration.
///
/// Invoke once at crate level in every addon that enables napi's `async` feature (i.e.
/// runs exported functions on napi-rs's Tokio runtime). Expands in the calling crate so
/// the pinned image is the addon's own `cdylib`.
///
/// The hook runs when Node registers the module — after the library is fully loaded
/// (never under the OS loader lock, where loader APIs are unsafe to call) and only when
/// the library was actually loaded as a Node addon, never in plain executables (CLI
/// binaries, test runners) that link the same crate.
///
/// napi-rs keeps a single registration hook per addon: the addon must not define
/// another `#[napi(module_exports)]` function, or one of the two is silently dropped.
#[macro_export]
macro_rules! pin_addon_image {
    () => {
        #[doc(hidden)]
        #[::napi_derive::napi(module_exports)]
        fn __oxc_pin_addon_image() -> ::napi::Result<()> {
            $crate::pin_module_image();
            Ok(())
        }
    };
}

/// Pin the code image containing this (statically linked) function so the OS loader
/// never unmaps it.
///
/// Must only be called from code that is loaded as a dynamic library, e.g. an N-API
/// registration hook — prefer [`pin_addon_image!`](crate::pin_addon_image). Aborts if
/// pinning fails: continuing would allow a Tokio thread to execute unmapped code after
/// a worker exits, which is memory-unsafe.
#[expect(clippy::print_stderr)]
pub fn pin_module_image() {
    static PIN: Once = Once::new();
    PIN.call_once(|| {
        if !pin_impl() {
            eprintln!(
                "oxc: failed to pin native addon image in memory; \
                 aborting instead of risking execution of unloaded code during worker teardown"
            );
            std::process::abort();
        }
    });
}

/// `GET_MODULE_HANDLE_EX_FLAG_PIN` keeps the DLL loaded until process termination
/// regardless of later `FreeLibrary` calls.
#[cfg(windows)]
fn pin_impl() -> bool {
    use windows_sys::Win32::System::LibraryLoader::{
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_PIN, GetModuleHandleExW,
    };

    let mut handle = std::ptr::null_mut();
    // SAFETY: `pin_module_image` is an address inside this module's image and `handle`
    // is a valid out-pointer.
    unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN,
            pin_module_image as *const u16,
            &raw mut handle,
        ) != 0
    }
}

/// Take a never-released `dlopen` reference to our own already-loaded image, which
/// keeps it mapped for the process lifetime. `RTLD_NOLOAD` pins the already-loaded
/// image only — it can never map and pin a second copy under a mismatched path.
#[cfg(unix)]
fn pin_impl() -> bool {
    // SAFETY: `pin_module_image` is an address inside this module's image; `info` is a
    // valid out-pointer; `dli_fname` remains valid while the image is loaded, which
    // `dlopen`'s never-released reference then guarantees for the process lifetime.
    unsafe {
        let mut info = std::mem::zeroed::<libc::Dl_info>();
        if libc::dladdr(pin_module_image as *const libc::c_void, &raw mut info) == 0
            || info.dli_fname.is_null()
        {
            return false;
        }
        !libc::dlopen(info.dli_fname, libc::RTLD_NOW | libc::RTLD_NOLOAD).is_null()
    }
}

/// Other targets (e.g. WASM) have no dynamic unload of native addon images.
#[cfg(not(any(windows, unix)))]
fn pin_impl() -> bool {
    true
}
