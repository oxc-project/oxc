use std::{
    cell::Cell,
    cmp, env, fs,
    ptr::{self, NonNull},
    slice, str,
    sync::{
        atomic::{AtomicU32, Ordering},
        mpsc::channel,
    },
    thread::available_parallelism,
};

use bpaf::Bpaf;
use napi::{
    Status,
    bindgen_prelude::{FnArgs, Function, Promise, Uint8Array},
    threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
};
use napi_derive::napi;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use oxc_allocator::{FixedSizeAllocator, free_fixed_size_allocator};
use oxc_parser::Parser;
use oxc_span::SourceType;

mod files;
use files::FILES;

/// CLI arguments.
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// Number of threads to use
    ///
    /// - 0 for using as many CPU cores as system has (default).
    /// - 1 for using only 1 CPU core.
    #[bpaf(argument("INT"), fallback(0))]
    pub threads: u32,

    /// Number of iterations to perform
    #[bpaf(argument("INT"))]
    pub iterations: usize,

    /// `true` if should visit AST on JS side
    #[bpaf(flag(true, false), fallback(false))]
    pub js: bool,

    /// Visitor to use
    #[bpaf(argument("STR"), fallback(String::new()))]
    pub visitor: String,

    /// ID of visitor
    #[bpaf(argument("INT"), fallback(0))]
    pub visitor_id: u32,

    /// Enable logging
    #[bpaf(flag(true, false), fallback(false))]
    pub log: bool,
}

/// `true` if logging is enabled.
static mut LOG: bool = false;

/// Log a message if logging is enabled.
macro_rules! log {
    ($($tokens:tt)*) => {
        // SAFETY: `LOG` is only mutated in `run` function, which is only called once,
        // and before any usage of `log!` macro
        if unsafe { LOG } {
            println!($($tokens)*);
        }
    }
}

/// JS `startThreads` function, which starts requested number of worker threads.
type StartThreads = ThreadsafeFunction<
    // Arguments
    FnArgs<(
        u32,  // Number of threads
        bool, // `true` if logging enabled
    )>,
    // Return value
    Promise<()>,
    // Arguments (repeated)
    FnArgs<(u32, bool)>,
    // ErrorStatus
    Status,
    // CalleeHandled
    false,
>;

/// JS runner function, which runs on a worker thread.
type Runner = ThreadsafeFunction<
    // Arguments
    u32, // Visitor ID
    // Return value
    (),
    // Arguments (repeated)
    u32,
    // ErrorStatus
    Status,
    // CalleeHandled
    false,
>;

/// Thread data.
/// Each thread in thread pool has its own instance of `ThreadData`.
struct ThreadData {
    id: u32,
    run: Runner,
    fixed_size_allocator: FixedSizeAllocator,
}

/// Counter for number of registered worker threads.
static REGISTERED_WORKERS_COUNT: AtomicU32 = AtomicU32::new(0);

/// Pointer to array of `ThreadData`s.
static mut THREAD_DATAS_PTR: NonNull<ThreadData> = NonNull::dangling();

thread_local! {
    /// Thread local containing pointer to [`ThreadData`] for this thread
    static THREAD_DATA_PTR: Cell<NonNull<ThreadData>> = const { Cell::new(NonNull::dangling()) };
}

mod unsafe_ptr {
    use super::*;

    /// An unsafe wrapper around a `NonNull<T>`.
    ///
    /// It's marked as `Send` and `Sync` so can be transferred across threads,
    /// unlike the `NonNull` pointer which it wraps.
    ///
    /// # SAFETY
    ///
    /// It is the user's responsibility to ensure that the way `UnsafePtr`s are used is sound,
    /// and that is safe to pass the `UnsafePtr` across threads.
    pub struct UnsafePtr<T: ?Sized>(NonNull<T>);

    impl<T: ?Sized> Clone for UnsafePtr<T> {
        fn clone(&self) -> Self {
            *self
        }
    }

    impl<T: ?Sized> Copy for UnsafePtr<T> {}

    impl<T: ?Sized> UnsafePtr<T> {
        /// Create an [`UnsafePtr`].
        ///
        /// # SAFETY
        /// Caller must ensure the pointer is used in a sound manner.
        /// See docs for [`UnsafePtr`].
        pub unsafe fn new(ptr: NonNull<T>) -> Self {
            Self(ptr)
        }

        /// Unwrap [`UnsafePtr`] into the underlying `NonNull<T>` pointer.
        pub fn into_inner(self) -> NonNull<T> {
            self.0
        }
    }

    // SAFETY: See above
    unsafe impl<T> Send for UnsafePtr<T> {}
    // SAFETY: See above
    unsafe impl<T> Sync for UnsafePtr<T> {}
}
use unsafe_ptr::UnsafePtr;

/// Entry point from JS.
///
/// * Determine number of threads to use.
/// * Call JS `startWorkers` function to start up worker threads
///   (those worker threads each call `register_worker` when they start up).
/// * Initialize global rayon thread pool with same number of threads.
/// * Pass a pointer to a `ThreadData` to each rayon thread.
/// * Run workload.
///
/// # SAFETY
/// * Must only be called from JS main thread.
/// * Must only be called once.
#[napi]
#[allow(
    clippy::trailing_empty_array,
    clippy::missing_panics_doc,
    clippy::print_stderr,
    clippy::allow_attributes
)]
pub async unsafe fn run(start_workers: StartThreads) -> bool {
    // Parse CLI args
    let Some(options) = parse_options() else { return false };

    // SAFETY: This is only place that `LOG` is written to, and caller promises to only call
    // this function once, so no synchronisation problems
    unsafe { LOG = options.log };

    log!("> Initializing");

    // Get number of threads to use
    let thread_count = match get_thread_count(&options) {
        Ok(thread_count) => thread_count,
        Err(err) => {
            eprintln!("{err}");
            return false;
        }
    };

    // TODO: It seems my benchmarking was wrong and this complicated and unsafe method of avoiding
    // contention between threads doesn't gain much/anything.
    // Probably better to revert to just collecting the worker threads in a `Mutex<Vec<ThreadData>>`.

    // Initialize `Vec` to store `ThreadData`s.
    // Store a pointer to the `Vec`'s contents in `THREAD_DATAS_PTR` static.
    // `register_worker` will use this pointer to initialize the elements of the `Vec`.
    let mut datas = Vec::<ThreadData>::with_capacity(thread_count as usize);
    // SAFETY: Pointer to a slice can never be null
    let datas_ptr = unsafe { NonNull::new_unchecked(datas.as_mut_ptr()) };
    // SAFETY: This is the only place that `THREAD_DATAS_PTR` is written to, and caller promises
    // to only call this function once, so no synchronisation problems
    unsafe { THREAD_DATAS_PTR = datas_ptr };

    // Wrap `datas_ptr` in an `UnsafePtr` to allow moving it over the async boundary.
    // SAFETY: Nothing which happens during the call to `start_workers` invalidates the pointer.
    let datas_ptr = unsafe { UnsafePtr::new(datas_ptr) };

    // Call JS to start worker threads
    start_workers
        .call_async(FnArgs::from((thread_count, options.log)))
        .await
        .unwrap()
        .await
        .unwrap();

    // Check the expected number of worker threads were registered.
    // TODO: If this check fails (or a `start_workers` call above panics),
    // any `ThreadData`s registered before the failure will not be dropped, causing a memory leak.
    // Do we need to guard against that? Does it matter anyway since process exits then anyway?
    // TODO: Is `SeqCst` overkill?
    // TODO: Can we make counting registered workers a debug-mode only thing?
    let registered_count = REGISTERED_WORKERS_COUNT.load(Ordering::SeqCst);
    if registered_count != thread_count {
        eprintln!("Failed to start worker threads");
        return false;
    }

    // Set length of `datas` `Vec` to the number of threads.
    // Now when `datas` is dropped at end of this function, the `ThreadData`s it contains will also be dropped.
    // SAFETY: `Vec` was created with capacity of `thread_count`, and has not been altered since.
    // We checked above that `thread_count` workers have been registered, so all elements of the `Vec`
    // are initialized.
    unsafe { datas.set_len(thread_count as usize) };

    // Start `rayon` thread pool with same number of threads
    // SAFETY: `datas` lives until the end of this function, so `datas_ptr` remains valid until then.
    // No work occurs in thread pool after end of this function.
    unsafe { init_rayon_thread_pool(datas_ptr, thread_count as usize) };

    log!("> Initialized {thread_count} workers");

    // Load test files
    let files = load_test_files();
    let files =
        files.iter().map(|(path, source_text)| (*path, source_text.as_str())).collect::<Vec<_>>();

    // Run workload
    run_workload(&options, &files);

    true
}

/// Parse options from CLI arguments.
fn parse_options() -> Option<Options> {
    let mut args = std::env::args_os();
    if args.next().is_some_and(|arg| arg == "node") {
        args.next();
    }
    let args = args.collect::<Vec<_>>();

    let options_parser = options();
    match options_parser.run_inner(&*args) {
        Ok(mut options) => {
            options.visitor_id = match options.visitor.as_str() {
                "" => 0,
                "empty" => 1,
                "debugger" => 2,
                "ident" => 3,
                _ => panic!("Unknown visitor"),
            };
            Some(options)
        }
        Err(e) => {
            e.print_message(100);
            None
        }
    }
}

/// Get number of threads to use.
///
/// `--threads` CLI argument takes precedence, otherwise get available parallelism from OS.
///
/// Return value will be greater than 0.
///
/// # Errors
/// Returns `Err` if unable to determine number of threads.
fn get_thread_count(options: &Options) -> Result<u32, String> {
    let max_thread_count = cmp::min(rayon::max_num_threads(), u32::MAX as usize);

    let thread_count = options.threads;
    if thread_count > 0 {
        if thread_count as usize > max_thread_count {
            return Err(format!(
                "Requested too many threads: {thread_count} vs {max_thread_count} maximum"
            ));
        }
        return Ok(thread_count);
    }

    available_parallelism()
        .map(|thread_count| {
            // `max_thread_count <= u32::MAX` so `as u32` cannot truncate
            #[expect(clippy::cast_possible_truncation)]
            let thread_count = cmp::min(thread_count.get(), max_thread_count as usize) as u32;
            thread_count
        })
        .map_err(|e| format!("Failed to determine available parallelism: {e}"))
}

// TODO: Don't hard-code this here
const BUFFER_SIZE: usize = 2_147_483_616;

/// Register a JS worker thread.
/// Is passed a `run` function.
///
/// # SAFETY
/// * Must only be called in response to a request to call this made by `run` calling `startWorkers`.
/// * `worker_id` must be less than thread count passed to `startWorkers` by `run`.
/// * Each call to this function must pass a unique `worker_id`.
#[napi]
#[allow(clippy::missing_panics_doc, clippy::needless_pass_by_value, clippy::allow_attributes)]
pub unsafe fn register_worker(
    worker_id: u32,
    store_buffer: Function<Uint8Array, ()>,
    run: Function<u32, ()>,
) {
    log!("> Registering worker {worker_id}");

    // Wrap `run` in a `ThreadsafeFunction`
    let run = run.build_threadsafe_function().build().unwrap();

    // Create `Allocator`
    let fixed_size_allocator = FixedSizeAllocator::new(worker_id);
    let allocator = fixed_size_allocator.get();

    // Mark allocator as sent to JS.
    // SAFETY: `allocator` was created by a `FixedSizeAllocator`. We don't create a mut ref from the pointer.
    let metadata_ptr = unsafe { allocator.fixed_size_metadata_ptr() };
    {
        // SAFETY: Fixed-size allocators always have a valid `FixedSizeAllocatorMetadata`
        // stored at the pointer returned by `Allocator::fixed_size_metadata_ptr`.
        let metadata = unsafe { metadata_ptr.as_ref() };
        // TODO: Is `Ordering::SeqCst` excessive here?
        metadata.is_double_owned.store(true, Ordering::SeqCst);
    };

    // Send buffer to JS
    // SAFETY: TODO
    let buffer = unsafe {
        Uint8Array::with_external_data(
            allocator.data_ptr().as_ptr(),
            BUFFER_SIZE,
            move |_ptr, _len| free_fixed_size_allocator(metadata_ptr),
        )
    };
    store_buffer.call(buffer).unwrap();

    // Store `ThreadData`
    let data = ThreadData { id: worker_id, run, fixed_size_allocator };

    // SAFETY: `THREAD_DATAS_PTR` is initialized in `run`, and points to a slice of memory large enough
    // to accomodate `thread_count` x `ThreadData` instances.
    // Caller promises this function has only been called in response to a call to `startWorkers`
    // and that `worker_id` is less than `thread_count`, so `THREAD_DATAS_PTR.add(worker_id)` is in bounds.
    // Caller also promises this function is called each time with a unique `worker_id`,
    // so there are no synchronisation issues of 2 threads writing to the same address at same time.
    unsafe {
        let data_ptr = THREAD_DATAS_PTR.add(worker_id as usize);
        data_ptr.write(data);
    }

    // Increment counter of number of registered workers.
    // TODO: Is `SeqCst` overkill?
    // TODO: Can we make counting registered workers a debug-mode only thing?
    REGISTERED_WORKERS_COUNT.fetch_add(1, Ordering::SeqCst);
}

/// Start a rayon thread pool and assign a `ThreadData` to each thread.
///
/// Pointer to `ThreadData` is stored in `THREAD_DATA` thread local storage for each thread.
///
/// # SAFETY
/// * `datas_ptr` must be valid pointer to an array of `thread_count` valid `ThreadData` instances.
/// * Those `ThreadData` instances must remain valid until the thread pool completes all work.
unsafe fn init_rayon_thread_pool(datas_ptr: UnsafePtr<ThreadData>, thread_count: usize) {
    // Start `rayon` thread pool
    rayon::ThreadPoolBuilder::new().num_threads(thread_count).build_global().unwrap();

    // Store pointer to `ThreadData` for each thread in thread-local storage.
    //
    // `broadcast` executes the closure on every thread in the thread pool,
    // passing the thread ID of that thread into the closure.
    // Those thread IDs are unique and cover the range `0..thread_count`.
    //
    // Each thread gets assigned one of the `ThreadData`s which start at `datas_ptr`.
    // Because the thread IDs are unique, the `ThreadData` each thread receives is unique too.
    // The caller guarantees that the `ThreadData`s remains valid until after the thread pool has
    // completed all work.
    //
    // Therefore, when running tasks, each thread can safely dereference its `NonNull<ThreadData>` pointer
    // to a `&mut ThreadData`, knowing that it's a valid reference, and no other thread can have access to it.
    //
    // This is sound, but there's no way to do this with safe code.
    // `UnsafePtr` wrapper circumvents the type system, and allows copying `datas_ptr`
    // into `broadcast` closure.

    #[cfg_attr(not(debug_assertions), expect(unused_variables, unused_mut))]
    let mut thread_ids = rayon::broadcast(|ctx| {
        let thread_id = ctx.index();

        debug_assert!(thread_id < thread_count);
        debug_assert!(ctx.num_threads() == thread_count);

        // SAFETY: We created rayon thread pool with `thread_count` threads.
        // `thread_id` is less than `thread_count`, there are `thread_count` `ThreadData` instances
        // starting at `datas_ptr`.
        // Therefore `datas_ptr.add(thread_id)` points to a valid `ThreadData`, and cannot be out of
        // bounds of the allocation containing the `ThreadData`s.
        let data_ptr = unsafe { datas_ptr.into_inner().add(thread_id) };
        THREAD_DATA_PTR.set(data_ptr);

        log!("> Set thread data for thread {thread_id}");

        // Return `()` in release mode to avoid the overhead of building a `Vec<usize>`
        #[cfg(debug_assertions)]
        {
            thread_id
        }
    });

    // Check thread IDs are unique
    #[cfg(debug_assertions)]
    {
        thread_ids.sort_unstable();
        assert!(
            thread_ids.len() == thread_count
                && thread_ids.into_iter().enumerate().all(|(expected_id, id)| id == expected_id)
        );
    }
}

/// Read test files from disk.
fn load_test_files() -> Vec<(&'static str, String)> {
    let cwd = env::current_dir().unwrap();
    let root_path = cwd.parent().unwrap().parent().unwrap();

    FILES
        .iter()
        .map(|&relative_path| {
            let path = root_path.join(relative_path);
            let bytes = fs::read(path).unwrap();
            simdutf8::basic::from_utf8(&bytes).unwrap();
            log!("Loaded test file: {relative_path}");
            // SAFETY: `simdutf8` has ensured it's a valid UTF-8 string
            (relative_path, unsafe { String::from_utf8_unchecked(bytes) })
        })
        .collect()
}

/// Run workload across all threads.
fn run_workload(options: &Options, files: &[(&str, &str)]) -> bool {
    let files_len = files.len();
    let failures = (0..options.iterations)
        .into_par_iter()
        .filter(|&iteration| {
            let (path, source_text) = files[iteration % files_len];
            !run_job(path, source_text, options)
        })
        .count();

    failures == 0
}

/// Metadata written to end of buffer.
///
/// Copied from `oxc_linter` crate.
/// Fields re-ordered to match what `#[ast]` macro does.
#[repr(C)]
struct RawTransferMetadata {
    /// Padding to pad struct to size 16.
    pub(crate) _padding: u64,
    /// Offset of `Program` within buffer.
    /// Note: In `RawTransferMetadata` (in `napi/parser`), this field is offset of `RawTransferData`,
    /// but here it's offset of `Program`.
    pub data_offset: u32,
    /// `true` if AST is TypeScript.
    pub is_ts: bool,
}

/// Run single job on a thread.
fn run_job(path: &str, source_text: &str, options: &Options) -> bool {
    // SAFETY: Each thread has exclusive access to its `ThreadData`
    let thread_data = unsafe { THREAD_DATA_PTR.get().as_mut() };

    log!("> Run job on worker {} - {path}", thread_data.id);

    // Write source text into start of buffer
    let allocator = thread_data.fixed_size_allocator.get();
    // SAFETY: Allocator is wrapped in a `FixedSizeAllocator`, which takes care of dropping `Allocator`
    let source_ptr = unsafe { allocator.alloc_bytes_start(source_text.len()) }.as_ptr();
    // SAFETY: We just reserved space in allocator for source text
    unsafe {
        ptr::copy_nonoverlapping(source_text.as_ptr(), source_ptr, source_text.len());
    }
    // SAFETY: We just wrote source_text into allocator at `source_ptr`
    let source_text = unsafe {
        let slice = slice::from_raw_parts(source_ptr, source_text.len());
        str::from_utf8_unchecked(slice)
    };

    // Parse source
    log!("> Parsing: '{path}' on worker {}", thread_data.id);

    let source_type = SourceType::from_path(path).unwrap();
    let program = Parser::new(allocator, source_text, source_type).parse().program;
    let program = allocator.alloc(program);

    // Write offset of `Program` in metadata at end of buffer
    let program_offset = ptr::from_ref(program) as u32;
    let metadata = RawTransferMetadata { data_offset: program_offset, is_ts: false, _padding: 0 };
    let metadata_ptr = allocator.end_ptr().cast::<RawTransferMetadata>();
    // SAFETY: `Allocator` was created by `FixedSizeAllocator` which reserved space after `end_ptr`
    // for a `RawTransferMetadata`. `end_ptr` is aligned for `RawTransferMetadata`.
    unsafe { metadata_ptr.write(metadata) };

    // Run JS `run` function
    if !options.js {
        // Reset allocator
        thread_data.fixed_size_allocator.reset();
        return true;
    }

    let (tx, rx) = channel();

    let status = thread_data.run.call_with_return_value(
        options.visitor_id,
        ThreadsafeFunctionCallMode::NonBlocking,
        move |result, _env| {
            let _ = match &result {
                Ok(()) => tx.send(Ok(())),
                Err(e) => tx.send(Err(e.to_string())),
            };

            result
        },
    );

    // Reset allocator
    thread_data.fixed_size_allocator.reset();

    if status != Status::Ok {
        log!("Failed to schedule callback: {status:?}");
        return false;
    }

    match rx.recv() {
        Ok(Ok(())) => true,
        Ok(Err(e)) => {
            log!("Callback reported error: {e}");
            false
        }
        Err(e) => {
            log!("Callback did not respond: {e}");
            false
        }
    }
}
