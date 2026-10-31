import { Worker } from "node:worker_threads";

/**
 * Spawn JS plugin workers (Node.js worker threads) to run JS plugins in parallel (`--js-threads` option).
 *
 * Each worker registers its own JS plugin callbacks with Rust (via `registerJsPluginWorker`),
 * and then sends a message to signal that it's ready.
 * The returned `Promise` resolves once all workers are registered.
 *
 * Workers stay alive while Rust holds references to their callbacks.
 * Caller should terminate them once linting is complete.
 *
 * If a worker crashes, the process exits, to avoid Rust-side threads waiting forever for a response.
 *
 * @param count - Number of workers to spawn
 * @returns Array of workers
 */
export function spawnJsPluginWorkers(count: number): Promise<Worker[]> {
  const workerUrl = new URL("./js_plugin_worker.js", import.meta.url);

  const workers: Worker[] = [];
  const readyPromises: Promise<void>[] = [];
  for (let i = 0; i < count; i++) {
    const worker = new Worker(workerUrl);
    workers.push(worker);

    readyPromises.push(
      new Promise((resolve, reject) => {
        worker.once("message", () => resolve());
        worker.once("error", reject);
        // A worker which exits before registering would otherwise leave this `Promise` pending forever
        worker.once("exit", (code) => reject(new Error(`JS plugin worker ${i + 1} exited with code ${code}`)));
      }),
    );

    worker.on("error", (err) => {
      // oxlint-disable-next-line no-console
      console.error(`JS plugin worker ${i + 1} crashed:`, err);
      process.exit(1);
    });
  }

  return Promise.all(readyPromises).then(() => workers);
}
