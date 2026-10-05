import fs from 'node:fs'
import { createRequire } from 'node:module'
import { parse } from 'node:path'
import { WASI } from 'node:wasi'
import { parentPort, threadId, Worker, workerData } from 'node:worker_threads'

const require = createRequire(import.meta.url)

// The thread spawn that created this worker has already returned, so the loader
// thread may be waiting on this thread. A failure before the crash hook below is
// in place (a missing '@napi-rs/wasm-runtime', say) has to raise the crash flags
// too, then fail the worker as before.
let handler
try {
  const {
    instantiateNapiModuleSync,
    MessageHandler,
    getDefaultContext,
    emnapiAsyncWorkPlugin,
    emnapiTSFNPlugin,
  } = require('@napi-rs/wasm-runtime')

  if (parentPort) {
    parentPort.on('message', (data) => {
      globalThis.onmessage({ data })
    })
  }

  Object.assign(globalThis, {
    self: globalThis,
    require,
    Worker,
    importScripts: function (f) {
      // oxlint-disable-next-line no-eval -- WASI importScripts polyfill
      ;(0, eval)(fs.readFileSync(f, 'utf8') + '//# sourceURL=' + f)
    },
    postMessage: function (msg) {
      if (parentPort) {
        parentPort.postMessage(msg)
      }
    },
  })

  const emnapiContext = getDefaultContext()

  const __cwd = process.cwd()
  const __rootDir =
    (workerData && typeof workerData.rootDir === 'string' && workerData.rootDir) ||
    parse(__cwd).root
  const __hostRoot =
    (workerData && typeof workerData.hostRoot === 'string' && workerData.hostRoot) ||
    (process.platform === 'android' ? __cwd : __rootDir)

  handler = new MessageHandler({
    onLoad({ wasmModule, wasmMemory }) {
      const wasi = new WASI({
        version: 'preview1',
        env: process.env,
        preopens: {
          [__rootDir]: __hostRoot,
          [__hostRoot]: __hostRoot,
        },
      })

      return instantiateNapiModuleSync(wasmModule, {
        childThread: true,
        wasi,
        context: emnapiContext,
        // The wasm links a "basic" emnapi archive (no C async-work /
        // threadsafe-function implementations), so every thread that
        // instantiates it must provide the JavaScript implementations
        // through the emnapi plugins.
        plugins: [emnapiAsyncWorkPlugin, emnapiTSFNPlugin],
        overwriteImports(importObject) {
          importObject.env = {
            ...importObject.env,
            ...importObject.napi,
            ...importObject.emnapi,
            memory: wasmMemory,
          }
        },
      })
    },
  })
} catch (error) {
  if (workerData && workerData.crashFlag instanceof Int32Array) {
    __raiseWasiThreadCrashFlags(error)
  }
  throw error
}

// Tell the loader thread synchronously that this wasm thread died, so its 'exit'
// teardown does not re-enter wasm and wait on this thread forever. emnapi calls
// beforeReportError (which runs emnapi_thread_crashed, so the main thread may
// throw 'unwind' and start exiting) before the 'error' event is even posted.
// A loader that predates the flag passes none.
if (workerData && workerData.crashFlag instanceof Int32Array) {
  const __beforeReportError = handler.beforeReportError
  handler.beforeReportError = function (...args) {
    if (!__raiseWasiThreadCrashFlags(args[0])) {
      // No view (the thread was spawned before the loader read the address,
      // say): the export stores into the same word, but only a worker that
      // loaded has an instance to call it on.
      try {
        const __napiThreadCrashed = this.instance?.exports?.napi_wasm_thread_crashed
        if (typeof __napiThreadCrashed === 'function') {
          __napiThreadCrashed()
        }
      } catch {}
    }
    return __beforeReportError.apply(this, args)
  }
}

// The error goes into the shared crash report first. The 'error' event that
// would carry it is dropped when the loader terminates this worker before the
// event is sent, so the report is the only copy the loader can count on.
// Layout: three Int32 words — state (0 empty, 1 writing, 2 written), byte
// length, threadId — then the UTF-8 JSON of { name, message, stack }. Only the
// first worker to crash writes it, and it is complete before the flag is raised.
//
// Then the loader's flag, then the addon's. The loader thread may already be
// inside wasm, in a cleanup call that waits on this thread; the addon's flag is
// what those waits check between short slices. It is a word in the shared wasm
// memory (the loader passes a view of it, see napi_wasm_thread_crash_flag_address),
// so no instance is needed: a worker that failed while loading raises it too.
// Once it is up the waits trap, and the loader, whose flag is already up, turns
// the throw into its crash rejection. Returns whether the addon's flag was raised.
function __raiseWasiThreadCrashFlags(error) {
  try {
    __writeCrashReport(workerData.crashReport, error)
  } catch {}
  try {
    Atomics.store(workerData.crashFlag, 0, 1)
  } catch {}
  const addonCrashFlag = workerData.addonCrashFlag
  if (!(addonCrashFlag instanceof Int32Array)) {
    return false
  }
  try {
    Atomics.store(addonCrashFlag, 0, 1)
    return true
  } catch {
    return false
  }
}

function __writeCrashReport(report, error) {
  if (!(report instanceof SharedArrayBuffer) || report.byteLength <= 12) {
    return
  }
  const header = new Int32Array(report, 0, 3)
  if (Atomics.compareExchange(header, 0, 0, 1) !== 0) {
    return
  }
  let length = 0
  try {
    const body = new Uint8Array(report, 12)
    const isObject =
      error !== null && (typeof error === 'object' || typeof error === 'function')
    const name = isObject && typeof error.name === 'string' ? error.name : 'Error'
    const message = isObject && typeof error.message === 'string'
      ? error.message
      : String(error)
    const stack = isObject && typeof error.stack === 'string' ? error.stack : undefined
    const encoder = new TextEncoder()
    let bytes = encoder.encode(JSON.stringify({ name, message, stack }))
    if (bytes.length > body.length) {
      bytes = encoder.encode(
        JSON.stringify({ name, message: message.slice(0, body.length >> 3) }),
      )
    }
    if (bytes.length <= body.length) {
      body.set(bytes)
      length = bytes.length
    }
    Atomics.store(header, 2, threadId)
  } finally {
    Atomics.store(header, 1, length)
    Atomics.store(header, 0, 2)
  }
}

globalThis.onmessage = function (e) {
  handler.handle(e)
}
