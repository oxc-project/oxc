#![expect(clippy::print_stdout)]

use std::{cell::Cell, cmp, mem, ptr::NonNull, sync::Mutex, thread::available_parallelism};

use bpaf::Bpaf;
use napi::{
    Status,
    bindgen_prelude::{Function, Promise},
    threadsafe_function::ThreadsafeFunction,
};
use napi_derive::napi;

/// JS runner function, which runs on a worker thread.
type Runner = ThreadsafeFunction<
    // Arguments
    (),
    // Return value
    (),
    // Arguments (repeated)
    (),
    // ErrorStatus
    Status,
    // CalleeHandled
    false,
>;

/// JS `startThreads` function, which starts
type StartThreads = ThreadsafeFunction<
    // Arguments
    u32, // Number of threads
    // Return value
    Promise<()>,
    // Arguments (repeated)
    u32,
    // ErrorStatus
    Status,
    // CalleeHandled
    false,
>;

/// JS runner functions, each on its own worker thread
static RUNNERS: Mutex<Vec<Runner>> = Mutex::new(Vec::new());

/// CLI arguments
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct TestCommand {
    /// Number of threads to use. Set to 1 for using only 1 CPU core
    #[bpaf(argument("INT"), hide_usage)]
    pub threads: Option<u32>,
}

thread_local! {
    static RUNNER: Cell<NonNull<Runner>> = const { Cell::new(NonNull::dangling()) };
}

/// Entry point from JS.
///
/// * Determine number of threads to use.
/// * Call JS `startWorkers` function to start up worker threads
///   (those worker threads each call `register_worker` when they start up).
/// * Initialize global rayon thread pool with same number of threads.
/// * Pass a pointer to a `Runner` to each rayon thread.
/// * Runs workload.
#[napi]
#[allow(clippy::trailing_empty_array, clippy::missing_panics_doc, clippy::allow_attributes)]
pub async fn run(start_workers: StartThreads) -> bool {
    println!("> Initializing");

    // Get number of threads
    let Some(thread_count) = get_threads() else { return false };

    // Call JS to start worker threads
    start_workers.call_async(thread_count).await.unwrap().await.unwrap();

    let mut runners = {
        let mut runners = RUNNERS.lock().unwrap();
        mem::take(&mut *runners)
    };

    #[expect(clippy::print_stderr)]
    if runners.len() != thread_count as usize {
        eprintln!("Failed to start worker threads");
        return false;
    }

    // Start `rayon` thread pool with same number of threads
    // SAFETY: TODO
    unsafe { init_rayon_thread_pool(&mut runners) };

    println!("> Initialized {thread_count} workers");

    // TODO: Run workload

    true
}

/// Wrapper for a `NonNull<Runner>` pointer, that allows copying it across threads.
#[derive(Clone, Copy)]
struct RunnerPtr(NonNull<Runner>);

impl RunnerPtr {
    /// SAFETY: TODO
    unsafe fn new(ptr: NonNull<Runner>) -> Self {
        Self(ptr)
    }

    fn into_inner(self) -> NonNull<Runner> {
        self.0
    }
}

// SAFETY: TODO
unsafe impl Sync for RunnerPtr {}

/// Start a rayon thread pool and assign a `Runner` to each.
///
/// Pointer to `Runner` is stored in `RUNNER` thread local storage for each thread.
///
/// # SAFETY
/// The slice passed to this function must remain valid until the thread pool completes all work.
unsafe fn init_rayon_thread_pool(runners: &mut [Runner]) {
    let thread_count = runners.len();

    // Start `rayon` thread pool
    rayon::ThreadPoolBuilder::new().num_threads(thread_count).build_global().unwrap();

    // Store pointer to `Runner` for each thread in thread-local storage.
    // SAFETY: Pointer to a slice can never be null.
    let runners_ptr = unsafe { NonNull::new_unchecked(runners.as_mut_ptr()) };
    // SAFETY: TODO
    let runners_ptr = unsafe { RunnerPtr::new(runners_ptr) };

    let mut thread_ids = rayon::broadcast(|ctx| {
        let thread_id = ctx.index();

        debug_assert!(thread_id < thread_count);
        debug_assert!(ctx.num_threads() == thread_count);

        // SAFETY: TODO
        let runner_ptr = unsafe { runners_ptr.into_inner().add(thread_id) };
        RUNNER.set(runner_ptr);

        println!("> Set runner for thread {thread_id}");

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
                && thread_ids
                    .into_iter()
                    .zip(0..thread_count)
                    .all(|(id, expected_id)| id == expected_id)
        );
    }
}

/// Get number of threads to use.
///
/// `--threads` CLI argument takes precedence, otherwise get available parallelism from OS.
#[expect(clippy::print_stderr)]
fn get_threads() -> Option<u32> {
    // Parse CLI arguments
    let mut args = std::env::args_os();
    if args.next().is_some_and(|arg| arg == "node") {
        args.next();
    }
    let args = args.collect::<Vec<_>>();

    let command_parser = test_command();
    let command = match command_parser.run_inner(&*args) {
        Ok(command) => command,
        Err(e) => {
            e.print_message(100);
            return None;
        }
    };

    let max_thread_count = cmp::min(rayon::max_num_threads(), u32::MAX as usize);
    if let Some(thread_count) = command.threads {
        if thread_count > 0 {
            if thread_count as usize > max_thread_count {
                eprintln!(
                    "Requested too many threads: {thread_count} vs {max_thread_count} maximum"
                );
                return None;
            }

            return Some(thread_count);
        }
    }

    match available_parallelism() {
        Ok(thread_count) => {
            // `max_thread_count <= u32::MAX` so `as u32` cannot truncate
            #[expect(clippy::cast_possible_truncation)]
            let thread_count = cmp::min(thread_count.get(), max_thread_count as usize) as u32;
            Some(thread_count)
        }
        Err(e) => {
            eprintln!("Failed to determine available parallelism: {e}");
            None
        }
    }
}

/// Register a JS runner function.
/// Called from a JS worker thread.
#[napi]
#[allow(clippy::missing_panics_doc, clippy::allow_attributes)]
pub fn register_worker(worker_id: u32, run: Function<(), ()>) {
    println!("> Registering worker {worker_id}");

    let runner = run.build_threadsafe_function().build().unwrap();
    let mut runners = RUNNERS.lock().unwrap();
    runners.push(runner);
}
