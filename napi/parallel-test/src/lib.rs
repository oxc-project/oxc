#![expect(clippy::print_stdout)]

use std::{
    cell::Cell, cmp, convert::identity, mem, ptr::NonNull, sync::Mutex,
    thread::available_parallelism,
};

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

    /// Number of iterations to perform
    #[bpaf(argument("INT"), hide_usage)]
    pub iterations: usize,
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
#[allow(
    clippy::trailing_empty_array,
    clippy::missing_panics_doc,
    clippy::print_stderr,
    clippy::allow_attributes
)]
pub async fn run(start_workers: StartThreads) -> bool {
    println!("> Initializing");

    // Parse CLI args
    let Some(command) = parse_options() else { return false };

    // Get number of threads to use
    let thread_count = match get_threads(&command) {
        Ok(thread_count) => thread_count,
        Err(err) => {
            eprintln!("{err}");
            return false;
        }
    };

    // Call JS to start worker threads
    start_workers.call_async(thread_count).await.unwrap().await.unwrap();

    let mut runners = {
        let mut runners = RUNNERS.lock().unwrap();
        mem::take(&mut *runners)
    };

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

/// Parse options from CLI arguments.
fn parse_options() -> Option<TestCommand> {
    let mut args = std::env::args_os();
    if args.next().is_some_and(|arg| arg == "node") {
        args.next();
    }
    let args = args.collect::<Vec<_>>();

    let command_parser = test_command();
    match command_parser.run_inner(&*args) {
        Ok(command) => Some(command),
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
/// Return value will not be greater than 0.
///
/// Returns `None` if unable to determine number of threads.
fn get_threads(command: &TestCommand) -> Result<u32, String> {
    let max_thread_count = cmp::min(rayon::max_num_threads(), u32::MAX as usize);

    if let Some(thread_count) = command.threads {
        if thread_count > 0 {
            if thread_count as usize > max_thread_count {
                return Err(format!(
                    "Requested too many threads: {thread_count} vs {max_thread_count} maximum"
                ));
            }
            return Ok(thread_count);
        }
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

/// Start a rayon thread pool and assign a `Runner` to each.
///
/// Pointer to `Runner` is stored in `RUNNER` thread local storage for each thread.
///
/// # SAFETY
/// The slice passed to this function must remain valid until the thread pool completes all work.
#[expect(clippy::items_after_statements)]
unsafe fn init_rayon_thread_pool(runners: &mut [Runner]) {
    let thread_count = runners.len();

    // Start `rayon` thread pool
    rayon::ThreadPoolBuilder::new().num_threads(thread_count).build_global().unwrap();

    // Store pointer to `Runner` for each thread in thread-local storage.
    // Use `RunnerPtr` wrapper to allow copying pointer into `broadcast` closure.
    #[derive(Clone, Copy)]
    struct RunnerPtr(NonNull<Runner>);
    // SAFETY: TODO
    unsafe impl Sync for RunnerPtr {}

    // SAFETY: Pointer to a slice can never be null.
    let runners_ptr = RunnerPtr(unsafe { NonNull::new_unchecked(runners.as_mut_ptr()) });

    let mut thread_ids = rayon::broadcast(|ctx| {
        let thread_id = ctx.index();

        debug_assert!(thread_id < thread_count);
        debug_assert!(ctx.num_threads() == thread_count);

        // SAFETY: TODO
        let runner_ptr = unsafe { identity(runners_ptr).0.add(thread_id) };
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
