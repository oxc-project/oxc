import {
  emnapiAsyncWorkPlugin as __emnapiAsyncWorkPlugin,
  emnapiTSFNPlugin as __emnapiTSFNPlugin,
  createOnMessage as __wasmCreateOnMessageForFsProxy,
  instantiateNapiModule as __emnapiInstantiateNapiModule,
  WASI as __WASI,
} from '@napi-rs/wasm-runtime'
import { createContext as __emnapiCreateContext } from '@emnapi/runtime'


export const __napiBindingTarget = 'wasm32-wasi'
function __napiStampBindingTarget(exportsObject, target) {
  if (
    Object.prototype.hasOwnProperty.call(exportsObject, '__napiBindingTarget')
  ) {
    if (exportsObject.__napiBindingTarget === target) {
      // Already ours: the root entry aliases the object it loaded, so a WASI
      // fallback candidate — or a `NAPI_RS_NATIVE_LIBRARY_PATH` override that
      // is a generated loader — arrives already stamped with this same value.
      return target
    }
    const error = new Error(
      '`__napiBindingTarget` is reserved by the generated binding loader, but the loaded binding already exports it. Rename the export, e.g. #[napi(js_name = "...")].',
    )
    error.code = 'ERR_NAPI_BINDING_TARGET_CONFLICT'
    throw error
  }
  if (!Object.isExtensible(exportsObject)) {
    // A `#[napi(module_exports)]` hook may seal or freeze this object
    // (`Object::seal` / `Object::freeze`). Reporting the artifact is metadata,
    // never a reason to fail an otherwise successful load, so the stamp is
    // skipped. What a consumer still sees then follows the entry point: the
    // browser and deferred loaders declare `__napiBindingTarget` at module
    // level and go on reporting it, while the CommonJS entries hand back this
    // very object as `module.exports`, so there the value is absent.
    return target
  }
  try {
    // [[Define]], not [[Set]]: an ordinary assignment walks the prototype
    // chain, so an inherited accessor could swallow the value or throw and
    // fail an otherwise successful load. The descriptor is what a successful
    // assignment would have produced.
    Object.defineProperty(exportsObject, '__napiBindingTarget', {
      configurable: true,
      enumerable: true,
      value: target,
      writable: true,
    })
  } catch {
    // Same rule as the non-extensible skip above: reporting the artifact is
    // metadata, never a reason to fail an otherwise successful load. An exotic
    // object (a Proxy whose defineProperty trap refuses) is skipped, not
    // thrown over.
  }
  // The CommonJS loaders assign this return value so `cjs-module-lexer` — and
  // therefore Node's CJS -> ESM named export detection — can see
  // `__napiBindingTarget` statically.
  return target
}

const __wasi = new __WASI({
  version: 'preview1',
})

const __wasmUrl = new URL('./minify.wasm32-wasi.wasm', import.meta.url).href
const __wasmResponse = await globalThis.fetch(__wasmUrl)
if (!__wasmResponse.ok) {
  throw new Error(
    'Failed to fetch WASI module ' +
      __wasmUrl +
      ': ' +
      __wasmResponse.status +
      ' ' +
      (__wasmResponse.statusText || 'Unknown Status'),
  )
}
const __wasmFile = await __wasmResponse.arrayBuffer()

const __sharedMemory = new WebAssembly.Memory({
  initial: 4000,
  maximum: 65536,
  shared: true,
})
const __asyncWorkPoolSize = 4
const __workerPoolSize = Math.max(
  2,
  globalThis.navigator?.hardwareConcurrency ?? 4,
)

let __emnapiContext

const __wasiDisposeSymbol = Symbol.for('napi.rs.wasi.dispose')
const __wasiWorkers = new Set()
// The thread manager has to be reachable *before* anything that can throw
// during load or registration. Initialization can fail after the pool has
// already spawned workers, and the rollback still has to mark their
// terminations as expected — but `__napiModule` is assigned only when
// instantiation RETURNS, so on exactly that path it is still undefined. A
// plugin factory runs while the emnapi module is being created, before the
// wasm is loaded and before any registration function runs, and its context
// carries the very same manager instance.
let __wasiThreadManager

function __captureWasiThreadManager(context) {
  if (context && context.PThread) {
    __wasiThreadManager = context.PThread
  }
  return {}
}

function __getWasiThreadManager() {
  const manager =
    __wasiThreadManager !== undefined
      ? __wasiThreadManager
      : __napiModule
        ? __napiModule.PThread
        : undefined
  if (manager && typeof manager.terminateWorker === 'function') {
    return manager
  }
  return undefined
}
let __napiInstance
let __emnapiContextDestroyed = false
let __emnapiContextDestroyPromise
let __emnapiWasmEnvCleanupPrepared = false
let __emnapiWasmEnvCleanupPreparing = false
let __emnapiWasmEnvCleanupRan = false
let __emnapiWasmEnvCleanupDrained = false
let __emnapiWasmEnvCleanupDrainPromise
let __wasiDisposed = false
let __wasiAsyncWorkDrainPromise
let __wasiDisposePromise
let __completeWasiDisposal = function () {}
// Overridden by loader flavors that have a last-resort reclaim for a rollback
// that stopped short of destroying the context. See
// `__rollbackWasiInitialization`.
let __retainWasiRollbackForRetry = function () {}

function __isThenable(value) {
  return (
    value !== null &&
    (typeof value === 'object' || typeof value === 'function') &&
    typeof value.then === 'function'
  )
}

function __createCleanupError(errors, message) {
  if (errors.length === 1) {
    return errors[0]
  }
  const __AggregateError = globalThis.AggregateError
  if (typeof __AggregateError === 'function') {
    return new __AggregateError(errors, message)
  }
  const error = new Error(message)
  error.errors = errors
  return error
}

function __attachCleanupErrors(error, cleanupErrors) {
  if (cleanupErrors.length === 0) {
    return error
  }
  const cleanupError = __createCleanupError(
    cleanupErrors,
    'WASI binding cleanup failed',
  )
  try {
    if (
      error &&
      (typeof error === 'object' || typeof error === 'function')
    ) {
      if (error.cause === undefined) {
        error.cause = cleanupError
        if (error.cause === cleanupError) {
          return error
        }
      }
      if (Array.isArray(error.cleanupErrors)) {
        error.cleanupErrors.push(cleanupError)
        return error
      } else {
        const attachedCleanupErrors = [cleanupError]
        error.cleanupErrors = attachedCleanupErrors
        if (error.cleanupErrors === attachedCleanupErrors) {
          return error
        }
      }
    }
  } catch {}
  const aggregate = __createCleanupError(
    [error, cleanupError],
    'WASI binding initialization and cleanup failed',
  )
  try {
    aggregate.cause = error
  } catch {}
  return aggregate
}

function __wrapEmnapiContextDestroyForSettlement(
  context,
  prepareEnvCleanup,
  isPreparingEnvCleanup,
) {
  let destroy
  try {
    destroy = context.destroy
  } catch {
    return context
  }
  if (typeof destroy !== 'function') {
    return context
  }
  try {
    Object.defineProperty(context, 'destroy', {
      configurable: true,
      enumerable: false,
      writable: true,
      value: function () {
        // Reentered from a promise hook that fired inside the barrier: the
        // frame running it destroys as soon as it returns.
        if (isPreparingEnvCleanup?.()) {
          return
        }
        prepareEnvCleanup?.()
        return Reflect.apply(destroy, this, arguments)
      },
    })
  } catch {}
  return context
}

function __isPreparingWasmEnvCleanup() {
  return __emnapiWasmEnvCleanupPreparing
}

function __prepareWasmEnvCleanup() {
  if (__emnapiWasmEnvCleanupPrepared || __emnapiWasmEnvCleanupPreparing) {
    return
  }
  const prepare = __napiInstance?.exports?.napi_prepare_wasm_env_cleanup
  if (typeof prepare === 'function') {
    // The addon settles the promises it cancels synchronously, under a
    // non-reentrant lifecycle mutex: anything a promise hook calls from in
    // here must not reach this export again.
    __emnapiWasmEnvCleanupPreparing = true
    try {
      prepare()
    } finally {
      __emnapiWasmEnvCleanupPreparing = false
    }
    __emnapiWasmEnvCleanupRan = true
  }
  __emnapiWasmEnvCleanupPrepared = true
}

// Mirror the primitive @emnapi/core schedules its threadsafe-function dispatch
// on, so the drain turns below interleave with that dispatch instead of racing
// ahead of it on a faster queue.
const __scheduleMacrotask = (function () {
  if (typeof setImmediate === 'function') {
    return function (callback) {
      setImmediate(callback)
    }
  }
  const __MessageChannel = globalThis.MessageChannel
  if (typeof __MessageChannel === 'function') {
    return function (callback) {
      const channel = new __MessageChannel()
      channel.port1.onmessage = function () {
        channel.port1.onmessage = null
        try {
          channel.port1.close()
        } catch {}
        try {
          channel.port2.close()
        } catch {}
        callback()
      }
      channel.port2.postMessage(null)
    }
  }
  return function (callback) {
    setTimeout(callback, 0)
  }
})()

// A real, *referenced* timer, for waits that must let the whole host make
// progress between looks — the async-work drain polls the addon rather than
// interleaving with the @emnapi/core dispatch, so a zero-delay macrotask there
// would spin the loop instead of yielding it. Falls back to the macrotask
// scheduler on a host without timers.
function __scheduleTimer(callback, delay) {
  const setTimer = globalThis.setTimeout
  if (typeof setTimer !== 'function') {
    __scheduleMacrotask(callback)
    return
  }
  try {
    setTimer(callback, delay)
  } catch {
    __scheduleMacrotask(callback)
  }
}

// Turns to wait for while the addon still reports queued settlements. Reaching
// zero is the only success. A counter still nonzero at this bound rejects the
// disposal as retryable (`ERR_NAPI_WASI_CLEANUP_PENDING`) rather than
// destroying the context over a still-queued settlement — the wait stays
// bounded either way.
const __WASM_ENV_CLEANUP_DRAIN_TURNS = 128
// Without `napi_wasm_env_cleanup_pending` the queue is not observable. Fall
// back to the number of turns @emnapi/core needs to coalesce and dispatch a
// call made on this thread (two), plus a margin.
const __WASM_ENV_CLEANUP_BLIND_DRAIN_TURNS = 4

/**
 * `napi_prepare_wasm_env_cleanup` only *queues* the promise settlements of the
 * tasks it cancelled: `napi_call_threadsafe_function` appends to the
 * threadsafe-function queue, and @emnapi/core dispatches that queue from a
 * macrotask — two coalescing turns later, even for a call made on this very
 * thread. `Context.destroy()` then runs the threadsafe function's cleanup hook,
 * which drains the queue with a null env and *discards* whatever is still in it.
 *
 * So destroying without yielding first strands exactly the promises the barrier
 * exists to settle. Yield real event-loop turns until the addon reports the
 * queue empty; microtask checkpoints cannot help, no number of them lets a
 * macrotask run.
 *
 * Returns nothing when there is nothing to wait for, which keeps disposal
 * synchronous in the common case.
 *
 * The "already drained" flag is set only once a wait has actually finished.
 * Scheduling a macrotask can fail — a host-provided or patched `setImmediate`
 * that throws is enough — and a disposal that rejects stays retryable, so
 * marking the drain complete up front would make the retry skip it and destroy
 * the context with the barrier's settlements still queued.
 *
 * A wait that runs out of turns with the counter still nonzero rejects with
 * `ERR_NAPI_WASI_CLEANUP_PENDING` for the same reason: at that point
 * "finished" is indistinguishable from the stranding above, and destroying
 * would discard the very settlement the wait was for. The rejection leaves the
 * flag unset and disposal retryable.
 */
function __drainWasmEnvCleanup() {
  if (__emnapiWasmEnvCleanupDrained || !__emnapiWasmEnvCleanupRan) {
    return
  }
  if (__emnapiWasmEnvCleanupDrainPromise) {
    return __emnapiWasmEnvCleanupDrainPromise
  }
  const pending = __napiInstance?.exports?.napi_wasm_env_cleanup_pending
  const observable = typeof pending === 'function'
  if (observable) {
    let queued
    try {
      queued = pending()
    } catch {
      __emnapiWasmEnvCleanupDrained = true
      return
    }
    if (!queued) {
      __emnapiWasmEnvCleanupDrained = true
      return
    }
  }
  const limit = observable
    ? __WASM_ENV_CLEANUP_DRAIN_TURNS
    : __WASM_ENV_CLEANUP_BLIND_DRAIN_TURNS
  const drainPromise = (async () => {
    let queued = 0
    for (let turn = 0; turn < limit; turn++) {
      await new Promise((resolve) => {
        __scheduleMacrotask(resolve)
      })
      if (!observable) {
        continue
      }
      try {
        queued = pending()
      } catch {
        return
      }
      if (!queued) {
        return
      }
    }
    if (!observable) {
      // Blind wait: without `napi_wasm_env_cleanup_pending` the bound IS the
      // contract — there is nothing to consult, so finishing the turns is
      // finishing the drain.
      return
    }
    // The counter is still nonzero after every turn the bound allows. The wait
    // stays bounded — but claiming success here would be indistinguishable from
    // the stranding this drain exists to prevent: disposal would go on to
    // destroy the context, whose cleanup hook discards the still-queued
    // settlement with a null env, and the promise it was for hangs forever.
    // Reject instead, as a retryable cleanup failure: the drained flag stays
    // unset, dispose() (and the rollback) decline to destroy, and a later
    // dispose() runs the drain again — by which time the queue has usually been
    // delivered. A counter that is somehow stuck nonzero therefore costs each
    // attempt at most another bounded wait and a rejection, never a stranded
    // promise; the process-exit teardown still reclaims the context.
    const drainError = new Error(
      'the wasm environment still reports ' +
        queued +
        ' queued settlement(s) after ' +
        limit +
        ' event-loop turns; the context was not destroyed - retry dispose() to wait for the queue again',
    )
    drainError.code = 'ERR_NAPI_WASI_CLEANUP_PENDING'
    throw drainError
  })().then(
    (value) => {
      // Set only when the wait actually finished AND the queue was seen empty
      // (or is unobservable): a drain that timed out with settlements still
      // queued rejects above and must stay repeatable.
      __emnapiWasmEnvCleanupDrained = true
      __emnapiWasmEnvCleanupDrainPromise = undefined
      return value
    },
    (error) => {
      __emnapiWasmEnvCleanupDrainPromise = undefined
      throw error
    },
  )
  __emnapiWasmEnvCleanupDrainPromise = drainPromise
  return drainPromise
}

function __destroyEmnapiContext() {
  if (__emnapiContextDestroyed || __emnapiContext === undefined) {
    __emnapiContextDestroyed = true
    return
  }
  if (__emnapiContextDestroyPromise) {
    return __emnapiContextDestroyPromise
  }

  __prepareWasmEnvCleanup()
  const result = __emnapiContext.destroy()
  if (!__isThenable(result)) {
    __emnapiContextDestroyed = true
    return
  }

  const destroyPromise = Promise.resolve(result).then(
    (value) => {
      __emnapiContextDestroyed = true
      return value
    },
    (error) => {
      __emnapiContextDestroyPromise = undefined
      throw error
    },
  )
  __emnapiContextDestroyPromise = destroyPromise
  return destroyPromise
}

/**
 * Holds the event loop open until `work` settles.
 *
 * Nothing else can: the pool workers are deliberately unreferenced so an idle
 * binding cannot keep a process alive, and referencing them again for the
 * termination does not hold either — emnapi unreferences a worker the moment it
 * reports `async-thread-ready`, which for a worker that was still starting
 * lands *after* the termination began. Without a handle of its own, an
 * `await dispose()` with nothing else pending exits the process with its
 * promise unsettled, and everything after the `await` is skipped.
 *
 * The timer is cleared as soon as the work settles, so this never outlives the
 * disposal that asked for it.
 */
function __keepEventLoopAliveUntil(work) {
  const setTimer = globalThis.setInterval
  const clearTimer = globalThis.clearInterval
  if (typeof setTimer !== 'function' || typeof clearTimer !== 'function') {
    return work
  }
  let timer
  try {
    timer = setTimer(function () {}, 50)
  } catch {
    return work
  }
  const release = function () {
    try {
      clearTimer(timer)
    } catch {}
  }
  return work.then(
    (value) => {
      release()
      return value
    },
    (error) => {
      release()
      throw error
    },
  )
}

// How often to re-read `napi_wasm_async_work_pending` while waiting. The wait
// ends when the addon reports zero, so this only decides how promptly disposal
// notices — not how long it waits.
const __WASI_ASYNC_WORK_POLL_INTERVAL_MS = 1

/**
 * Settles this addon's outstanding `napi_async_work` before the teardown that
 * would strand it.
 *
 * `napi_prepare_wasm_env_cleanup` does not cover async work, and nothing about
 * it is observable from JavaScript: the threadless archive resolves
 * `napi_*_async_work` through the `@emnapi/core` plugins, but the threaded one
 * links the C `async_work.c` on the uv threadpool, so there the wasm neither
 * imports nor exports those symbols and the only brackets a loader could watch
 * (`_emnapi_ctx_*_waiting_request_counter`) are shared with threadsafe
 * functions. The addon is the one place both flavors go through, so it answers
 * for both, through the same kind of handshake the settlement drain uses:
 *
 *   - `napi_wasm_cancel_pending_async_work()` cancels what no thread has
 *     started. Those completion callbacks run with `napi_cancelled`, which
 *     napi-rs turns into a promise rejected with an `AbortError`.
 *   - `napi_wasm_async_work_pending()` counts what is still owed a completion
 *     callback. Work already executing refuses cancellation and stays counted
 *     until it finishes normally — which it can, because this runs before the
 *     barrier, before `Context.destroy()` and before anything is terminated.
 *
 * Both exports are optional: an addon built against a napi crate that predates
 * them drains nothing and keeps the previous behavior, exactly as the
 * `napi_wasm_env_cleanup_pending` handshake degrades.
 *
 * Returns nothing when there is nothing outstanding, which keeps disposal
 * synchronous in the common case. The promise it returns otherwise never
 * rejects.
 *
 * The wait has no deadline, and that is the point: giving up would destroy the
 * environment with a completion callback still owed, which is the stranding
 * this exists to prevent. A task whose `execute` never returns already keeps an
 * *undisposed* process alive in exactly the same way, so disposal inherits that
 * rather than inventing a bound it cannot honor.
 *
 * Safe to call from inside a completion callback, which is reachable: settling
 * a task runs addon code that can re-enter JavaScript — a setter on the value
 * being handed back, a threadsafe-function callback — and that JavaScript can
 * call `dispose()`. Two things make it terminate rather than wait on itself:
 *
 *   - The addon keeps a work registered until its completion callback
 *     *finishes*, so the count read here is at least one and this takes the
 *     polling path instead of declaring the environment drained and tearing it
 *     down from inside the frame that is still settling a promise.
 *   - The poll is a timer, so it cannot run until the callback has returned to
 *     the host — by which time that work has left the registry. The count the
 *     next poll reads is the one taken after the callback finished.
 *
 * `__disposeWasiBinding` hands every caller the same in-flight promise, so the
 * nested call joins this disposal rather than starting a second one.
 */
function __drainWasiAsyncWork() {
  if (__wasiAsyncWorkDrainPromise !== undefined) {
    return __wasiAsyncWorkDrainPromise
  }
  const exports = __napiInstance?.exports
  const pending = exports?.napi_wasm_async_work_pending
  const cancelPending = exports?.napi_wasm_cancel_pending_async_work
  if (typeof pending !== 'function' || typeof cancelPending !== 'function') {
    return
  }

  const readPending = () => {
    try {
      return pending()
    } catch (error) {
      // A trap is the only way this call fails: it reads a counter and cannot
      // allocate or call back into JavaScript. A trapped instance can no longer
      // run anything, so its outstanding work is unreachable by definition —
      // there is nothing left to wait for, and refusing to dispose would only
      // keep a dead instance and its stuck counter alive. Best-effort here is
      // the honest answer, and it is what disposal did before this drain
      // existed.
      //
      // Only a trap. Anything else means the export is not what this loader
      // thinks it is, which is a defect worth surfacing rather than disposing
      // over.
      if (error instanceof globalThis.WebAssembly.RuntimeError) {
        return 0
      }
      throw error
    }
  }

  if (!readPending()) {
    return
  }
  try {
    cancelPending()
  } catch {
    // Cancellation is an optimization: it bounds the wait by the work already
    // executing. Failing it only means waiting for the whole queue instead.
  }
  if (!readPending()) {
    return
  }

  const drainPromise = __keepEventLoopAliveUntil(
    (async () => {
      while (readPending()) {
        await new Promise((resolve) => {
          __scheduleTimer(resolve, __WASI_ASYNC_WORK_POLL_INTERVAL_MS)
        })
      }
    })(),
  ).then(
    () => {
      __wasiAsyncWorkDrainPromise = undefined
    },
    (error) => {
      // A wait that could not run is not a wait that finished. The only way
      // here is a host whose timers and macrotask primitives all refuse, and
      // the work is still outstanding — reporting success would destroy the
      // environment over it, which is the stranding this exists to prevent.
      // Reject instead: disposal stays retryable, and the context is not
      // destroyed. Clearing the memo first is what makes the retry re-run this.
      __wasiAsyncWorkDrainPromise = undefined
      throw error
    },
  )
  __wasiAsyncWorkDrainPromise = drainPromise
  return drainPromise
}

/**
 * `@emnapi/wasi-threads` counts a worker exit as expected only when its own
 * thread manager performed the termination. A bare `worker.terminate()` reaches
 * the manager's `exit` listener instead, which reports
 * `worker (tid = N) sent an error! ... stopped with exit code 1` and rethrows
 * inside the emit — aborting the `once('exit')` that backs the terminate
 * promise, so disposal never settles and the process dies with an uncaught
 * exception. Mark the termination through the manager first.
 *
 * The manager comes from `__getWasiThreadManager`, not from `__napiModule`:
 * the initialization rollback runs on the one path where instantiation never
 * returned, so `__napiModule` is still undefined there while the workers it
 * spawned are already registered and loaded.
 *
 * Not `terminateAllThreads()`: that one recreates the pool it just shut down.
 */
function __terminateWasiWorkers() {
  const cleanupErrors = []
  const pending = []
  const threadManager = __getWasiThreadManager()

  for (const worker of __wasiWorkers) {
    let result
    try {
      if (threadManager) {
        threadManager.terminateWorker(worker)
        // `terminateWorker` leaves behind a reporter that logs every message
        // still queued on the port, which Node flushes on exit. Nothing is
        // listening for those any more.
        worker.onmessage = undefined
      }
      result = worker.terminate()
    } catch (error) {
      cleanupErrors.push(error)
      continue
    }
    if (__isThenable(result)) {
      pending.push(
        Promise.resolve(result).then(
          () => {
            __wasiWorkers.delete(worker)
          },
          (error) => {
            cleanupErrors.push(error)
          },
        ),
      )
    } else {
      __wasiWorkers.delete(worker)
    }
  }

  const finish = () => {
    if (cleanupErrors.length > 0) {
      throw __createCleanupError(
        cleanupErrors,
        'Failed to terminate WASI workers',
      )
    }
  }
  return pending.length > 0
    ? __keepEventLoopAliveUntil(Promise.all(pending)).then(finish)
    : finish()
}

function __finishWasiDisposal() {
  const workerResult = __terminateWasiWorkers()
  if (__isThenable(workerResult)) {
    return Promise.resolve(workerResult).then(__completeWasiDisposal)
  }
  return __completeWasiDisposal()
}

function __continueWasiDisposal() {
  const destroyResult = __destroyEmnapiContext()
  if (__isThenable(destroyResult)) {
    return Promise.resolve(destroyResult).then(__finishWasiDisposal)
  }
  return __finishWasiDisposal()
}

function __cleanUpWasmEnvForWasiDisposal() {
  // Run the pre-teardown barrier, then let the settlements it queued actually
  // reach JavaScript, and only then destroy the environment. Doing these two
  // back to back is what strands them.
  __prepareWasmEnvCleanup()
  const drainResult = __drainWasmEnvCleanup()
  if (__isThenable(drainResult)) {
    return Promise.resolve(drainResult).then(__continueWasiDisposal)
  }
  return __continueWasiDisposal()
}

function __startWasiDisposal() {
  // Outstanding `napi_async_work` goes first, while the environment is still
  // completely live: the completion callbacks run addon code, and everything
  // after this point takes that away from them — the barrier shuts the async
  // runtime down, `Context.destroy()` stops JavaScript calls, and terminating
  // the pool threads removes what would have reported the work finished.
  const asyncWorkResult = __drainWasiAsyncWork()
  if (__isThenable(asyncWorkResult)) {
    return Promise.resolve(asyncWorkResult).then(
      __cleanUpWasmEnvForWasiDisposal,
    )
  }
  return __cleanUpWasmEnvForWasiDisposal()
}

/**
 * Disposes this generated WASI binding.
 *
 * Access this function with:
 * binding[Symbol.for('napi.rs.wasi.dispose')]()
 */
function __disposeWasiBinding() {
  if (__wasiDisposePromise) {
    return __wasiDisposePromise
  }
  if (__wasiDisposed) {
    return Promise.resolve()
  }

  let resolveDispose
  let rejectDispose
  const disposePromise = new Promise((resolve, reject) => {
    resolveDispose = resolve
    rejectDispose = reject
  })
  __wasiDisposePromise = disposePromise

  let result
  try {
    result = __startWasiDisposal()
  } catch (error) {
    __wasiDisposePromise = undefined
    rejectDispose(error)
    return disposePromise
  }

  Promise.resolve(result).then(
    (value) => {
      __wasiDisposed = true
      resolveDispose(value)
    },
    (error) => {
      __wasiDisposePromise = undefined
      rejectDispose(error)
    },
  )
  return disposePromise
}

function __publishWasiDispose(exports) {
  Object.defineProperty(exports, __wasiDisposeSymbol, {
    configurable: false,
    enumerable: false,
    value: __disposeWasiBinding,
    writable: false,
  })
}

function __finishWasiInitializationRollback(cleanupErrors) {
  let workerResult
  try {
    workerResult = __terminateWasiWorkers()
  } catch (cleanupError) {
    cleanupErrors.push(cleanupError)
    return cleanupErrors
  }
  if (__isThenable(workerResult)) {
    return Promise.resolve(workerResult)
      .catch((cleanupError) => {
        cleanupErrors.push(cleanupError)
      })
      .then(() => cleanupErrors)
  }
  return cleanupErrors
}

function __destroyContextForWasiRollback(cleanupErrors) {
  let destroyResult
  try {
    destroyResult = __destroyEmnapiContext()
  } catch (cleanupError) {
    cleanupErrors.push(cleanupError)
    return __finishWasiInitializationRollback(cleanupErrors)
  }
  if (__isThenable(destroyResult)) {
    return Promise.resolve(destroyResult)
      .catch((cleanupError) => {
        cleanupErrors.push(cleanupError)
      })
      .then(() => __finishWasiInitializationRollback(cleanupErrors))
  }
  return __finishWasiInitializationRollback(cleanupErrors)
}

/**
 * Leaves a rollback that could not reach the queued settlements undestroyed, and
 * hands it to whatever this flavor has that can still reclaim it.
 */
function __retainFailedWasiRollback(cleanupErrors) {
  try {
    __retainWasiRollbackForRetry()
  } catch (cleanupError) {
    cleanupErrors.push(cleanupError)
  }
  return cleanupErrors
}

/**
 * Initialization can fail *after* registration has already run, and registration
 * runs with a live environment: a module-init hook can start async work and then
 * return an error, and the promise it created may already have escaped into
 * JavaScript. The barrier cancels that work and *queues* the settlement, so this
 * path needs the same drain the ordinary disposal does — destroying without
 * yielding discards the queue with a null env and strands the promise.
 *
 * Stays synchronous when nothing is queued, which covers every failure before
 * `beforeInit`: there is no instance to run the barrier on, so nothing to drain.
 *
 * A barrier or drain that did *not* finish stops the rollback short of
 * destroying, which is what `dispose()` already does — a rejected drain there
 * never reaches `__continueWasiDisposal`. Destroying anyway is the worse of the
 * two trades, and not because of what it saves:
 *
 *   - It cannot deliver the settlements. `Context.destroy()` runs the
 *     threadsafe function's cleanup hook, which drains the queue with a null env
 *     and discards it, so a promise that already escaped into JavaScript hangs
 *     forever with nothing left that could ever settle it.
 *   - It saves less than it looks. `Context.destroy()` stops JavaScript calls
 *     and runs cleanup hooks; it does not free the wasm instance or its Memory,
 *     which this module's scope holds either way. What stopping short retains is
 *     the emnapi context's bookkeeping and its un-run cleanup hooks.
 *   - Retry is not theoretical. A rollback that records a cleanup error is
 *     already kept in the process-wide registry above, so re-`require()`ing this
 *     file replays it instead of re-instantiating — and the `6e15de6f` flag fix
 *     means the replay drains again rather than skipping it. Destroying first is
 *     what makes that retained record useless.
 *
 * The residual cost is honest: the CJS flavor hands the context to its
 * `process.on('exit')` teardown, so a process that never retries still reclaims
 * it on the way out. The ESM browser flavor has no equivalent — a module that
 * throws while evaluating is permanently errored, so re-importing rethrows
 * without re-running this file — and there the context stays until the realm
 * goes away. That is the deliberate choice: a hung promise is a silent liveness
 * bug with no upper bound, while the retained bookkeeping is bounded by the page.
 */
function __rollbackWasiInitialization() {
  // The environment teardown this rollback performs, kept nested so it cannot
  // be reached without the async-work drain below running first.
  function __rollbackWasmEnvForWasiInitialization() {
    const cleanupErrors = []
    let drainResult
    let settlementsUnreached = false
    try {
      __prepareWasmEnvCleanup()
      drainResult = __drainWasmEnvCleanup()
    } catch (cleanupError) {
      cleanupErrors.push(cleanupError)
      settlementsUnreached = true
    }
    if (__isThenable(drainResult)) {
      return Promise.resolve(drainResult).then(
        () => __destroyContextForWasiRollback(cleanupErrors),
        (cleanupError) => {
          cleanupErrors.push(cleanupError)
          return __retainFailedWasiRollback(cleanupErrors)
        },
      )
    }
    if (settlementsUnreached) {
      return __retainFailedWasiRollback(cleanupErrors)
    }
    return __destroyContextForWasiRollback(cleanupErrors)
  }

  // Same reason as `__startWasiDisposal`: a module-init hook can start async
  // work before the load goes on to fail, and this rollback tears down exactly
  // what those completions need. Settle them while everything is still live,
  // before the barrier and the teardown above take that away.
  //
  // A drain that could not finish leaves async work possibly outstanding, and
  // destroying the context over it would strand exactly what this rollback is
  // there to settle. Stop short and retain instead — the same trade
  // `__rollbackWasmEnvForWasiInitialization` makes for the settlement drain, so
  // the context stays reclaimable by a retry or by this flavor's own
  // last-resort teardown.
  const __retainAfterAsyncWorkDrainFailure = (cleanupError) =>
    __retainFailedWasiRollback([cleanupError])
  let asyncWorkResult
  try {
    asyncWorkResult = __drainWasiAsyncWork()
  } catch (cleanupError) {
    return __retainAfterAsyncWorkDrainFailure(cleanupError)
  }
  if (__isThenable(asyncWorkResult)) {
    return Promise.resolve(asyncWorkResult).then(
      __rollbackWasmEnvForWasiInitialization,
      __retainAfterAsyncWorkDrainFailure,
    )
  }
  return __rollbackWasmEnvForWasiInitialization()
}

let __wasiModule
let __napiModule

try {
  __emnapiContext = __wrapEmnapiContextDestroyForSettlement(
    __emnapiCreateContext({ autoDestroy: false }),
    __prepareWasmEnvCleanup,
    __isPreparingWasmEnvCleanup,
  )
  __emnapiContext.suppressDestroy()
  
  ;({
    instance: __napiInstance,
    module: __wasiModule,
    napiModule: __napiModule,
  } = await __emnapiInstantiateNapiModule(__wasmFile, {
    context: __emnapiContext,
    asyncWorkPoolSize: __asyncWorkPoolSize,
    reuseWorker: false,
    plugins: [
      __captureWasiThreadManager,
      __emnapiAsyncWorkPlugin,
      __emnapiTSFNPlugin,
    ],
    wasi: __wasi,
    onCreateWorker() {
      const worker = new Worker(new URL('./wasi-worker-browser.mjs', import.meta.url), {
        type: 'module',
      })
      __wasiWorkers.add(worker)


      return worker
    },
    overwriteImports(importObject) {
      importObject.env = {
        ...importObject.env,
        ...importObject.napi,
        ...importObject.emnapi,
        memory: __sharedMemory,
      }
      return importObject
    },
    beforeInit({ instance }) {
      __napiInstance = instance
      for (const name of Object.keys(instance.exports)) {
        if (name.startsWith('__napi_register__')) {
          instance.exports[name]()
        }
      }
    },
  }))
  __publishWasiDispose(__napiModule.exports)
  // The default export hands out this object; a named module export does not
  // travel with it, so carry the marker on the binding itself too. After the
  // host install, which hands the same object to addon-provided registration
  // functions that may put anything on it, and inside this `try`, so a claimed
  // name fails the load through the rollback below rather than past it.
  __napiStampBindingTarget(__napiModule.exports, __napiBindingTarget)
} catch (error) {
  const cleanupErrors = await __rollbackWasiInitialization()
  throw __attachCleanupErrors(error, cleanupErrors)
}
export default __napiModule.exports
export const LegalCommentsMode = __napiModule.exports.LegalCommentsMode
export const minify = __napiModule.exports.minify
export const minifySync = __napiModule.exports.minifySync
export const Severity = __napiModule.exports.Severity
