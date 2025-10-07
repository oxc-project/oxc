// Lazy implementation
/*
// Visitor object returned by a `Rule`'s `create` function.
export interface Visitor {
  [key: string]: VisitFn;
}
*/

import type { VisitorObject as Visitor } from '../generated/visitor.d.ts';
export type { Visitor };

// Hook function that runs before traversal.
// If returns `false`, traversal is skipped for the rule.
export type BeforeHook = () => boolean | void;

// Hook function that runs after traversal.
export type AfterHook = () => void;

// Visitor object returned by a `Rule`'s `createOnce` function.
export interface VisitorWithHooks extends Visitor {
  before?: BeforeHook;
  after?: AfterHook;
}

// Visit function for a specific AST node type.
export type VisitFn = (node: Node) => void;

// Range of source offsets.
export type Range = [number, number];

// Interface for any type which has `range` field
export interface Ranged {
  range: Range;
}

// Interface for any type which has location properties.
export interface Span extends Ranged {
  start: number;
  end: number;
  loc: Location;
}

// Source code location.
export interface Location {
  start: LineColumn;
  end: LineColumn;
}

// Line number + column number pair.
// `line` is 1-indexed, `column` is 0-indexed.
export interface LineColumn {
  line: number;
  column: number;
}

// AST node type.
export interface Node extends Span {}

// AST token type.
export interface Token extends Span {
  type: string;
  value: string;
}

// Currently we only support `Node`s, but will add support for `Token`s later.
export type NodeOrToken = Node | Token;

// Comment.
export interface Comment extends Span {
  type: 'Line' | 'Block';
  value: string;
}

// Element of compiled visitor array.
// * `VisitFn | null` for leaf nodes.
// * `EnterExit | null` for non-leaf nodes.
export type CompiledVisitorEntry = VisitFn | EnterExit | null;

// Enter+exit pair, for non-leaf nodes in compiled visitor.
export interface EnterExit {
  enter: VisitFn | null;
  exit: VisitFn | null;
}

// Rule metadata.
// TODO: Fill in all properties.
export interface RuleMeta {
  fixable?: 'code' | 'whitespace' | null | undefined;
  messages?: Record<string, string>;
  [key: string]: unknown;
}

// Buffer with typed array views of itself stored as properties.
export interface BufferWithArrays extends Uint8Array {
  uint32: Uint32Array;
  float64: Float64Array;
}
