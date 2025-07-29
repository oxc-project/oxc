// Visitor object returned by a `Rule`'s `create` function.
export interface Visitor {
  [key: string]: VisitFn;
}

// Visit function for a specific AST node type.
export type VisitFn = (node: Node) => void;

// AST node type.
// We'll make this type a union later on.
export type Node = Object;
