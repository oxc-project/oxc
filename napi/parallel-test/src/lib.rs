#![expect(clippy::print_stdout)]

use std::{mem, sync::Mutex, thread::available_parallelism};

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

/// Entry point from JS.
///
/// * Determines number of threads to use.
/// * Calls JS `startWorkers` function to start up worker threads
///   (those worker threads each call `register_worker` when they start up).
/// * Runs workload.
#[napi]
#[allow(clippy::trailing_empty_array, clippy::missing_panics_doc, clippy::allow_attributes)]
pub async fn run(start_workers: StartThreads) -> bool {
    println!("> Initializing");

    // Get number of threads
    let Some(thread_count) = get_threads() else { return false };

    // Call JS to start worker threads
    start_workers.call_async(thread_count).await.unwrap().await.unwrap();

    let runners = {
        let mut runners = RUNNERS.lock().unwrap();
        mem::take(&mut *runners)
    };

    #[expect(clippy::print_stderr)]
    if runners.len() != thread_count as usize {
        eprintln!("Failed to start worker threads");
        return false;
    }

    println!("> Initialized {thread_count} workers");

    true
}

/// Get number of threads to use.
///
/// `--threads` CLI argument takes precedence, otherwise get available parallelism from OS.
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

    if let Some(thread_count) = command.threads {
        if thread_count > 0 {
            return Some(thread_count);
        }
    }

    match available_parallelism() {
        Ok(thread_count) => u32::try_from(thread_count.get()).ok().or(Some(u32::MAX)),
        #[expect(clippy::print_stderr)]
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
