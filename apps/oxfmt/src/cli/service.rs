use std::{
    cmp::Ordering,
    collections::BinaryHeap,
    fs,
    path::Path,
    sync::{Arc, Condvar, Mutex, mpsc},
    time::{Duration, Instant},
};

use cow_utils::CowUtils;
use tracing::debug_span;

use oxc_diagnostics::{DiagnosticSender, DiagnosticService};

use super::command::OutputMode;
use crate::core::{FormatPlan, FormatResult, SourceFormatter, utils};

pub enum SuccessResult {
    /// Path with elapsed time, only measured in check mode
    Changed(String, Option<Duration>),
    Unchanged,
}

/// A queued item, ordered so that the largest file pops first (ties: earliest first).
struct Queued<T> {
    size: u64,
    seq: u64,
    item: T,
}

impl<T> PartialEq for Queued<T> {
    fn eq(&self, other: &Self) -> bool {
        self.seq == other.seq
    }
}
impl<T> Eq for Queued<T> {}
impl<T> PartialOrd for Queued<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T> Ord for Queued<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.size.cmp(&other.size).then_with(|| other.seq.cmp(&self.seq))
    }
}

struct Pending<T> {
    heap: BinaryHeap<Queued<T>>,
    seq: u64,
    walk_done: bool,
}

/// Entries received from the walk but not yet picked up by a worker.
/// Workers take the largest accumulated one, so a big file does not wait behind small ones.
/// The lock is only held to push or pop; idle workers sleep on the condvar.
struct LargestFirst<T> {
    pending: Mutex<Pending<T>>,
    available: Condvar,
}

impl<T> LargestFirst<T> {
    fn new() -> Self {
        let pending = Pending { heap: BinaryHeap::new(), seq: 0, walk_done: false };
        Self { pending: Mutex::new(pending), available: Condvar::new() }
    }

    fn push(&self, size: u64, item: T) {
        let mut pending = self.pending.lock().unwrap();
        pending.seq += 1;
        let seq = pending.seq;
        pending.heap.push(Queued { size, seq, item });
        drop(pending);
        self.available.notify_one();
    }

    /// No more entries will be pushed: release the workers once the heap is drained.
    fn finish(&self) {
        self.pending.lock().unwrap().walk_done = true;
        self.available.notify_all();
    }

    /// Pop the largest entry, sleeping while the heap is empty and the walk is running.
    fn pop(&self) -> Option<T> {
        let pending = self.pending.lock().unwrap();
        let mut pending = self
            .available
            .wait_while(pending, |pending| pending.heap.is_empty() && !pending.walk_done)
            .unwrap();
        pending.heap.pop().map(|queued| queued.item)
    }
}

pub struct FormatService {
    cwd: Box<Path>,
    format_mode: OutputMode,
    formatter: SourceFormatter,
}

impl FormatService {
    pub fn new<T>(cwd: T, format_mode: OutputMode, formatter: SourceFormatter) -> Self
    where
        T: Into<Box<Path>>,
    {
        Self { cwd: cwd.into(), format_mode, formatter }
    }

    /// Process entries as they are received from the channel
    pub fn run_streaming(
        &self,
        rx_entry: mpsc::Receiver<FormatPlan>,
        tx_error: &DiagnosticSender,
        tx_success: &mpsc::Sender<SuccessResult>,
    ) {
        let process = |plan: FormatPlan| {
            let start_time = matches!(self.format_mode, OutputMode::Check).then(Instant::now);

            let path = Arc::clone(&plan.path);
            let Ok(source_text) = debug_span!("oxfmt::read", path = %path.display())
                .in_scope(|| utils::read_to_string(&path))
            else {
                // This happens if binary file is attempted to be formatted
                // e.g. `.ts` for MPEG-TS video file
                let diagnostics = DiagnosticService::wrap_diagnostics(
                    self.cwd.clone(),
                    &path,
                    "",
                    vec![
                        oxc_diagnostics::OxcDiagnostic::error(format!(
                            "Failed to read file: {}",
                            path.display()
                        ))
                        .with_help("This may be due to the file being a binary or inaccessible."),
                    ],
                );
                let _ = tx_error.send(diagnostics);
                return;
            };

            let (code, is_changed) = match self.formatter.format(&source_text, plan) {
                FormatResult::Success { code, is_changed } => (code, is_changed),
                FormatResult::Error(diagnostics) => {
                    let errors = DiagnosticService::wrap_diagnostics(
                        self.cwd.clone(),
                        &path,
                        &source_text,
                        diagnostics,
                    );
                    let _ = tx_error.send(errors);
                    return;
                }
            };

            // Write back if needed
            if matches!(self.format_mode, OutputMode::Write) && is_changed {
                match fs::write(&path, &code) {
                    Ok(()) => (),
                    Err(err) => {
                        let diagnostics = DiagnosticService::wrap_diagnostics(
                            self.cwd.clone(),
                            &path,
                            "",
                            vec![oxc_diagnostics::OxcDiagnostic::error(format!(
                                "Failed to save '{}': {err}",
                                path.display()
                            ))],
                        );
                        let _ = tx_error.send(diagnostics);
                        return;
                    }
                }
            }

            // Report result
            let result = match (&self.format_mode, is_changed) {
                (OutputMode::Check | OutputMode::ListDifferent, true) => {
                    let display_path = path
                        // Show path relative to `cwd` for cleaner output
                        .strip_prefix(&self.cwd)
                        .unwrap_or(path.as_ref())
                        .to_string_lossy()
                        // Normalize path separators for consistent output across platforms
                        .cow_replace('\\', "/")
                        .to_string();

                    let elapsed = start_time.map(|start| start.elapsed());
                    SuccessResult::Changed(display_path, elapsed)
                }
                _ => SuccessResult::Unchanged,
            };
            let _ = tx_success.send(result);
        };

        // A single worker has no other file to start early: keep walk order and buffer nothing.
        if rayon::current_num_threads() == 1 {
            rayon::scope(|_| {
                for plan in rx_entry {
                    process(plan);
                }
            });
            return;
        }

        let queue = LargestFirst::new();
        rayon::in_place_scope(|scope| {
            scope.spawn_broadcast(|_, _| {
                while let Some(plan) = queue.pop() {
                    process(plan);
                }
            });
            // This thread is outside the rayon pool, it only moves entries into the queue.
            for plan in rx_entry {
                queue.push(plan.size, plan);
            }
            queue.finish();
        });
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::LargestFirst;

    #[test]
    fn pops_largest_first_and_ties_in_arrival_order() {
        let queue = LargestFirst::new();
        for (size, item) in [(1, "small"), (9, "big"), (5, "mid-1"), (5, "mid-2")] {
            queue.push(size, item);
        }
        queue.finish();
        let order: Vec<_> = std::iter::from_fn(|| queue.pop()).collect();
        assert_eq!(order, ["big", "mid-1", "mid-2", "small"]);
    }

    #[test]
    fn idle_workers_get_every_entry_and_stop_after_finish() {
        let queue = LargestFirst::new();
        let mut popped: Vec<u64> = thread::scope(|scope| {
            let worker = || {
                scope.spawn(|| {
                    let mut items = vec![];
                    while let Some(item) = queue.pop() {
                        items.push(item);
                    }
                    items
                })
            };
            let workers: Vec<_> = std::iter::repeat_with(worker).take(4).collect();
            for item in 0..100 {
                queue.push(item, item);
            }
            queue.finish();
            workers.into_iter().flat_map(|worker| worker.join().unwrap()).collect()
        });
        popped.sort_unstable();
        assert_eq!(popped, (0..100).collect::<Vec<_>>());
    }
}
