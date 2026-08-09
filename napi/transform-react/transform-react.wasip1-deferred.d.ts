export type WasiBinding = typeof import('./transform-react.wasip1.cjs')

export type WasiModuleInput =
  | WebAssembly.Module
  | PromiseLike<WebAssembly.Module>

export interface WasiInstance {
  readonly exports: WasiBinding
  dispose(): Promise<void>
}

export function instantiate(wasmInput: WasiModuleInput): Promise<WasiBinding>
export function createInstance(wasmInput: WasiModuleInput): Promise<WasiInstance>
/** Dispose the singleton and retry retained failed-initialization cleanup. */
export function dispose(): Promise<void>
