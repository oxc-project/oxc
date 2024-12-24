/* tslint:disable */
/* eslint-disable */
export function browserslist(query: string, opts: any): any;
export interface OxcOptions {
    run?: OxcRunOptions;
    parser?: OxcParserOptions;
    linter?: OxcLinterOptions;
    transformer?: OxcTransformerOptions;
    codegen?: OxcCodegenOptions;
    minifier?: OxcMinifierOptions;
    controlFlow?: OxcControlFlowOptions;
}

export interface OxcRunOptions {
    syntax?: boolean;
    lint?: boolean;
    format?: boolean;
    prettierFormat?: boolean;
    prettierIr?: boolean;
    transform?: boolean;
    typeCheck?: boolean;
    scope?: boolean;
    symbol?: boolean;
}

export interface OxcParserOptions {
    allowReturnOutsideFunction?: boolean;
    preserveParens?: boolean;
    sourceType?: "script" | "module";
    sourceFilename?: string;
}

export interface OxcLinterOptions {}

export interface OxcTransformerOptions {
    target?: string;
}

export interface OxcCodegenOptions {
    indentation?: number;
    enableTypescript?: boolean;
}

export interface OxcControlFlowOptions {
    verbose?: boolean;
}

export interface OxcMinifierOptions {
    whitespace?: boolean;
    mangle?: boolean;
    compress?: boolean;
    compressOptions?: OxcCompressOptions;
}

export interface OxcCompressOptions {
    booleans: boolean;
    drop_debugger: boolean;
    drop_console: boolean;
    evaluate: boolean;
    join_vars: boolean;
    loops: boolean;
    typeofs: boolean;
}


import type { Program, Span } from "@oxc-project/types";
export * from "@oxc-project/types";


export interface Oxc {
    ast: Program;
    ir: string;
    controlFlowGraph: string;
    symbols: any;
    scopeText: string;
    codegenText: string;
    formattedText: string;
    prettierFormattedText: string;
    prettierIrText: string;
}

export interface Comment {
    type: CommentType;
    value: string;
    start: number;
    end: number;
}

export type CommentType = "Line" | "Block";


export type NodeId = number;
export type NodeFlags = {
    JSDoc: 1,
    Class: 2,
    HasYield: 4
    Parameter: 8
};



export type SymbolId = number;
export type SymbolFlags = unknown;
export type RedeclarationId = unknown;



export type ScopeId = number;



export type ReferenceId = number;
export type ReferenceFlags = {
    None: 0,
    Read: 0b1,
    Write: 0b10,
    Type: 0b100,
    Value: 0b11
}


export class Oxc {
  free(): void;
  constructor();
  /**
   * Returns Array of String
   * # Errors
   * # Panics
   */
  getDiagnostics(): any[];
  /**
   * Returns comments
   * # Errors
   */
  getComments(): any[];
  /**
   * # Errors
   * Serde serialization error
   */
  run(source_text: string, options: OxcOptions): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_oxc_free: (a: number, b: number) => void;
  readonly __wbg_get_oxc_ast: (a: number) => any;
  readonly __wbg_get_oxc_ir: (a: number) => [number, number];
  readonly __wbg_get_oxc_controlFlowGraph: (a: number) => [number, number];
  readonly __wbg_get_oxc_symbols: (a: number) => any;
  readonly __wbg_get_oxc_scopeText: (a: number) => [number, number];
  readonly __wbg_get_oxc_codegenText: (a: number) => [number, number];
  readonly __wbg_get_oxc_formattedText: (a: number) => [number, number];
  readonly __wbg_get_oxc_prettierFormattedText: (a: number) => [number, number];
  readonly __wbg_get_oxc_prettierIrText: (a: number) => [number, number];
  readonly oxc_new: () => number;
  readonly oxc_getDiagnostics: (a: number) => [number, number, number, number];
  readonly oxc_getComments: (a: number) => [number, number, number, number];
  readonly oxc_run: (a: number, b: number, c: number, d: any) => [number, number];
  readonly browserslist: (a: number, b: number, c: any) => [number, number, number];
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __externref_drop_slice: (a: number, b: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
