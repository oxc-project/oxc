import fs from 'node:fs'
import { createRequire } from 'node:module'
import { parse } from 'node:path'
import { WASI } from 'node:wasi'
import { parentPort, Worker, workerData } from 'node:worker_threads'

const require = createRequire(import.meta.url)

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

const handler = new MessageHandler({
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

globalThis.onmessage = function (e) {
  handler.handle(e)
}
