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

// AST node type.
export interface Node {
  start: number;
  end: number;
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
