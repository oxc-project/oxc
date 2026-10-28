import {
  instantiateNapiModuleSync,
  MessageHandler,
  WASI,
  emnapiAsyncWorkPlugin,
  emnapiTSFNPlugin,
} from '@napi-rs/wasm-runtime'

const handler = new MessageHandler({
  onLoad({ wasmModule, wasmMemory }) {
    const wasi = new WASI({
      print: function () {
        // eslint-disable-next-line no-console
        console.log.apply(console, arguments)
      },
      printErr: function () {
        // eslint-disable-next-line no-console
        console.error.apply(console, arguments)
      },
    })
    return instantiateNapiModuleSync(wasmModule, {
      childThread: true,
      wasi,
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

// When this wasm thread dies, raise the addon's crash flag before emnapi reports
// it. The loader thread may be inside wasm, in a cleanup call that waits on this
// thread; the flag is what those waits check between short slices, and once it
// is up they trap instead of waiting for good. It is a word in the shared wasm
// memory, and the loader posts a view of it (see
// napi_wasm_thread_crash_flag_address) after it instantiated the wasm, or right
// after it created this worker. No view, no flag: an addon built with an older
// napi, or a failure before the view arrived.
let __addonCrashFlag
const __beforeReportError = handler.beforeReportError
handler.beforeReportError = function (...args) {
  if (__addonCrashFlag !== undefined) {
    try {
      Atomics.store(__addonCrashFlag, 0, 1)
    } catch {}
  }
  return __beforeReportError.apply(this, args)
}

globalThis.onmessage = function (e) {
  const data = e && e.data
  if (data && data.__napiRsAddonCrashFlag instanceof Int32Array) {
    __addonCrashFlag = data.__napiRsAddonCrashFlag
    return
  }
  handler.handle(e)
}
