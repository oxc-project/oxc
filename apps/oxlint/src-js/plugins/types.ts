// Lazy implementation
/*
// Visitor object returned by a `Rule`'s `create` function.
export interface Visitor {
  [key: string]: VisitFn;
}
*/

import type { VisitorObject as Visitor } from '../../dist/generated/visit/visitor.d.ts';
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

// Internal interface for any type which has `start` and `end` properties.
// We'll add `range` and `loc` properties to this later.
interface Spanned {
  start: number;
  end: number;
}

// AST node type.
export interface Node extends Spanned {}

// AST token type.
export interface Token extends Spanned {
  type: string;
  value: string;
}

// Currently we only support `Node`s, but will add support for `Token`s later.
export type NodeOrToken = Node | Token;

// Comment.
export interface Comment extends Spanned {
  type: 'Line' | 'Block';
  value: string;
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
  [key: string]: unknown;
}
