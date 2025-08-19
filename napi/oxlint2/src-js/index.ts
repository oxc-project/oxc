import { Worker } from 'node:worker_threads';
import { lint } from './bindings.js';

import type { Plugin } from './context.ts';

const WORKER_URL = new URL('./worker.js', import.meta.url);

const LOG = false;

// --------------------
// Plugin registration
// --------------------

// Absolute paths of plugins which have been loaded
const registeredPluginPaths = new Set<string>();

// Count of rules registered so far
let registeredRuleCount = 0;

/**
 * Load a plugin.
 *
 * Called from Rust.
 *
 * Main logic is in separate function `loadPluginImpl`, because V8 cannot optimize functions
 * containing try/catch.
 *
 * @param {string} path - Absolute path of plugin file
 * @returns {string} - JSON result
 */
async function loadPlugin(path: string): Promise<string> {
  try {
    return await loadPluginImpl(path);
  } catch (err) {
    return JSON.stringify({ Failure: getErrorMessage(err) });
  }
}

async function loadPluginImpl(path: string): Promise<string> {
  if (registeredPluginPaths.has(path)) {
    return JSON.stringify({
      Failure: 'This plugin has already been registered',
    });
  }

  const { default: plugin } = (await import(path)) as { default: Plugin };

  registeredPluginPaths.add(path);

  // TODO: Use a validation library to assert the shape of the plugin, and of rules
  const offset = registeredRuleCount;
  const ruleNames = Object.keys(plugin.rules);
  registeredRuleCount += ruleNames.length;

  return JSON.stringify({ Success: { name: plugin.meta.name, offset, ruleNames } });
}

/**
 * Get error message from an error.
 *
 * `err` is expected to be an `Error` object, but can be anything.
 *
 * This function will never throw, and always returns a string, even if:
 *
 * * `err` is `null` or `undefined`.
 * * `err` is an object with a getter for `message` property which throws.
 * * `err` has a getter for `message` property which returns a different value each time it's accessed.
 *
 * @param err - Error
 * @returns Error message
 */
function getErrorMessage(err: unknown): string {
  try {
    const { message } = err as undefined | { message: string };
    if (typeof message === 'string' && message !== '') return message;
  } catch {}

  return 'Unknown error';
}

// --------------------
// Worker threads
// --------------------

const workers: Worker[] = [];

/**
 * Initialize worker threads.
 *
 * Called from Rust.
 *
 * @param {number} threadCount - Number of worker threads to create
 * @returns {undefined}
 */
async function initWorkerThreads(threadCount: number): Promise<undefined> {
  // if (LOG) console.log('> Starting', threadCount, 'workers');

  return new Promise((resolve) => {
    let remainingCount = threadCount;
    function done(_: any): void {
      if (--remainingCount === 0) {
        resolve(void 0);
        // if (LOG) console.log('> Started', threadCount, 'workers');
      }
    }

    for (let id = 0; id < threadCount; id++) {
      const worker = new Worker(WORKER_URL, { workerData: { id, LOG } });
      worker.addListener('message', done);
      workers.push(worker);
    }
  });
}

// --------------------
// Run linter
// --------------------

// Call Rust, passing `initWorkerThreads`, and `loadPlugin` as callbacks
const success = await lint(initWorkerThreads, loadPlugin);

// Terminate worker threads
await Promise.all(workers.map(worker => worker.terminate()));

// Note: It's recommended to set `process.exitCode` instead of calling `process.exit()`.
// `process.exit()` kills the process immediately and `stdout` may not be flushed before process dies.
// https://nodejs.org/api/process.html#processexitcode
if (!success) process.exitCode = 1;
