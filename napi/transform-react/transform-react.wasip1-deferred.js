import {
  emnapiAsyncWorkPlugin as __emnapiAsyncWorkPlugin,
  emnapiTSFNPlugin as __emnapiTSFNPlugin,
  instantiateNapiModule as __emnapiInstantiateNapiModule,
  WASI as __WASI,
} from '@napi-rs/wasm-runtime'
import { createContext as __emnapiCreateContext } from '@emnapi/runtime'


export const WASM_MEMORY = Object.freeze({
  initialPages: 1024,
  maximumPages: 65536,
  pageBytes: 65536,
  initialBytes: 1024 * 65536,
  maximumBytes: 65536 * 65536,
})

let __createdInstances = 0
let __liveInstances = 0

/**
 * Counters for instances created by THIS module evaluation, not process-wide:
 * a second bundled copy of this loader keeps its own. Only successfully
 * created instances are counted, and `liveInstances` drops when an instance's
 * `dispose()` resolves.
 *
 * `declaredInitialMemoryBytes` is declared address space, not a host's
 * committed-memory metric; pair it with host telemetry rather than treating it
 * as a quota.
 */
export function getDeferredRuntimeStats() {
  return Object.freeze({
    createdInstances: __createdInstances,
    liveInstances: __liveInstances,
    declaredInitialMemoryBytes: WASM_MEMORY.initialBytes,
  })
}

const __arrayBufferByteLengthGetter = Object.getOwnPropertyDescriptor(
  ArrayBuffer.prototype,
  'byteLength',
).get
const __memoryBufferGetter = Object.getOwnPropertyDescriptor(
  WebAssembly.Memory.prototype,
  'buffer',
).get
// One managed initialization per Memory, success or failure: an attempt that
// throws may already have written into linear memory, so the bytes are not a
// clean slate for a second instance. Module-local, like the counters above.
const __claimedMemories = new WeakSet()

function __resolveInstanceMemory(__options) {
  const __provided = __options == null ? undefined : __options.memory
  if (__provided === undefined || __provided === null) {
    // Page counts are handed to the engine unvalidated: it already rejects a
    // negative, over-4GiB or below-maximum value with a precise message, and a
    // second set of bounds here would only drift from it.
    const __allocated = new WebAssembly.Memory({
      initial:
        __options != null && __options.initialMemoryPages !== undefined
          ? __options.initialMemoryPages
          : WASM_MEMORY.initialPages,
      maximum:
        __options != null && __options.maximumMemoryPages !== undefined
          ? __options.maximumMemoryPages
          : WASM_MEMORY.maximumPages,
    })
    // Claimed like a caller-provided one. The handle publishes it as
    // `instance.memory`, so handing it back to `createInstance()` is as easy
    // as passing your own twice, and it would put two live instances on one
    // linear memory: each initialization rewrites the emnapi/WASI state the
    // other is still running on.
    __claimedMemories.add(__allocated)
    return __allocated
  }
  if (
    __options.initialMemoryPages !== undefined ||
    __options.maximumMemoryPages !== undefined
  ) {
    throw new TypeError(
      'Pass either memory or initialMemoryPages/maximumMemoryPages, not both',
    )
  }
  let __buffer
  try {
    // Brand check: the getter throws for anything that is not a genuine
    // WebAssembly.Memory, including a cross-realm look-alike object.
    __buffer = Reflect.apply(__memoryBufferGetter, __provided, [])
  } catch {
    throw new TypeError('memory must be an unshared WebAssembly.Memory')
  }
  try {
    // Throws for a SharedArrayBuffer. This loader has no threads, and shared
    // growth does not detach: external views handed to the addon would
    // silently outlive the bytes they describe.
    Reflect.apply(__arrayBufferByteLengthGetter, __buffer, [])
  } catch {
    throw new TypeError(
      'The deferred loader requires an unshared WebAssembly.Memory',
    )
  }
  // The intrinsic getters above accept a genuine Memory from ANY realm, but
  // the loader's dependencies do not: `WASI.setMemory` in
  // `@napi-rs/wasm-runtime` and emnapi identify a Memory with a realm-local
  // `instanceof`. A Memory built in another realm (a `node:vm` context, a
  // same-origin iframe) would pass every check here and only fail deep inside
  // initialization. Reject it up front, and before the claim below, so the
  // caller keeps it usable in the realm that made it.
  if (!(__provided instanceof WebAssembly.Memory)) {
    throw new TypeError(
      'memory must be a WebAssembly.Memory created in the same realm as this loader',
    )
  }
  if (__claimedMemories.has(__provided)) {
    throw new TypeError(
      'This WebAssembly.Memory has already been used for a deferred initialization attempt and cannot be reused, including after a failed initialization or a disposal',
    )
  }
  // Last step, after every check: a rejected option bag must leave the Memory
  // unclaimed, or a caller could not fix the call and retry with it.
  __claimedMemories.add(__provided)
  return __provided
}

export const __napiBindingTarget = 'wasm32-wasip1'
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

/**
 * Deferred, workerd-safe instantiation: no top-level I/O, no compile-from-bytes.
 * Accepts ONLY a precompiled WebAssembly.Module, or a Promise resolving to one
 * (e.g. `import mod from './transform-react.wasm32-wasip1.wasm'` under a CompiledWasm
 * module rule / wrangler module import). Byte buffers, URLs and Response
 * objects are rejected: they require dynamic Wasm compilation, which
 * Cloudflare Workers disallows.
 */
async function __resolveModule(__wasmInput) {
  const __module = await __wasmInput
  // Brand check, not `instanceof`: `WebAssembly.Module.imports` throws unless
  // its argument is a genuine WebAssembly.Module, so prototype-spoofed byte
  // buffers are rejected while cross-realm Module instances are accepted.
  try {
    WebAssembly.Module.imports(__module)
  } catch {
    throw new TypeError(
      "instantiate() and createInstance() expect a precompiled WebAssembly.Module (or a Promise resolving to one), " +
        "e.g. import mod from './transform-react.wasm32-wasip1.wasm' under a CompiledWasm module rule / wrangler module import. " +
        "Byte buffers, URLs and Response objects require dynamic Wasm compilation, which Cloudflare Workers disallows.",
    )
  }
  return __module
}

let __normalizedModules

function __rememberNormalizedModule(__module, __normalizedModule) {
  if (!__normalizedModules) {
    __normalizedModules = new WeakMap()
  }
  __normalizedModules.set(__module, __normalizedModule)
  return __normalizedModule
}

async function __normalizeModuleForEmnapi(__module) {
  if (__module instanceof WebAssembly.Module) {
    return __module
  }
  if (__normalizedModules) {
    const __normalizedModule = __normalizedModules.get(__module)
    if (__normalizedModule) {
      return __normalizedModule
    }
  }
  // @emnapi/core currently performs realm-local `instanceof` checks after
  // accepting the module. Structured cloning preserves compiled code without
  // compiling bytes and produces a Module owned by the current realm.
  if (typeof structuredClone === 'function') {
    try {
      const __normalizedModule = structuredClone(__module)
      if (__normalizedModule instanceof WebAssembly.Module) {
        return __rememberNormalizedModule(__module, __normalizedModule)
      }
    } catch {}
  }
  // MessageChannel uses the same structured-clone semantics and covers older
  // browser/Node hosts that expose it but not the structuredClone function.
  if (typeof MessageChannel === 'function') {
    let __channel
    try {
      __channel = new MessageChannel()
      const __normalizedModule = await new Promise((resolve, reject) => {
        __channel.port1.onmessage = (event) => resolve(event.data)
        __channel.port1.onmessageerror = () =>
          reject(new TypeError('Failed to clone WebAssembly.Module'))
        try {
          __channel.port2.postMessage(__module)
        } catch (error) {
          reject(error)
        }
      })
      if (__normalizedModule instanceof WebAssembly.Module) {
        return __rememberNormalizedModule(__module, __normalizedModule)
      }
    } catch {
    } finally {
      try {
        __channel?.port1.close()
      } catch {}
      try {
        __channel?.port2.close()
      } catch {}
    }
  }
  // Last-resort compatibility for genuine, extensible foreign Modules.
  try {
    Object.setPrototypeOf(__module, WebAssembly.Module.prototype)
  } catch {}
  if (__module instanceof WebAssembly.Module) {
    return __module
  }
  throw new TypeError(
    'This host cannot normalize a cross-realm WebAssembly.Module; ' +
      'provide structuredClone or MessageChannel support.',
  )
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

function __captureEmnapiAutoDestroyListener(__process) {
  if (
    !__process ||
    typeof __process.prependListener !== 'function' ||
    typeof __process.removeListener !== 'function'
  ) {
    return
  }
  let __autoDestroyListener
  const __captureListener = (__event, __listener) => {
    if (__event === 'beforeExit' && __autoDestroyListener === undefined) {
      __autoDestroyListener = __listener
    }
  }
  try {
    // Run before existing newListener hooks so a hook that registers its own
    // beforeExit listener cannot be mistaken for emnapi's registration.
    __process.prependListener('newListener', __captureListener)
  } catch {
    return
  }
  return () => {
    try {
      __process.removeListener('newListener', __captureListener)
    } catch {}
    if (__autoDestroyListener !== undefined) {
      try {
        __process.removeListener('beforeExit', __autoDestroyListener)
      } catch {}
    }
  }
}

function __attachCleanupError(__error, __cleanupError) {
  try {
    if (
      __error &&
      (typeof __error === 'object' || typeof __error === 'function') &&
      __error.cause === undefined
    ) {
      __error.cause = __cleanupError
    }
  } catch {}
}

// Mirror the primitive @emnapi/core schedules its threadsafe-function dispatch
// on, so the drain turns below interleave with that dispatch instead of racing
// ahead of it on a faster queue.
const __scheduleMacrotask = (function () {
  if (typeof setImmediate === 'function') {
    return function (__callback) {
      setImmediate(__callback)
    }
  }
  const __MessageChannel = globalThis.MessageChannel
  if (typeof __MessageChannel === 'function') {
    return function (__callback) {
      const __channel = new __MessageChannel()
      __channel.port1.onmessage = function () {
        __channel.port1.onmessage = null
        try {
          __channel.port1.close()
        } catch {}
        try {
          __channel.port2.close()
        } catch {}
        __callback()
      }
      __channel.port2.postMessage(null)
    }
  }
  return function (__callback) {
    setTimeout(__callback, 0)
  }
})()

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
 */
function __drainWasmEnvCleanup(__instance) {
  const __pending = __instance?.exports.napi_wasm_env_cleanup_pending
  const __observable = typeof __pending === 'function'
  if (__observable) {
    let __queued
    try {
      __queued = __pending()
    } catch {
      return
    }
    if (!__queued) {
      return
    }
  }
  const __limit = __observable
    ? __WASM_ENV_CLEANUP_DRAIN_TURNS
    : __WASM_ENV_CLEANUP_BLIND_DRAIN_TURNS
  return (async () => {
    let __queued = 0
    for (let __turn = 0; __turn < __limit; __turn++) {
      await new Promise((resolve) => {
        __scheduleMacrotask(resolve)
      })
      if (!__observable) {
        continue
      }
      try {
        __queued = __pending()
      } catch {
        return
      }
      if (!__queued) {
        return
      }
    }
    if (!__observable) {
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
    // Reject instead, as a retryable cleanup failure: `__prepareForDisposal`
    // leaves its drained flag unset, dispose() (and the instantiation-failure
    // path) decline to destroy, and a later dispose() runs the drain again — by
    // which time the queue has usually been delivered. A counter that is
    // somehow stuck nonzero therefore costs each attempt at most another
    // bounded wait and a rejection, never a stranded promise; the managed
    // beforeExit destroyer still reclaims the context.
    const __drainError = new Error(
      'the wasm environment still reports ' +
        __queued +
        ' queued settlement(s) after ' +
        __limit +
        ' event-loop turns; the context was not destroyed - retry dispose() to wait for the queue again',
    )
    __drainError.code = 'ERR_NAPI_WASI_CLEANUP_PENDING'
    throw __drainError
  })()
}

// A real, *referenced* timer for the async-work wait below, which polls the
// addon rather than interleaving with the @emnapi/core dispatch: a zero-delay
// macrotask there would spin the loop instead of yielding it. Falls back to the
// macrotask scheduler on a host without timers.
function __scheduleTimer(__callback, __delay) {
  const __setTimer = globalThis.setTimeout
  if (typeof __setTimer !== 'function') {
    __scheduleMacrotask(__callback)
    return
  }
  try {
    __setTimer(__callback, __delay)
  } catch {
    __scheduleMacrotask(__callback)
  }
}

// A real, referenced timer rather than a zero-delay macrotask, for the same
// reason the async-work wait uses one: this polls the addon instead of
// interleaving with the @emnapi/core dispatch, so a zero-delay turn would spin
// the loop instead of yielding it.
const __WASM_RUNTIME_WORK_POLL_INTERVAL_MS = 1
// Arrivals it takes before the poll paces on the host's timers alone. One
// proves nothing: a timer armed before the host's timers stopped still fires.
const __WASM_RUNTIME_WORK_POLL_TRUSTED_ARRIVALS = 2
// How long a parked turn's own timer must already have been due before a
// backup that runs calls it dropped. Slack, not a deadline: a timer is due
// against the event loop's clock, which is read once per iteration, while
// these are `Date.now()` readings taken part-way through one, so the two
// drift apart by however long the loop has been inside the current iteration.
const __WASM_RUNTIME_WORK_POLL_STALL_MS = 50
// How long a backup itself waits. What is left of it after the slack and one
// interval — 149 ms — has to cover the *two* poll turns that can separate a
// parked turn from the last backup armed while the host's timers still
// worked, so the ceiling on a single turn is half of it. See the invariant on
// `__armWasmRuntimePollStallBackup`.
const __WASM_RUNTIME_WORK_POLL_BACKUP_MS = 200

/**
 * Pacing state for one runtime-work poll.
 *
 * Per poll, never per module: whether the host's timers arrive is not a
 * property of the module. A host can lose its timers between two disposals,
 * and in the deferred shape every instance shares this module — one healthy
 * instance must not disarm the fallback for the next one.
 */
function __createWasmRuntimePollPace() {
  return {
    // Timers armed by *this* poll that have actually arrived.
    arrivals: 0,
    // The turn waiting on a timer alone *right now* — undefined whenever no
    // turn is parked — and when that turn's own timer came due.
    settleTurn: undefined,
    turnTimerDueAt: 0,
  }
}

/**
 * The backup that ends a turn whose timer is never going to arrive.
 *
 * Once the poll paces on the timer alone it has nothing left to fall back on
 * if the host's timers stop mid-poll: the turn that armed the dead timer is
 * the turn that parks, and a parked poll schedules nothing that could notice.
 * So every turn arms one of these before it yields, and each one compares due
 * times instead of measuring how long the parked turn has been waiting.
 *
 * Invariant: a parked turn is ended by the newest backup that was armed while
 * the host's timers still worked, and a backup ends a turn only when that
 * turn's own timer was already due a whole window before the backup itself.
 * Neither half turns on how far apart the arms happen to fall — what bounds
 * the rescue is how far back that newest live backup is:
 *
 * - *Ends it.* Hosts run timers in due order, so a backup that runs while a
 *   turn due a whole window earlier is still parked proves that turn's timer
 *   was dropped rather than merely late. That same comparison is what leaves a
 *   healthy host alone: there the turn's timer has already run and cleared
 *   `settleTurn` before any backup due after it can look.
 * - *Two turns back, not one.* A turn that ended does not prove its own timer
 *   arrived: until `…_TRUSTED_ARRIVALS` is reached every turn arms both
 *   primitives and the macrotask wins, so such a turn can end with its own
 *   timer — and the backup armed one line before it — already dead. The
 *   arrival that then flips the poll onto the timer alone can itself be a
 *   timer armed before the host's timers died. So the turn that parks can sit
 *   two turns past the last live arm, and the newest live backup is due
 *   `…_BACKUP_MS` less *two* turn lengths after that turn's own timer.
 *   Arming on every turn is what holds it to two, rather than however far back
 *   a throttle last let one through.
 * - *Ceiling.* Coverage therefore holds while two consecutive poll turns fit
 *   inside `…_BACKUP_MS` less the slack and one interval: 149 ms, so 74 ms
 *   per turn (measured: a 74 ms turn is still rescued, a 75 ms one parks).
 *   Past that the turn stays parked and the disposal promise never settles.
 *   The bound is deliberate: reaching it takes a host that drops timers
 *   mid-poll *and* keeps every poll turn busy for more than 74 ms, and neither
 *   Node nor WebContainer — the hosts that run the threaded artifact — does
 *   the second.
 *
 * The poll then goes back to arming both primitives until two fresh arrivals
 * prove the timers again. A host that stops running the timers it has
 * *already* accepted leaves nothing to fire, and the disposal promise stays
 * pending rather than wedging the thread — the same outcome as a blocking
 * closure that never returns. Unreferenced wherever the host allows it: the
 * poll's own turn timers are what keep the loop alive, never these.
 */
function __armWasmRuntimePollStallBackup(__pace) {
  const __setTimer = globalThis.setTimeout
  if (typeof __setTimer !== 'function') {
    // Nothing to back up: `__scheduleTimer` is on the macrotask channel
    // already, and that one cannot park.
    return
  }
  // Read before arming, so this never claims to be due earlier than the timer
  // actually is: a backup ends a turn only when it is provably due after it.
  const __dueAt = Date.now() + __WASM_RUNTIME_WORK_POLL_BACKUP_MS
  let __handle
  try {
    __handle = __setTimer(() => {
      const __settleTurn = __pace.settleTurn
      if (
        !__settleTurn ||
        __pace.turnTimerDueAt > __dueAt - __WASM_RUNTIME_WORK_POLL_STALL_MS
      ) {
        // No turn is parked, or the parked one's timer came due too close to
        // this backup to call it dropped — it may still arrive, and the turn
        // that armed it armed a backup due a whole window after *that*.
        return
      }
      __pace.arrivals = 0
      __pace.settleTurn = undefined
      __settleTurn()
    }, __WASM_RUNTIME_WORK_POLL_BACKUP_MS)
  } catch {
    return
  }
  if (__handle && typeof __handle.unref === 'function') {
    try {
      __handle.unref()
    } catch {}
  }
}

/**
 * One turn of the runtime-work poll.
 *
 * `__scheduleTimer` falls back to the macrotask scheduler when `setTimeout` is
 * missing or throws, but not when it is present, returns a handle and never
 * fires — fake timers in a test suite that disposes from an `afterEach`, or a
 * host whose timers belong to an IO context that is already gone. That host
 * would park this poll forever, and the poll is unbounded, so nothing would
 * ever call `…_finish`.
 *
 * Arm both primitives until timers armed by this poll have arrived twice, and
 * let whichever lands first end the turn; the loser resolves nothing. A host
 * with working timers therefore pays the double arming for the first turn or
 * two — the macrotask wins the race, but the timers behind it still arrive and
 * are counted — and paces on the timer alone from then on, instead of spinning
 * the loop on a zero-delay queue. A host whose timers never arrive keeps both,
 * and the macrotask is what keeps the poll moving. A host whose timers stop
 * after proving themselves is caught by `__armWasmRuntimePollStallBackup`,
 * which ends the parked turn and puts this poll back on both.
 */
function __yieldWasmRuntimePollTurn(__pace) {
  // Armed before the turn yields, and by every turn: what rescues a parked
  // turn has to have been armed while the host's timers still worked, and the
  // turn that parks is the one whose own timer is already dead.
  __armWasmRuntimePollStallBackup(__pace)
  return new Promise((resolve) => {
    let __settled = false
    const __settle = () => {
      if (__settled) {
        return
      }
      __settled = true
      if (__pace.settleTurn === __settle) {
        // Nothing is parked any more: a backup running later must not read a
        // due time this turn has already answered.
        __pace.settleTurn = undefined
      }
      resolve()
    }
    __scheduleTimer(() => {
      __pace.arrivals++
      __settle()
    }, __WASM_RUNTIME_WORK_POLL_INTERVAL_MS)
    // Read next to the arming it describes; see
    // `__armWasmRuntimePollStallBackup` for what the two due times mean.
    const __turnTimerDueAt = Date.now() + __WASM_RUNTIME_WORK_POLL_INTERVAL_MS
    if (__pace.arrivals < __WASM_RUNTIME_WORK_POLL_TRUSTED_ARRIVALS) {
      __scheduleMacrotask(__settle)
      return
    }
    // Paced by the timer alone from here; the backup is what ends this turn if
    // the timer never arrives.
    __pace.settleTurn = __settle
    __pace.turnTimerDueAt = __turnTimerDueAt
  })
}

/**
 * Yield event-loop turns until the addon reports its runtime work finished.
 *
 * The window between `napi_prepare_wasm_env_cleanup_begin` and
 * `…_finish` — the turns are the entire point of splitting the barrier, because
 * on a threaded artifact the work `…_finish` joins can itself be waiting for a
 * JavaScript turn from this very thread.
 *
 * Unbounded, for the same reason the async-work wait above is: giving up means
 * calling `…_finish`, which joins on this thread, and the work it would join is
 * the work waiting for a turn from this thread — so a bound does not end the
 * wait, it only moves it somewhere the JavaScript thread can no longer be
 * reached. A blocking closure that never returns keeps the disposal promise
 * pending instead. The host contract is in `crates/async-runtime/README.md`: a
 * blocking closure must never wait on a JavaScript turn.
 */
async function __pollWasmRuntimeWork(__workPending) {
  const __pace = __createWasmRuntimePollPace()
  for (;;) {
    await __yieldWasmRuntimePollTurn(__pace)
    try {
      if (!__workPending()) {
        return
      }
    } catch {
      // A trap is the only way this fails, and a trapped instance has no
      // reachable work left. Stop polling and finish.
      return
    }
  }
}

// How often to re-read `napi_wasm_async_work_pending` while waiting. The wait
// ends when the addon reports zero, so this only decides how promptly disposal
// notices — not how long it waits.
const __WASI_ASYNC_WORK_POLL_INTERVAL_MS = 1

/**
 * Settles this instance's outstanding `napi_async_work` before the teardown
 * that would strand it.
 *
 * The same hole the eager loaders had, and the same fix: the environment
 * cleanup barrier covers promise settlements queued on the threadsafe-function
 * queue and says nothing about `napi_async_work`, so destroying the context
 * with a work still outstanding leaves its completion callback with nowhere to
 * run and its promise unsettled forever.
 *
 * This flavor is threadless, so `compute` runs on the JavaScript thread inside
 * the macrotask that dequeued it: while this is running, any outstanding work
 * is queued rather than executing, and `napi_wasm_cancel_pending_async_work`
 * can take all of it. The poll is still what decides when the drain is done —
 * cancellation delivers those completions from a later macrotask, not
 * synchronously.
 *
 * Per instance, from that instance's own exports: two instances of this loader
 * have separate registries and must not wait on each other.
 *
 * Both exports are optional, so an addon built against a napi crate that
 * predates them keeps the previous behavior.
 *
 * Returns nothing when there is nothing outstanding, which keeps disposal
 * synchronous in the common case. No deadline: giving up would destroy the
 * environment with a completion callback still owed.
 */
function __drainInstanceAsyncWork(__instance) {
  const __exports = __instance?.exports
  const __pending = __exports?.napi_wasm_async_work_pending
  const __cancelPending = __exports?.napi_wasm_cancel_pending_async_work
  if (typeof __pending !== 'function' || typeof __cancelPending !== 'function') {
    return
  }

  const __readPending = () => {
    try {
      return __pending()
    } catch (__error) {
      // A trap is the only way this call fails: it reads a counter and cannot
      // allocate or call back into JavaScript. A trapped instance can no longer
      // run anything, so its outstanding work is unreachable by definition and
      // best-effort is the honest answer. Anything else means the export is not
      // what this loader thinks it is — a defect worth surfacing.
      if (__error instanceof globalThis.WebAssembly.RuntimeError) {
        return 0
      }
      throw __error
    }
  }

  if (!__readPending()) {
    return
  }
  try {
    __cancelPending()
  } catch {
    // Cancellation only bounds the wait. Failing it means waiting for the queue
    // to run instead, which the poll below already does.
  }
  if (!__readPending()) {
    return
  }

  return (async () => {
    while (__readPending()) {
      await new Promise((__resolve) => {
        __scheduleTimer(__resolve, __WASI_ASYNC_WORK_POLL_INTERVAL_MS)
      })
    }
  })()
}

function __createLifecycleReentryError(__operation) {
  const __error = new Error(
    __operation +
      '() cannot run while an emnapi Context.destroy() call is still active; await the original cleanup promise instead.',
  )
  __error.code = 'ERR_NAPI_WASI_LIFECYCLE_REENTRY'
  return __error
}

const __managedEmnapiContextDestroyers = new Set()
let __managedCleanupProcess
let __managedBeforeExitListener
let __managedDestroyPromise
let __managedDestroyersInFlight
let __managedBeforeExitRegistrationRetryCount = 0
let __managedBeforeExitRegistrationRetryScheduled = false
let __moduleLifecycleDestroyDepth = 0

function __removeManagedEmnapiCleanupListeners() {
  const __process = __managedCleanupProcess
  const __beforeExitListener = __managedBeforeExitListener
  __managedCleanupProcess = undefined
  __managedBeforeExitListener = undefined
  __managedBeforeExitRegistrationRetryCount = 0
  if (__process && __beforeExitListener) {
    try {
      __process.removeListener('beforeExit', __beforeExitListener)
    } catch {}
  }
}

function __scheduleManagedBeforeExitListenerRegistration() {
  if (
    !__managedCleanupProcess ||
    __managedBeforeExitListener ||
    __managedEmnapiContextDestroyers.size === 0 ||
    __managedBeforeExitRegistrationRetryScheduled ||
    __managedBeforeExitRegistrationRetryCount >= 3
  ) {
    return
  }
  __managedBeforeExitRegistrationRetryScheduled = true
  __managedBeforeExitRegistrationRetryCount++
  queueMicrotask(() => {
    __managedBeforeExitRegistrationRetryScheduled = false
    if (
      !__managedCleanupProcess ||
      __managedBeforeExitListener ||
      __managedEmnapiContextDestroyers.size === 0
    ) {
      return
    }
    try {
      __registerManagedBeforeExitListener()
    } catch {}
  })
}

function __registerManagedBeforeExitListener() {
  if (!__managedCleanupProcess || __managedBeforeExitListener) {
    return
  }
  try {
    __managedCleanupProcess.once(
      'beforeExit',
      __destroyManagedEmnapiContextsBeforeExit,
    )
  } catch (error) {
    __scheduleManagedBeforeExitListenerRegistration()
    throw error
  }
  __managedBeforeExitListener = __destroyManagedEmnapiContextsBeforeExit
  __managedBeforeExitRegistrationRetryCount = 0
}

function __settleManagedEmnapiContextDestroy(__promise) {
  if (__managedDestroyPromise === __promise) {
    __managedDestroyPromise = undefined
    __managedDestroyersInFlight = undefined
  }
  if (__managedEmnapiContextDestroyers.size === 0) {
    __removeManagedEmnapiCleanupListeners()
    return
  }
  try {
    __registerManagedBeforeExitListener()
  } catch {}
}

function __destroyManagedEmnapiContexts(__excludedDestroyers) {
  if (__managedDestroyPromise) {
    return __managedDestroyPromise
  }
  const __destroyers = Array.from(__managedEmnapiContextDestroyers).filter(
    (__destroy) => !__excludedDestroyers?.has(__destroy),
  )
  if (__destroyers.length === 0) {
    return Promise.resolve()
  }
  let __resolveDestroy
  let __rejectDestroy
  const __promise = new Promise((resolve, reject) => {
    __resolveDestroy = resolve
    __rejectDestroy = reject
  })
  __managedDestroyPromise = __promise
  __managedDestroyersInFlight = new Set(__destroyers)
  void Promise.all(
    __destroyers.map((__destroy) => {
      try {
        return Promise.resolve(__destroy()).then(
          () => ({ failed: false }),
          (error) => ({ failed: true, error }),
        )
      } catch (error) {
        return { failed: true, error }
      }
    }),
  ).then((__results) => {
    let __primaryError
    let __failed = false
    for (const __result of __results) {
      if (!__result.failed) {
        continue
      }
      if (!__failed) {
        __failed = true
        __primaryError = __result.error
      } else {
        __attachCleanupError(__primaryError, __result.error)
      }
    }
    if (__failed) {
      __rejectDestroy(__primaryError)
    } else {
      __resolveDestroy()
    }
  }, __rejectDestroy)
  void __promise.then(
    () => {
      __settleManagedEmnapiContextDestroy(__promise)
    },
    () => {
      __settleManagedEmnapiContextDestroy(__promise)
    },
  )
  return __promise
}

async function __drainManagedEmnapiContexts(__excludedDestroyers) {
  const __attemptedDestroyers = new Set(__excludedDestroyers)
  let __primaryError
  let __failed = false
  while (true) {
    let __promise = __managedDestroyPromise
    let __destroyers = __managedDestroyersInFlight
    if (!__promise) {
      __promise = __destroyManagedEmnapiContexts(__attemptedDestroyers)
      __destroyers = __managedDestroyersInFlight
      if (!__destroyers) {
        break
      }
    }
    for (const __destroy of __destroyers) {
      __attemptedDestroyers.add(__destroy)
    }
    try {
      await __promise
    } catch (error) {
      if (!__failed) {
        __failed = true
        __primaryError = error
      } else {
        __attachCleanupError(__primaryError, error)
      }
    }
  }
  if (__failed) {
    throw __primaryError
  }
}

function __destroyManagedEmnapiContextsBeforeExit() {
  // A once listener is consumed before Node invokes it, including when another
  // cleanup batch is still pending.
  __managedBeforeExitListener = undefined
  if (__managedDestroyPromise) {
    return
  }
  void __destroyManagedEmnapiContexts().catch((error) => {
    queueMicrotask(() => {
      throw error
    })
  })
}

function __registerManagedEmnapiContext(__process, __destroy) {
  __managedEmnapiContextDestroyers.add(__destroy)
  if (
    !__managedCleanupProcess &&
    __process &&
    typeof __process.once === 'function' &&
    typeof __process.removeListener === 'function'
  ) {
    __managedCleanupProcess = __process
  }
  let __registered = true
  return () => {
    if (!__registered) {
      return
    }
    __registered = false
    __managedEmnapiContextDestroyers.delete(__destroy)
    if (__managedEmnapiContextDestroyers.size === 0) {
      __removeManagedEmnapiCleanupListeners()
    }
  }
}

async function __createManagedEmnapiContext(
  __prepareEnvCleanup,
  __isPreparingEnvCleanup,
) {
  const __process =
    typeof process === 'object' && process !== null ? process : undefined
  const __finishAutoDestroyCapture =
    __captureEmnapiAutoDestroyListener(__process)
  let __emnapiContext
  let __contextInitializationError
  let __contextInitializationFailed = false
  try {
    __emnapiContext = __wrapEmnapiContextDestroyForSettlement(
      __emnapiCreateContext({ autoDestroy: false }),
      __prepareEnvCleanup,
      __isPreparingEnvCleanup,
    )
    // emnapi 2.x still registers an unconditional process.once('beforeExit')
    // auto-destroy listener on Node hosts, and suppressDestroy() only
    // neutralizes its callback without removing it. This loader must stay
    // side-effect free per instance, so the listener is captured and removed;
    // suppressDestroy() remains the safety net when removal is unavailable.
    __emnapiContext.suppressDestroy()
  } catch (error) {
    __contextInitializationError = error
    __contextInitializationFailed = true
  } finally {
    // Remove only the exact emnapi callback captured above.
    __finishAutoDestroyCapture?.()
  }
  if (__emnapiContext === undefined) {
    throw __contextInitializationError
  }
  let __disposed = false
  let __destroying = false
  let __destroyPromise
  let __cleanupRegistered = false
  let __unregisterCleanup
  const __destroy = (__blocksModuleLifecycle = false) => {
    if (__disposed) {
      return
    }
    if (__destroying) {
      throw __createLifecycleReentryError('dispose')
    }
    if (__destroyPromise) {
      return __destroyPromise
    }
    __destroying = true
    let __result
    const __finishDestroyInvocation = () => {
      __destroying = false
    }
    const __finishModuleLifecycleDestroy = () => {
      if (__blocksModuleLifecycle) {
        __blocksModuleLifecycle = false
        __moduleLifecycleDestroyDepth--
      }
    }
    if (__blocksModuleLifecycle) {
      __moduleLifecycleDestroyDepth++
    }
    try {
      // Context.destroy() disables JS before cleanup hooks run, so settle
      // runtime-owned promises while this environment can still call JS.
      __prepareEnvCleanup?.()
      if (__isPreparingEnvCleanup?.()) {
        // Reached from inside the barrier, so `Context.destroy()` below would
        // hit the wrapper's in-flight no-op. Recording that as a completed
        // destroy is what makes the frame that *did* start the barrier skip the
        // real one afterwards, leaving the context retained with its cleanup
        // hooks unrun. Refuse instead: nothing is flagged, the context stays
        // registered for managed beforeExit cleanup, and a later destroy still
        // works. dispose() coalesces reentrancy before it can get here, so this
        // is the backstop for any other caller that manages to. A handshake
        // parked between the two halves of the barrier does not reach here —
        // `__prepareEnvCleanup` closes one rather than skipping it.
        throw __createLifecycleReentryError('dispose')
      }
      __result = __emnapiContext.destroy()
    } catch (error) {
      __finishDestroyInvocation()
      __finishModuleLifecycleDestroy()
      throw error
    }
    let __then
    try {
      if (
        __result !== null &&
        (typeof __result === 'object' || typeof __result === 'function')
      ) {
        __then = __result.then
      }
    } catch (error) {
      __finishDestroyInvocation()
      __finishModuleLifecycleDestroy()
      throw error
    }
    if (typeof __then === 'function') {
      let __resolveResult
      let __rejectResult
      const __resultPromise = new Promise((resolve, reject) => {
        __resolveResult = resolve
        __rejectResult = reject
      })
      const __promise = __resultPromise.then(
        (value) => {
          __finishDestroyInvocation()
          __finishModuleLifecycleDestroy()
          __disposed = true
          __destroyPromise = undefined
          __unregisterCleanup?.()
          return value
        },
        (error) => {
          __finishDestroyInvocation()
          __finishModuleLifecycleDestroy()
          __destroyPromise = undefined
          throw error
        },
      )
      __destroyPromise = __promise
      try {
        Reflect.apply(__then, __result, [__resolveResult, __rejectResult])
      } catch (error) {
        __rejectResult(error)
      }
      return __promise
    }
    __finishDestroyInvocation()
    __finishModuleLifecycleDestroy()
    __disposed = true
    __unregisterCleanup?.()
  }
  const __destroyForModuleLifecycle = () => __destroy(true)
  const __registerCleanup = (
    __beforeExitDestroy = __destroyForModuleLifecycle,
  ) => {
    if (__cleanupRegistered || __disposed) {
      return
    }
    __unregisterCleanup = __registerManagedEmnapiContext(
      __process,
      __beforeExitDestroy,
    )
    __cleanupRegistered = true
    __registerManagedBeforeExitListener()
  }
  if (__contextInitializationFailed) {
    let __registrationError
    let __registrationFailed = false
    try {
      __registerCleanup()
    } catch (error) {
      __attachCleanupError(__contextInitializationError, error)
      __registrationError = error
      __registrationFailed = true
    }
    try {
      await __destroyForModuleLifecycle()
    } catch (error) {
      __attachCleanupError(
        __registrationFailed
          ? __registrationError
          : __contextInitializationError,
        error,
      )
      try {
        __registerManagedBeforeExitListener()
      } catch {}
    }
    throw __contextInitializationError
  }
  return {
    context: __emnapiContext,
    destroy: __destroy,
    destroyForModuleLifecycle: __destroyForModuleLifecycle,
    registerCleanup: __registerCleanup,
  }
}

async function __createInstance(
  __wasmInput,
  __options,
  __beforeExitDestroy,
  __onManagedDestroyer,
) {
  const __module = await __resolveModule(__wasmInput)
  const __emnapiModule = await __normalizeModuleForEmnapi(__module)
  const __wasi = new __WASI({
    version: 'preview1',
  })
  // The wasm module is linked with `--import-memory`, so a Memory must be
  // provided. It is resolved here in function scope (workerd bans global scope
  // allocation) and is never shared (no threads, no SharedArrayBuffer).
  // Resolve it before the emnapi context so a rejected option bag or a host
  // memory-limit failure cannot leak a context that never reaches
  // instantiation.
  const __wasmMemory = __resolveInstanceMemory(__options)
  let __lifecycleState = 'pending'
  let __destroyEmnapiContext
  let __destroyOwnedContext
  let __destroyManagedOwnedContext
  let __napiInstance
  let __wasmEnvCleanupRan = false
  let __wasmEnvCleanupPrepared = false
  let __wasmEnvCleanupPreparing = false
  // The closer for a barrier parked between `…_begin` and `…_finish`, set only
  // while that window is open. `__wasmEnvCleanupPreparing` cannot tell that
  // apart from a purely synchronous frame, which must not be re-entered and
  // which nothing outside it can finish; this window spans real event-loop
  // turns, so a caller that cannot yield — the managed beforeExit teardown —
  // can land inside it, and can close it. See `__prepareEnvCleanup`.
  let __finishParkedEnvCleanup
  // Raised while a caller that can still yield is driving the barrier, so the
  // queue it leaves behind is expected rather than lost.
  let __wasmEnvCleanupYielding = false
  let __wasmEnvSettlementLossReported = false
  let __wasmEnvCleanupDrained = false
  let __wasmEnvCleanupDrainPromise
  const __isPreparingEnvCleanup = () => __wasmEnvCleanupPreparing
  /**
   * Say so when the barrier leaves settlements queued and nothing is left that
   * could deliver them.
   *
   * Only `dispose()` and the initialization rollback yield the event-loop turns
   * @emnapi/core needs to dispatch its queue. Every other caller of the barrier
   * destroys in the same turn — a raw `Context.destroy()`, the managed
   * beforeExit teardown — and `Context.destroy()` runs the threadsafe
   * function's cleanup hook, which drains that queue with a null env and
   * discards it. The promises those settlements were for then hang forever,
   * silently.
   *
   * Loud, once, and never throwing: this runs from inside `Context.destroy()`,
   * where throwing would take the whole teardown down with it. Destroying
   * anyway is still the right trade — the queue is already unreachable by then.
   */
  const __reportUnreachedSettlements = () => {
    if (__wasmEnvCleanupYielding || __wasmEnvSettlementLossReported) {
      return
    }
    const __pending = __napiInstance?.exports.napi_wasm_env_cleanup_pending
    if (typeof __pending !== 'function') {
      return
    }
    let __queued
    try {
      __queued = __pending()
    } catch {
      return
    }
    if (!__queued) {
      return
    }
    __wasmEnvSettlementLossReported = true
    try {
      const __consoleHost = globalThis.console
      if (__consoleHost && typeof __consoleHost.error === 'function') {
        __consoleHost.error(
          'napi-rs: the wasm environment is being destroyed with ' +
            __queued +
            ' queued promise settlement(s). Context.destroy() discards them, so those promises never settle. Dispose the instance instead: only dispose() yields the event-loop turns the settlements need.',
        )
      }
    } catch {}
  }
  const __prepareEnvCleanup = () => {
    if (__wasmEnvCleanupPrepared) {
      return
    }
    // A handshake parked between its two halves is one this frame can close,
    // and must: every caller of this is about to destroy the context, and the
    // turns the poll is waiting for will not come. Closing it runs `…_finish`,
    // which is the call that joins, so this degrades to exactly the single
    // call below. Leaving it open destroys the context with the barrier still
    // raised and the runtime never joined.
    const __finishParked = __finishParkedEnvCleanup
    if (__finishParked !== undefined) {
      __finishParked()
      __reportUnreachedSettlements()
      return
    }
    if (__wasmEnvCleanupPreparing) {
      return
    }
    const __prepareWasmEnvCleanup =
      __napiInstance?.exports.napi_prepare_wasm_env_cleanup
    if (typeof __prepareWasmEnvCleanup === 'function') {
      // The addon settles the promises it cancels synchronously, under a
      // non-reentrant lifecycle mutex: anything a promise hook calls from in
      // here must not reach this export again.
      __wasmEnvCleanupPreparing = true
      try {
        __prepareWasmEnvCleanup()
      } finally {
        __wasmEnvCleanupPreparing = false
      }
      __wasmEnvCleanupRan = true
      __reportUnreachedSettlements()
    }
    __wasmEnvCleanupPrepared = true
  }
  /**
   * The barrier for the callers that can yield: `__prepareEnvCleanup` with real
   * event-loop turns in the middle.
   *
   * `napi_prepare_wasm_env_cleanup` waits — it returns only once the addon's
   * async runtime has quiesced, and the work it waits for can itself be waiting
   * for a JavaScript turn from this thread. The addon's two-phase form splits
   * that: `…_begin` stops the runtime without joining and reports whether
   * anything is still live, `napi_wasm_runtime_work_pending` answers that again
   * without blocking, and `…_finish` joins.
   *
   * The poll is unbounded — see `__pollWasmRuntimeWork` — but a caller that
   * cannot yield closes the handshake itself rather than waiting for it, so
   * `…_finish` still runs on every teardown path. Feature-detected like every
   * other export here, and returns nothing whenever the handshake finished
   * without yielding.
   */
  const __prepareEnvCleanupWithTurns = () => {
    if (__wasmEnvCleanupPrepared || __wasmEnvCleanupPreparing) {
      return
    }
    const __exports = __napiInstance?.exports
    const __begin = __exports?.napi_prepare_wasm_env_cleanup_begin
    const __finish = __exports?.napi_prepare_wasm_env_cleanup_finish
    if (typeof __begin !== 'function' || typeof __finish !== 'function') {
      // No split to use. The settlement drain still follows this, so the queue
      // the single call leaves behind is expected rather than lost.
      __wasmEnvCleanupYielding = true
      try {
        __prepareEnvCleanup()
      } finally {
        __wasmEnvCleanupYielding = false
      }
      return
    }
    const __workPending = __exports?.napi_wasm_runtime_work_pending
    // The in-flight flag stays raised across the turns below, so a `destroy()`
    // from one of the JavaScript handlers they run is the same no-op it is
    // inside the single call: the barrier is up and the runtime is
    // mid-teardown, and destroying between the halves would strand exactly what
    // this delivers.
    __wasmEnvCleanupPreparing = true
    let __live
    try {
      __live = __begin()
    } catch (__error) {
      __wasmEnvCleanupPreparing = false
      throw __error
    }
    __wasmEnvCleanupRan = true
    const __finishEnvCleanup = () => {
      if (__wasmEnvCleanupPrepared) {
        // Already closed by a caller that could not yield. `…_finish` is
        // idempotent, but the flags it lowers are not.
        return
      }
      __finishParkedEnvCleanup = undefined
      try {
        __finish()
      } finally {
        __wasmEnvCleanupPreparing = false
      }
      __wasmEnvCleanupPrepared = true
    }
    if (!__live || typeof __workPending !== 'function') {
      __finishEnvCleanup()
      return
    }
    // Publish the closer before yielding: from here until `__finishEnvCleanup`
    // runs, a caller that cannot yield is entitled to end this handshake.
    __finishParkedEnvCleanup = __finishEnvCleanup
    return __pollWasmRuntimeWork(__workPending).then(
      __finishEnvCleanup,
      __finishEnvCleanup,
    )
  }
  // The barrier + settlement drain, hoisted out of the context destroyer so the
  // drain can yield without widening the destroyer's reentry window. Both
  // yielding paths run it — dispose() and the initialization-failure rollback.
  // The destroyer still runs the barrier itself (idempotently) for the one path
  // that cannot yield: managed beforeExit cleanup of an instance whose rollback
  // is being retried.
  //
  // "Drained" is recorded only once a wait has actually finished. Scheduling a
  // macrotask can fail — a host-provided or patched `setImmediate` that throws
  // is enough — and dispose() stays retryable after it rejects, so marking the
  // drain complete up front would make the retry skip it and destroy the context
  // with the barrier's settlements still queued.
  const __drainAfterEnvCleanup = () => {
    if (!__wasmEnvCleanupRan) {
      return
    }
    const __drained = __drainWasmEnvCleanup(__napiInstance)
    if (!__drained || typeof __drained.then !== 'function') {
      __wasmEnvCleanupDrained = true
      return
    }
    return __drained.then((__value) => {
      __wasmEnvCleanupDrained = true
      return __value
    })
  }
  const __prepareForDisposal = () => {
    if (__wasmEnvCleanupDrained) {
      return
    }
    if (__wasmEnvCleanupDrainPromise) {
      return __wasmEnvCleanupDrainPromise
    }
    // The barrier itself can yield now, so the memo below has to cover it too:
    // a reentrant caller must join this handshake rather than start a second
    // one while the first is parked between the two halves.
    const __prepared = __prepareEnvCleanupWithTurns()
    const __settled =
      __prepared && typeof __prepared.then === 'function'
        ? __prepared.then(__drainAfterEnvCleanup)
        : __drainAfterEnvCleanup()
    if (!__settled || typeof __settled.then !== 'function') {
      return
    }
    const __tracked = __settled.then(
      (__value) => {
        __wasmEnvCleanupDrainPromise = undefined
        return __value
      },
      (__error) => {
        __wasmEnvCleanupDrainPromise = undefined
        throw __error
      },
    )
    __wasmEnvCleanupDrainPromise = __tracked
    return __tracked
  }
  let __disposed = false
  const __runInstanceDisposal = async () => {
    if (__lifecycleState !== 'failed') {
      __lifecycleState = 'disposal'
    }
    // Outstanding async work first, while the environment is still completely
    // live: its completion callbacks run addon code, and the barrier and
    // `Context.destroy()` below each take that away. Undefined unless work is
    // outstanding.
    const __asyncWorkDrained = __drainInstanceAsyncWork(__napiInstance)
    if (__asyncWorkDrained) {
      await __asyncWorkDrained
    }
    // Then settle what the barrier cancelled, before the environment stops
    // accepting JavaScript calls. Undefined unless something is queued, so
    // an idle disposal is not delayed by a single turn.
    const __drained = __prepareForDisposal()
    if (__drained) {
      await __drained
    }
    const __result = await (__beforeExitDestroy
      ? __destroyManagedOwnedContext()
      : __destroyOwnedContext())
    // Only a completed destroy retires the instance; a throw above leaves
    // the counter untouched so a retried dispose() cannot double-decrement.
    if (!__disposed) {
      __disposed = true
      __liveInstances -= 1
    }
    return __result
  }
  let __instanceDisposePromise
  /**
   * The disposal frame runs the barrier, and the barrier settles the promises it
   * cancels synchronously — so a promise hook firing inside it can call this
   * same instance's dispose() again while the first call is still in its drain.
   * That nested call finds the barrier flagged in flight, prepares nothing,
   * drains nothing, and falls straight through to the context destroyer, whose
   * `Context.destroy()` hits the wrapper's in-flight no-op. It would record a
   * destruction that never happened, and the outer frame would then skip the
   * real one: both disposals resolve, no cleanup hook runs, the context stays
   * retained.
   *
   * Memoize before any of that starts, exactly like the eager loaders'
   * `__disposeWasiBinding`, so there is only ever one disposal frame per
   * instance and a reentrant caller awaits it instead of racing it. Cleared on
   * rejection: a drain that failed has to stay retryable.
   */
  const __disposeInstance = () => {
    if (__instanceDisposePromise) {
      return __instanceDisposePromise
    }
    let __resolveDispose
    let __rejectDispose
    const __disposePromise = new Promise((__resolve, __reject) => {
      __resolveDispose = __resolve
      __rejectDispose = __reject
    })
    __instanceDisposePromise = __disposePromise
    __runInstanceDisposal().then(__resolveDispose, (__error) => {
      __instanceDisposePromise = undefined
      __rejectDispose(__error)
    })
    return __disposePromise
  }
  const __destroyBeforeExit = __beforeExitDestroy
    ? async () => {
        if (__lifecycleState === 'failed') {
          await __destroyManagedOwnedContext()
          return
        }
        __lifecycleState = 'disposal'
        try {
          await __beforeExitDestroy()
        } catch (error) {
          if (__lifecycleState !== 'failed') {
            throw error
          }
          // The singleton's initialization rejection is already observable
          // through instantiate() and dispose(). Managed beforeExit cleanup
          // owns only context destruction, including retrying a failed rollback.
          await __destroyManagedOwnedContext()
        }
      }
    : undefined
  const {
    context: __emnapiContext,
    destroy,
    destroyForModuleLifecycle,
    registerCleanup: __registerCleanup,
  } = await __createManagedEmnapiContext(
    __prepareEnvCleanup,
    __isPreparingEnvCleanup,
  )
  __destroyEmnapiContext = destroy
  __destroyOwnedContext = () => __destroyEmnapiContext()
  __destroyManagedOwnedContext = destroyForModuleLifecycle
  try {
    if (__destroyBeforeExit) {
      __onManagedDestroyer(__destroyBeforeExit)
      await __registerCleanup(__destroyBeforeExit)
    }
    let __napiModule
    ;({
      instance: __napiInstance,
      napiModule: __napiModule,
    } = await __emnapiInstantiateNapiModule(__emnapiModule, {
      context: __emnapiContext,
      asyncWorkPoolSize: 0,
      plugins: [__emnapiAsyncWorkPlugin, __emnapiTSFNPlugin],
      wasi: __wasi,
      overwriteImports(importObject) {
        importObject.env = {
          ...importObject.env,
          ...importObject.napi,
          ...importObject.emnapi,
          memory: __wasmMemory,
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
    // `instantiate()` and `createInstance().exports` hand out this object; a
    // named module export does not travel with it. After the instance host
    // install, which hands the same object to addon-provided registration
    // functions that may put anything on it, and inside this `try`, so a
    // claimed name flips `__lifecycleState` to 'failed' and tears the instance
    // down rather than escaping a half-built one.
    __napiStampBindingTarget(__napiModule.exports, __napiBindingTarget)
    if (__lifecycleState === 'pending') {
      __lifecycleState = 'succeeded'
    }
    __createdInstances += 1
    __liveInstances += 1
    return {
      exports: __napiModule.exports,
      get memory() {
        return __wasmMemory
      },
      get memoryBytes() {
        // The Memory outlives the environment, so this stays readable after a
        // FAILED dispose() (which leaves the instance undisposed and
        // retryable). It reports 0 only once disposal has actually completed.
        return __disposed ? 0 : __wasmMemory.buffer.byteLength
      },
      get disposed() {
        return __disposed
      },
      dispose: __disposeInstance,
    }
  } catch (error) {
    __lifecycleState = 'failed'
    // Instantiation can fail *after* registration has run, and registration runs
    // with a live environment: a module-init hook can start async work and then
    // return an error, and the promise it created may already have escaped into
    // JavaScript. Settle what the barrier cancels before the context is
    // destroyed, exactly like dispose() does; destroying without yielding
    // discards the queue with a null env. Undefined unless something is queued,
    // so a failure before beforeInit costs no extra turn.
    let __settlementsUnreached = false
    try {
      // Registration can start async work too, and this path destroys the same
      // environment its completions need. A drain that cannot finish leaves the
      // work outstanding, so it counts as settlements unreached and stops the
      // rollback short of destroying, exactly like a failed barrier drain.
      const __asyncWorkDrained = __drainInstanceAsyncWork(__napiInstance)
      if (__asyncWorkDrained) {
        await __asyncWorkDrained
      }
      const __drained = __prepareForDisposal()
      if (__drained) {
        await __drained
      }
    } catch (drainError) {
      __attachCleanupError(error, drainError)
      __settlementsUnreached = true
    }
    let __registrationError
    let __registrationFailed = false
    if (!__beforeExitDestroy) {
      try {
        // Independent instances are caller-owned while pending and after
        // success. Register only failed rollback so cleanup remains retryable.
        await __registerCleanup()
      } catch (registrationError) {
        __attachCleanupError(error, registrationError)
        __registrationError = registrationError
        __registrationFailed = true
      }
    }
    if (__settlementsUnreached) {
      // The barrier or the drain did not finish, so the settlements it queued
      // are still in the threadsafe-function queue. Destroying now runs the
      // cleanup hook that drains that queue with a null env and discards it,
      // stranding a promise that already escaped into JavaScript — with nothing
      // left that could ever settle it. dispose() refuses to destroy for exactly
      // this reason (a rejected drain there never reaches the destroy), so this
      // path refuses too.
      //
      // Nothing leaks. The registration just above — and, for the singleton, the
      // one made before instantiation — leaves this context in
      // `__managedEmnapiContextDestroyers`, so beforeExit destroys it and
      // dispose() can retry it. Only the destruction is deferred, and the turns
      // that pass in the meantime are exactly what the queue needed.
      try {
        __registerManagedBeforeExitListener()
      } catch {}
      throw error
    }
    try {
      await __destroyManagedOwnedContext()
    } catch (disposeError) {
      // Initialization is the primary failure. Preserve it even if cleanup
      // also fails, while retaining the cleanup error when the value is
      // extensible and has no existing cause.
      __attachCleanupError(
        __registrationFailed ? __registrationError : error,
        disposeError,
      )
      try {
        __registerManagedBeforeExitListener()
      } catch {}
    }
    throw error
  }
}

/**
 * Create an independent instance. Call and await dispose() when the instance
 * is no longer needed so emnapi cleanup hooks run deterministically.
 *
 * The optional second argument selects this instance's linear memory: either
 * `memory` (an unshared, single-use WebAssembly.Memory you allocated) or
 * `initialMemoryPages` / `maximumMemoryPages`, never both. Omitted, the
 * loader allocates WASM_MEMORY.initialPages..WASM_MEMORY.maximumPages.
 *
 * A provided Memory must come from this loader's own realm: the WASI and
 * emnapi layers underneath identify one with a realm-local `instanceof`, so a
 * Memory built in a `node:vm` context or another frame is rejected. Every
 * Memory an instance runs on is single-use, the loader-allocated one included:
 * `instance.memory` cannot be recycled into a second `createInstance()`.
 */
export async function createInstance(__wasmInput, __options) {
  return __createInstance(__wasmInput, __options)
}

let __defaultModulePromise
let __defaultInstancePromise
let __defaultDisposePromise
let __defaultDisposalStarted = false
const __defaultManagedDestroyers = new WeakMap()
let __moduleDisposePromise

/**
 * Instantiate a module-local singleton. Concurrent and repeated calls
 * with the same module share one instance and one Memory allocation.
 */
export function instantiate(__wasmInput) {
  const __modulePromise = __resolveModule(__wasmInput)
  if (__moduleLifecycleDestroyDepth !== 0) {
    void __modulePromise.catch(() => {})
    return Promise.reject(__createLifecycleReentryError('instantiate'))
  }
  if (__moduleDisposePromise) {
    void __modulePromise.catch(() => {})
    return __moduleDisposePromise.then(() => instantiate(__modulePromise))
  }
  if (__defaultDisposalStarted) {
    // Observe rejected input immediately, but preserve lifecycle ordering and
    // error precedence by instantiating only after disposal succeeds. A failed
    // disposal retains the old instance only so its cleanup can be retried.
    void __modulePromise.catch(() => {})
    const __disposePromise = __defaultDisposePromise ?? dispose()
    return __disposePromise.then(() => instantiate(__modulePromise))
  }
  if (!__defaultInstancePromise) {
    __defaultModulePromise = __modulePromise
    const __instancePromise = __modulePromise.then((__module) =>
      __createInstance(
        __module,
        undefined,
        __disposeDefaultInstance,
        (__managedDestroyer) => {
          __defaultManagedDestroyers.set(
            __instancePromise,
            __managedDestroyer,
          )
        },
      ),
    )
    __defaultInstancePromise = __instancePromise
    void __instancePromise.catch(() => {
      if (__defaultInstancePromise === __instancePromise) {
        __defaultInstancePromise = undefined
        __defaultModulePromise = undefined
      }
    })
    return __instancePromise.then((__instance) => __instance.exports)
  }
  const __defaultModulePromiseForCall = __defaultModulePromise
  const __defaultInstancePromiseForCall = __defaultInstancePromise
  return Promise.all([__defaultModulePromiseForCall, __modulePromise]).then(
    async ([__defaultModule, __module]) => {
      if (__defaultModule !== __module) {
        throw new Error(
          'instantiate() already owns a different WebAssembly.Module; call dispose() first or use createInstance() for independent instances.',
        )
      }
      return (await __defaultInstancePromiseForCall).exports
    },
  )
}

async function __disposeDefaultInstance(__onDestroy) {
  if (__defaultDisposePromise) {
    return __defaultDisposePromise
  }
  const __instancePromise = __defaultInstancePromise
  if (!__instancePromise) {
    __defaultDisposalStarted = false
    return
  }
  __defaultDisposalStarted = true
  const __disposePromise = (async () => {
    let __instance
    try {
      __instance = await __instancePromise
    } catch (error) {
      const __managedDestroyer =
        __defaultManagedDestroyers.get(__instancePromise)
      if (__managedDestroyer) {
        __onDestroy?.(__managedDestroyer)
      }
      __defaultManagedDestroyers.delete(__instancePromise)
      throw error
    }
    const __managedDestroyer =
      __defaultManagedDestroyers.get(__instancePromise)
    if (__managedDestroyer) {
      __onDestroy?.(__managedDestroyer)
    }
    await __instance.dispose()
    if (__defaultInstancePromise === __instancePromise) {
      __defaultInstancePromise = undefined
      __defaultModulePromise = undefined
      __defaultDisposalStarted = false
    }
    __defaultManagedDestroyers.delete(__instancePromise)
  })()
  __defaultDisposePromise = __disposePromise
  try {
    await __disposePromise
  } finally {
    if (__defaultDisposePromise === __disposePromise) {
      __defaultDisposePromise = undefined
    }
  }
}

async function __dispose() {
  let __defaultDisposeError
  let __defaultDisposeFailed = false
  let __attemptedDefaultDestroyer
  try {
    await __disposeDefaultInstance((__destroyer) => {
      __attemptedDefaultDestroyer = __destroyer
    })
  } catch (error) {
    __defaultDisposeError = error
    __defaultDisposeFailed = true
  }
  const __excludedDestroyers = new Set()
  if (__defaultDisposeFailed && __attemptedDefaultDestroyer) {
    __excludedDestroyers.add(__attemptedDefaultDestroyer)
  }
  try {
    await __drainManagedEmnapiContexts(__excludedDestroyers)
  } catch (error) {
    if (!__defaultDisposeFailed) {
      throw error
    }
    if (error !== __defaultDisposeError) {
      __attachCleanupError(__defaultDisposeError, error)
    }
  }
  if (__defaultDisposeFailed) {
    throw __defaultDisposeError
  }
}

/**
 * Dispose the singleton created by instantiate(). A later call may create a
 * fresh instance, including from a different module. This also retries cleanup
 * retained after a failed initialization rollback.
 */
export function dispose() {
  if (__moduleLifecycleDestroyDepth !== 0) {
    return Promise.reject(__createLifecycleReentryError('dispose'))
  }
  if (__moduleDisposePromise) {
    return __moduleDisposePromise
  }
  let __resolveDispose
  let __rejectDispose
  const __promise = new Promise((resolve, reject) => {
    __resolveDispose = resolve
    __rejectDispose = reject
  })
  __moduleDisposePromise = __promise
  void __dispose().then(__resolveDispose, __rejectDispose)
  void __promise.then(
    () => {
      if (__moduleDisposePromise === __promise) {
        __moduleDisposePromise = undefined
      }
    },
    () => {
      if (__moduleDisposePromise === __promise) {
        __moduleDisposePromise = undefined
      }
    },
  )
  return __promise
}
