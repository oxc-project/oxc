// Visitor object returned by a `Rule`'s `create` function.
export interface Visitor {
  [key: string]: VisitFn;
}

// Visit function for a specific AST node type.
export type VisitFn = (node: Node) => void;

// AST node type.
// We'll make this type a union later on.
export type Node = { type: string; start: number; end: number; [key: string]: unknown };

// Element of compiled visitor array.
// * `VisitFn | null` for leaf nodes.
// * `EnterExit | null` for non-leaf nodes.
export type CompiledVisitorEntry = VisitFn | EnterExit | null;

// Enter+exit pair, for non-leaf nodes in compiled visitor.
export interface EnterExit {
  enter: VisitFn | null;
  exit: VisitFn | null;
}
