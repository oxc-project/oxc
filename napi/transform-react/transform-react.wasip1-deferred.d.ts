export type WasiBinding = typeof import('./transform-react.wasip1.cjs')

export type WasiModuleInput =
  | WebAssembly.Module
  | PromiseLike<WebAssembly.Module>

/** Run the instance on a linear memory the caller allocated. */
export interface WasiCallerMemoryOptions {
  /**
   * A caller-allocated linear memory for this instance. It must be unshared
   * and created in this loader's own realm — the WASI and emnapi layers
   * underneath identify a Memory with a realm-local `instanceof`, so one from
   * a `node:vm` context or another frame is rejected. It is single-use: once
   * a validated initialization attempt has begun, the same Memory cannot be
   * passed again — including after that attempt failed, and after the instance
   * was disposed.
   */
  memory: WebAssembly.Memory
  /** Not available beside `memory`: the loader allocates neither. */
  initialMemoryPages?: never
  /** Not available beside `memory`: the loader allocates neither. */
  maximumMemoryPages?: never
}

/** Let the loader allocate the linear memory, optionally sized. */
export interface WasiAllocatedMemoryOptions {
  /** Not available beside the page counts: they size the loader's own Memory. */
  memory?: never
  /** @default WASM_MEMORY.initialPages */
  initialMemoryPages?: number
  /** @default WASM_MEMORY.maximumPages */
  maximumMemoryPages?: number
}

/**
 * Either memory form, never a mix of the two: the loader throws a TypeError
 * on `memory` beside a page count. `{}` and an omitted argument select the
 * loader defaults.
 */
export type WasiInstanceOptions =
  | WasiCallerMemoryOptions
  | WasiAllocatedMemoryOptions

export interface WasiRuntimeStats {
  /** Instances created by this evaluated loader module, not process-wide. */
  createdInstances: number
  /** Created instances whose dispose() has not completed. */
  liveInstances: number
  /** Declared initial address space, not committed memory. */
  declaredInitialMemoryBytes: number
}

export interface WasiInstance {
  readonly exports: WasiBinding
  /** This instance's linear memory. Claimed, so it cannot start another one. */
  readonly memory: WebAssembly.Memory
  /** Current linear-memory size; 0 once dispose() has completed. */
  readonly memoryBytes: number
  readonly disposed: boolean
  dispose(): Promise<void>
}

/** The memory descriptor compiled into this loader. */
export const WASM_MEMORY: Readonly<{
  initialPages: number
  maximumPages: number
  pageBytes: number
  initialBytes: number
  maximumBytes: number
}>

export function getDeferredRuntimeStats(): Readonly<WasiRuntimeStats>

export function instantiate(wasmInput: WasiModuleInput): Promise<WasiBinding>
export function createInstance(
  wasmInput: WasiModuleInput,
  options?: WasiInstanceOptions,
): Promise<WasiInstance>
/** Dispose the singleton and retry retained failed-initialization cleanup. */
export function dispose(): Promise<void>

/** The WASI flavor this deferred loader instantiates. */
export declare const __napiBindingTarget: 'wasm32-wasip1'
