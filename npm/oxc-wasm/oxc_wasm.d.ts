/* tslint:disable */
/* eslint-disable */
/**
 * @param {string} query
 * @param {any} opts
 * @returns {any}
 */
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
  sourceType?: 'script' | 'module';
  sourceFilename?: string;
}

export interface OxcLinterOptions {}

export interface OxcTransformerOptions {}

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

import type { Program, Span } from '@oxc-project/types';
export * from '@oxc-project/types';

export interface Oxc {
  ast: Program;
  ir: string;
  controlFlowGraph: string;
  symbols: SymbolTable;
  scopeText: string;
  codegenText: string;
  formattedText: string;
  prettierFormattedText: string;
  prettierIrText: string;
  comments: Comment[];
  diagnostics: Error[];
}

export interface Comment {
  type: CommentType;
  value: string;
  start: number;
  end: number;
}

export type CommentType = 'Line' | 'Block';

export type IndexVec<I, T> = Array<T>;
export type CompactStr = string;

export interface SymbolTable {
  spans: IndexVec<SymbolId, Span>;
  names: IndexVec<SymbolId, CompactStr>;
  flags: IndexVec<SymbolId, SymbolFlags>;
  scopeIds: IndexVec<SymbolId, ScopeId>;
  declarations: IndexVec<SymbolId, NodeId>;
  resolvedReferences: IndexVec<SymbolId, ReferenceId[]>;
  redeclarations: IndexVec<SymbolId, RedeclarationId | null>;
  redeclarationSpans: IndexVec<RedeclarationId, Span[]>;
  references: IndexVec<ReferenceId, Reference>;
}

export interface Reference {
  nodeId: NodeId;
  symbolId: SymbolId | null;
  flags: ReferenceFlags;
}

export type NodeId = number;
export type NodeFlags = {
  JSDoc: 1;
  Class: 2;
  HasYield: 4;
  Parameter: 8;
};

export type SymbolId = number;
export type SymbolFlags = unknown;
export type RedeclarationId = unknown;

export type ReferenceId = number;
export type ReferenceFlags = {
  None: 0;
  Read: 0b1;
  Write: 0b10;
  Type: 0b100;
  Value: 0b11;
};

export type ScopeId = number;

export class Oxc {
  free(): void;
  constructor();
  /**
   * Returns Array of String
   * # Errors
   * # Panics
   * @returns {any[]}
   */
  getDiagnostics(): any[];
  /**
   * Returns comments
   * # Errors
   * @returns {any[]}
   */
  getComments(): any[];
  /**
   * # Errors
   * Serde serialization error
   * @param {string} source_text
   * @param {OxcOptions} options
   */
  run(source_text: string, options: OxcOptions): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_oxc_free: (a: number, b: number) => void;
  readonly __wbg_get_oxc_ast: (a: number) => number;
  readonly __wbg_get_oxc_ir: (a: number, b: number) => void;
  readonly __wbg_get_oxc_controlFlowGraph: (a: number, b: number) => void;
  readonly __wbg_get_oxc_symbols: (a: number) => number;
  readonly __wbg_get_oxc_scopeText: (a: number, b: number) => void;
  readonly __wbg_get_oxc_codegenText: (a: number, b: number) => void;
  readonly __wbg_get_oxc_formattedText: (a: number, b: number) => void;
  readonly __wbg_get_oxc_prettierFormattedText: (a: number, b: number) => void;
  readonly __wbg_get_oxc_prettierIrText: (a: number, b: number) => void;
  readonly oxc_new: () => number;
  readonly oxc_getDiagnostics: (a: number, b: number) => void;
  readonly oxc_getComments: (a: number, b: number) => void;
  readonly oxc_run: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly browserslist: (a: number, b: number, c: number, d: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
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
export default function __wbg_init(
  module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>,
): Promise<InitOutput>;
