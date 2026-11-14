import { getTypeAtLocation as getTypeAtLocationJSON } from "../bindings.js";
import { filePath } from "./context.ts";

import type { Node } from "../generated/types.d.ts";
import type { Range } from "./location.ts";

/**
 * Type of a node, from the `tsrs-lint` process which runs type-aware rules.
 *
 * Experimental.
 */
export interface TypeInfo {
  /** Type as printed by TypeScript's `checker.typeToString`. */
  text: string;
  /** TypeScript `TypeFlags` of the type. */
  flags: number;
  /** Kind of the TypeScript node the type is for, e.g. `"Identifier"`. */
  nodeKind: string;
}

/**
 * Parser services for the file.
 *
 * Type information is only available when linting with `--type-aware` and `tsrs-lint`.
 */
export const PARSER_SERVICES = Object.freeze({
  /**
   * Get the type of a node.
   *
   * The type is for the innermost TypeScript node with the same range as `node`.
   *
   * @param node - AST node
   * @returns Type of the node, or `null` if no TypeScript node has the same range
   * @throws {Error} If type information is unavailable
   */
  getTypeAtLocation(node: Node): TypeInfo | null {
    if (filePath === null) throw new Error("Cannot get type information in `createOnce`");
    const range = getTypeScriptRange(node);
    const json = getTypeAtLocationJSON(filePath, range[0], range[1]);
    return json === null ? null : JSON.parse(json);
  },
});

/**
 * Get the range of the TypeScript node which corresponds to `node`, where ESTree and TypeScript ranges differ.
 */
function getTypeScriptRange(node: Node): Range {
  if (node.type === "Identifier") {
    if (node.typeAnnotation != null) return [node.start, node.start + node.name.length];
  } else if (node.type === "FunctionExpression") {
    const { parent } = node;
    if (
      parent.type === "MethodDefinition"
      || parent.type === "TSAbstractMethodDefinition"
      || (parent.type === "Property"
        && parent.value === node
        && (parent.method || parent.kind !== "init"))
    ) {
      return parent.range;
    }
  }
  return node.range;
}
