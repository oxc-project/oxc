// Node types.
//
// AST nodes are typed with Oxc's own ESTree type definitions.
// The printer is more permissive than they are in 2 respects:
//
// 1. Its dispatch functions throw on node types they don't know - see `throwUnknownNode`.
//
// 2. It also accepts ASTs from other ESTree producers (Acorn, TS-ESLint), which differ from Oxc's AST
//    in a handful of places. The `*Node` aliases widen the properties where they do, and the branches
//    which read a property Oxc's AST doesn't have at all assert its type at the point of use.

import type * as ESTree from "../../../../npm/oxc-types/types.d.ts";

/**
 * Throw an error for a node whose type a dispatch function doesn't handle.
 *
 * Call it from the `default` arm of a dispatch `switch`. `node` is typed `never`, so the call only type-checks
 * if the `switch` has a `case` for every type in the node's union - a type missing from it is a compile error.
 *
 * At runtime it still throws, for a node of a type the printer's types don't cover, as can happen with ASTs
 * from other producers, or ones built by hand.
 *
 * @param node - Node, which the `switch` has narrowed to `never`
 * @param kind - What sort of node it should have been, for the error message
 */
export function throwUnknownNode(node: never, kind: string): never {
  throw new Error(`Unknown ${kind}: ${(node as { type: string }).type}`);
}

/**
 * The node types a `switch (node.type)` has not handled, with one variant per remaining `type`.
 *
 * Use it in the `default` arm, as `UnhandledNodes<typeof node, typeof node.type>`.
 *
 * Some AST types cover several node types with one `type` field which is a union -
 * e.g. `ESTree.Function`'s `type` is `"FunctionDeclaration" | "FunctionExpression" | ...`.
 * A `case` for one of those can't remove the whole type from `typeof node`, but it does remove that one
 * from `typeof node.type`. So this splits each type into one variant per `type` value not yet handled,
 * e.g. `ESTree.Function & { type: "FunctionDeclaration" }`. The `default` arm can then `Exclude` exactly
 * the variants which can't reach it, and any variant it doesn't exclude is a compile error.
 */
export type UnhandledNodes<Node, RemainingType> = Node extends { type: infer Type }
  ? RemainingType extends Type
    ? Node & { type: RemainingType }
    : never
  : never;

/** An `Expression` which is a literal. */
export type Literal = Extract<ESTree.Expression, { type: "Literal" }>;

/**
 * Properties which only some literals carry. `printLiteral` reads them on any literal to tell
 * regexes and bigints apart from the rest.
 */
export interface LiteralExtras {
  regex?: ESTree.RegExpLiteral["regex"];
  bigint?: string;
}

/**
 * Oxc marks abstract class members with a `TSAbstract*` node type, where others use an `abstract` flag.
 */
export type MethodDefinitionNode = ESTree.MethodDefinition & { abstract?: boolean };

/**
 * A class field, which Oxc may mark abstract with a flag rather than a `TSAbstract*` type.
 */
export type PropertyDefinitionNode = ESTree.PropertyDefinition & { abstract?: boolean };

/**
 * An `accessor` field, marked abstract the same way as `PropertyDefinitionNode`.
 */
export type AccessorPropertyNode = ESTree.AccessorProperty & { abstract?: boolean };

/**
 * A node carrying the offsets needed for source mappings.
 *
 * No `name` - a mapping recorded for one of these carries none. `NamedMappableNode` below is the
 * shape the named forms take, and requiring `name` on it is what steers a named call which cannot
 * produce a name to the plain form instead.
 */
export interface MappableNode {
  type: string;
  start?: number;
  end?: number;
}

/**
 * A node whose mapping records the name it had in the source.
 *
 * These are the identifier kinds Rust `oxc_codegen` names mappings for - identifier references and
 * names, binding, label, JSX and private identifiers.
 */
export interface NamedMappableNode extends MappableNode {
  name: string;
}

/**
 * An `Identifier` node.
 *
 * This does not include `JSXIdentifier` or `PrivateIdentifier`. A `JSXIdentifier`'s or `PrivateIdentifier`'s name
 * is recovered from the source by a different scan, so it goes to a write function of its own.
 */
export interface IdentMappableNode extends NamedMappableNode {
  type: "Identifier";
}

/**
 * A node whose mapping records no name, because it has no string `name` to record.
 *
 * `name?: object` is how a node which does have one is rejected - `string` is not assignable to it.
 * A `name` holding a node rather than a string, as a JSX element's does, is not a name a mapping could record,
 * and stays allowed.
 */
export interface UnnamedMappableNode extends MappableNode {
  name?: object;
}
