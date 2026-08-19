// Node types.
//
// AST nodes are typed with Oxc's own ESTree type definitions.
// The printer is more permissive than they are in 2 respects, which the types below cover:
//
// 1. Its dispatch functions throw on node types they don't know.
//    `UnknownNode` keeps those branches reachable - without it the type-checker narrows the node to `never`
//    and rejects the `node.type` read in the error message.
//
// 2. It also accepts ASTs from other ESTree producers (Acorn, TS-ESLint), which differ from Oxc's AST
//    in a handful of places. The `*Node` aliases widen the properties where they do, and the branches
//    which read a property Oxc's AST doesn't have at all assert its type at the point of use.

import type * as ESTree from "../../../../npm/oxc-types/types.d.ts";

/** A node whose type the printer doesn't handle. */
export interface UnknownNode {
  type: "<unknown node>";
}

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
 * A class body whose members may include a node type the printer does not know.
 */
export type ClassBodyNode = Omit<ESTree.ClassBody, "body"> & {
  body: Array<ESTree.ClassElement | UnknownNode>;
};

/**
 * An `export` whose declaration may be a node type the printer does not know.
 */
export type ExportNamedDeclarationNode = Omit<ESTree.ExportNamedDeclaration, "declaration"> & {
  declaration: ESTree.Declaration | UnknownNode | null;
};

/**
 * A node carrying the offsets needed for source mappings.
 */
export interface MappableNode {
  type: string;
  name?: unknown;
  start?: number;
  end?: number;
}
