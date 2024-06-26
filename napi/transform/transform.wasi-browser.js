import {
  instantiateNapiModuleSync as __emnapiInstantiateNapiModuleSync,
  getDefaultContext as __emnapiGetDefaultContext,
  WASI as __WASI,
  createOnMessage as __wasmCreateOnMessageForFsProxy,
} from '@napi-rs/wasm-runtime'

import __wasmUrl from './transform.wasm32-wasi.wasm?url'

const __wasi = new __WASI({
  version: 'preview1',
})

const __emnapiContext = __emnapiGetDefaultContext()

const __sharedMemory = new WebAssembly.Memory({
  initial: 4000,
  maximum: 65536,
  shared: true,
})

const __wasmFile = await fetch(__wasmUrl).then((res) => res.arrayBuffer())

const {
  instance: __napiInstance,
  module: __wasiModule,
  napiModule: __napiModule,
} = __emnapiInstantiateNapiModuleSync(__wasmFile, {
  context: __emnapiContext,
  asyncWorkPoolSize: 4,
  wasi: __wasi,
  onCreateWorker() {
    const worker = new Worker(new URL('./wasi-worker-browser.mjs', import.meta.url), {
      type: 'module',
    })
    
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
    __napi_rs_initialize_modules(instance)
  },
})

function __napi_rs_initialize_modules(__napiInstance) {
  __napiInstance.exports['__napi_register__TypeScriptBindingOptions_struct_0']?.()
  __napiInstance.exports['__napi_register__ReactBindingOptions_struct_1']?.()
  __napiInstance.exports['__napi_register__ArrowFunctionsBindingOptions_struct_2']?.()
  __napiInstance.exports['__napi_register__ES2015BindingOptions_struct_3']?.()
  __napiInstance.exports['__napi_register__TransformBindingOptions_struct_4']?.()
  __napiInstance.exports['__napi_register__Sourcemap_struct_5']?.()
  __napiInstance.exports['__napi_register__TransformResult_struct_6']?.()
  __napiInstance.exports['__napi_register__transform_7']?.()
  __napiInstance.exports['__napi_register__IsolatedDeclarationsResult_struct_8']?.()
  __napiInstance.exports['__napi_register__isolated_declaration_9']?.()
}
export const isolatedDeclaration = __napiModule.exports.isolatedDeclaration
export const transform = __napiModule.exports.transform
