// Lazy implementation
/*
// Visitor object returned by a `Rule`'s `create` function.
export interface Visitor {
  [key: string]: VisitFn;
}
*/

import type { Span } from "./location.ts";
import type { Token } from "./tokens.ts";

import type { VisitorObject as Visitor } from "../generated/visitor.d.ts";
export type { Visitor };

// Hook function that runs before traversal.
// If returns `false`, traversal is skipped for the rule.
export type BeforeHook = () => boolean | void;

// Hook function that runs after traversal.
export type AfterHook = () => void;

// Visitor object returned by a `Rule`'s `createOnce` function.
export type VisitorWithHooks = Visitor & {
  before?: BeforeHook;
  after?: AfterHook;
};

// AST node type.
export interface Node extends Span {}

export type NodeOrToken = Node | Token | Comment;

// Comment.
export interface Comment extends Span {
  type: "Line" | "Block";
  value: string;
}

// Buffer with typed array views of itself stored as properties.
export interface BufferWithArrays extends Uint8Array {
  uint32: Uint32Array;
  float64: Float64Array;
}
