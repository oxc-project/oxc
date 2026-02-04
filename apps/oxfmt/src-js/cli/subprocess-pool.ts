/**
 * Process pool for Prettier formatting on Node.js v24.
 *
 * Uses child_process.fork() instead of worker_threads to completely avoid
 * the ThreadsafeFunction race condition bug in Node.js v24.
 *
 * Each child process has its own V8 isolate, so there's no shared state
 * that can cause race conditions.
 *
 * See: https://github.com/nodejs/node/issues/55706
 */
import { fork, type ChildProcess } from "node:child_process";
import { fileURLToPath } from "node:url";

interface PendingRequest {
  resolve: (value: unknown) => void;
  reject: (error: Error) => void;
}

interface Worker {
  process: ChildProcess;
  busy: boolean;
  ready: boolean;
  pending: Map<number, PendingRequest>;
  readyPromise: Promise<void>;
  readyResolve: () => void;
}

interface QueuedTask {
  method: string;
  args: unknown;
  resolve: (value: unknown) => void;
  reject: (error: Error) => void;
}

export class SubprocessPool {
  private workers: Worker[] = [];
  private queue: QueuedTask[] = [];
  private nextId = 0;
  private workerPath: string;

  constructor(numWorkers: number) {
    // Resolve path to compiled subprocess script
    // Note: This file gets inlined into cli.js, so path is relative to dist/cli.js
    this.workerPath = fileURLToPath(new URL("./cli/format-subprocess.js", import.meta.url));

    // Spawn workers
    for (let i = 0; i < numWorkers; i++) {
      this.spawnWorker();
    }
  }

  private spawnWorker(): void {
    const proc = fork(this.workerPath, [], {
      // Ensure subprocess inherits env vars (for Prettier plugins, etc.)
      env: process.env,
    });

    let readyResolve!: () => void;
    const readyPromise = new Promise<void>((resolve) => {
      readyResolve = resolve;
    });

    const worker: Worker = {
      process: proc,
      busy: false,
      ready: false,
      pending: new Map(),
      readyPromise,
      readyResolve,
    };

    // Handle responses from worker via IPC
    proc.on(
      "message",
      (msg: { ready?: boolean; id?: number; result?: unknown; error?: string }) => {
        // Handle ready signal
        if (msg.ready) {
          worker.ready = true;
          worker.readyResolve();
          // Process any queued tasks now that this worker is ready
          this.processQueue();
          return;
        }

        const pending = worker.pending.get(msg.id!);
        if (!pending) return;

        worker.pending.delete(msg.id!);
        worker.busy = worker.pending.size > 0;

        if (msg.error) {
          pending.reject(new Error(msg.error));
        } else {
          pending.resolve(msg.result);
        }

        // Process next task in queue
        this.processQueue();
      },
    );

    // Handle worker exit
    proc.on("exit", () => {
      // Reject all pending requests
      for (const pending of worker.pending.values()) {
        pending.reject(new Error("Worker process exited"));
      }
      worker.pending.clear();

      // Remove from workers list
      const index = this.workers.indexOf(worker);
      if (index !== -1) {
        this.workers.splice(index, 1);
      }
    });

    this.workers.push(worker);
  }

  async run(method: string, args: unknown): Promise<unknown> {
    return new Promise((resolve, reject) => {
      // Find available worker that is ready
      const worker = this.workers.find((w) => w.ready && !w.busy);

      if (worker) {
        this.dispatchToWorker(worker, method, args, resolve, reject);
      } else {
        // Queue the task
        this.queue.push({ method, args, resolve, reject });
      }
    });
  }

  private dispatchToWorker(
    worker: Worker,
    method: string,
    args: unknown,
    resolve: (value: unknown) => void,
    reject: (error: Error) => void,
  ): void {
    const id = this.nextId++;
    worker.pending.set(id, { resolve, reject });
    worker.busy = true;

    worker.process.send({ id, method, args });
  }

  private processQueue(): void {
    while (this.queue.length > 0) {
      const worker = this.workers.find((w) => w.ready && !w.busy);
      if (!worker) break;

      const task = this.queue.shift()!;
      this.dispatchToWorker(worker, task.method, task.args, task.resolve, task.reject);
    }
  }

  async destroy(): Promise<void> {
    // Wait for all workers to be ready first
    await Promise.all(this.workers.map((w) => w.readyPromise));

    // Wait for pending work
    const pendingPromises: Promise<void>[] = [];
    for (const worker of this.workers) {
      for (const pending of worker.pending.values()) {
        pendingPromises.push(
          new Promise((resolve) => {
            const originalResolve = pending.resolve;
            const originalReject = pending.reject;
            pending.resolve = (value) => {
              originalResolve(value);
              resolve();
            };
            pending.reject = (error) => {
              originalReject(error);
              resolve();
            };
          }),
        );
      }
    }

    await Promise.all(pendingPromises);

    // Kill all workers
    for (const worker of this.workers) {
      worker.process.kill();
    }
    this.workers = [];
  }
}
