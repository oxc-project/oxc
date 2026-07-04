/*
 * AST walker for ASTs produced by custom (JS) parsers.
 *
 * The buffer-based visitor (`visitor.ts` + `generated/walk.js`) is keyed by Oxc node type IDs,
 * so it cannot handle the arbitrary node types which custom parsers can produce
 * (e.g. Ember's `GlimmerTemplate`). This walker uses string-keyed dispatch instead,
 * and traverses nodes dynamically via visitor keys (falling back to iterating object keys
 * for unknown node types), like ESLint does.
 *
 * This is a cold path (only used for files parsed by custom parsers),
 * so it favors clarity over micro-optimization.
 */

import esquery from "esquery";
import { NODE_TYPE_IDS_MAP, NODE_TYPES_COUNT } from "../generated/type_ids.ts";
import { parseSelector, IDENTIFIER_COUNT_INCREMENT } from "./selector.ts";
import { debugAssert } from "../utils/asserts.ts";

import type { ESQueryOptions, Selector as EsquerySelector } from "esquery";
import type { Node as EsqueryNode } from "estree";
import type { JsParserNode } from "./parsers.ts";
import type { Visitor } from "./types.ts";

const { matches: esqueryMatches } = esquery;

// Visit function for an AST node produced by a custom parser.
export type JsVisitFn = (node: JsParserNode) => void;

// Handler for one property of a visitor object.
interface Handler {
  // `esquery` selector to match, or `null` if handler matches unconditionally
  // (bare node type keys, and the universal selector `*`)
  esquerySelector: EsquerySelector | null;
  // Specificity of selector. Same scheme as `selector.ts` (identifier + attribute counts).
  // Handlers matching a node are called in ascending order of specificity, like ESLint.
  specificity: number;
  // Selector string (without trailing `:exit`). Used as tie-breaker for specificity ordering.
  selectorStr: string;
  // Visit function
  fn: JsVisitFn;
}

/**
 * Compiled visitor for ASTs produced by custom (JS) parsers.
 */
export interface CompiledJsVisitor {
  // Handlers keyed by node type (for bare node type keys e.g. `Program`, `CustomTemplate`).
  // Each array is sorted by (specificity, selectorStr).
  enterTypes: Map<string, Handler[]>;
  exitTypes: Map<string, Handler[]>;
  // Handlers for selectors (e.g. `VariableDeclarator > Identifier`) and the universal selector `*`.
  // Sorted by (specificity, selectorStr).
  enterSelectors: Handler[];
  exitSelectors: Handler[];
  // `true` if any handlers exist
  hasVisitors: boolean;
}

// Matches keys which are bare node type names (e.g. `Program`, `CustomTemplate`),
// as opposed to selectors (e.g. `Program > ExpressionStatement`, `:matches(X, Y)`).
const BARE_TYPE_REGEX = /^[A-Za-z_$][A-Za-z0-9_$]*$/;

/**
 * Compile visitor objects returned by rules' `create` / `createOnce` methods
 * into a single string-keyed compiled visitor.
 *
 * @param visitors - Visitor objects
 * @returns Compiled visitor
 * @throws {TypeError} If a visitor is not an object, or one of its properties is not a function
 * @throws {Error} If a visitor uses code path analysis events (`onCodePathStart` etc.),
 *   which are not supported for files parsed by custom parsers
 */
export function compileJsVisitors(visitors: Visitor[]): CompiledJsVisitor {
  const compiled: CompiledJsVisitor = {
    enterTypes: new Map(),
    exitTypes: new Map(),
    enterSelectors: [],
    exitSelectors: [],
    hasVisitors: false,
  };

  for (const visitor of visitors) {
    if (visitor === null || typeof visitor !== "object") {
      throw new TypeError("Visitor returned from `create` method must be an object");
    }

    for (const key of Object.keys(visitor)) {
      const fn = visitor[key] as unknown as JsVisitFn;
      if (typeof fn !== "function") {
        throw new TypeError(`'${key}' property of visitor object is not a function`);
      }

      compiled.hasVisitors = true;

      const isExit = key.endsWith(":exit");
      const name = isExit ? key.slice(0, -5) : key;

      // Reject code path analysis event handlers (`onCodePathStart` etc.).
      // CFG construction is tied to the buffer-based lint path, and is not available
      // for ASTs produced by custom parsers.
      const typeId = NODE_TYPE_IDS_MAP.get(name);
      if (typeId !== undefined && typeId >= NODE_TYPES_COUNT) {
        throw new Error(
          `Rules using code path analysis ('${name}') are not supported ` +
            "for files parsed by a custom parser",
        );
      }

      if (name === "*") {
        // Universal selector. Matches all nodes, with the lowest specificity (0).
        addHandler(compiled, isExit, null, {
          esquerySelector: null,
          specificity: 0,
          selectorStr: name,
          fn,
        });
      } else if (BARE_TYPE_REGEX.test(name)) {
        // Bare node type name. Matched by string comparison on `node.type`,
        // so works for node types unknown to Oxc (e.g. `GlimmerTemplate`).
        // Specificity is 1 identifier, same as ESLint treats a bare type selector.
        addHandler(compiled, isExit, name, {
          esquerySelector: null,
          specificity: IDENTIFIER_COUNT_INCREMENT,
          selectorStr: name,
          fn,
        });
      } else {
        // Selector. Parse with `esquery` (via `selector.ts` cache, which also computes specificity).
        // Matching is done with `esquery.matches` during the walk - the type-ID based fast paths
        // in `selector.ts` cannot be used, as they don't know about custom node types.
        const selector = parseSelector(name);
        addHandler(compiled, isExit, null, {
          esquerySelector: selector.esquerySelector,
          specificity: selector.specificity,
          selectorStr: name,
          fn,
        });
      }
    }
  }

  // Sort handlers by (specificity, selectorStr), matching ESLint's `NodeEventGenerator` ordering
  for (const handlers of compiled.enterTypes.values()) handlers.sort(compareHandlers);
  for (const handlers of compiled.exitTypes.values()) handlers.sort(compareHandlers);
  compiled.enterSelectors.sort(compareHandlers);
  compiled.exitSelectors.sort(compareHandlers);

  return compiled;
}

/**
 * Add a handler to compiled visitor.
 *
 * @param compiled - Compiled visitor
 * @param isExit - `true` if handler is an exit handler
 * @param type - Node type name for bare type handlers, or `null` for selector / `*` handlers
 * @param handler - Handler
 */
function addHandler(
  compiled: CompiledJsVisitor,
  isExit: boolean,
  type: string | null,
  handler: Handler,
): void {
  if (type === null) {
    (isExit ? compiled.exitSelectors : compiled.enterSelectors).push(handler);
    return;
  }

  const typesMap = isExit ? compiled.exitTypes : compiled.enterTypes;
  let handlers = typesMap.get(type);
  if (handlers === undefined) {
    handlers = [];
    typesMap.set(type, handlers);
  }
  handlers.push(handler);
}

/**
 * Compare handlers by (specificity, selectorStr).
 * @param a - First handler
 * @param b - Second handler
 * @returns Negative if `a` sorts first, positive if `b` sorts first, 0 if equal
 */
function compareHandlers(a: Handler, b: Handler): number {
  const diff = a.specificity - b.specificity;
  if (diff !== 0) return diff;

  const strA = a.selectorStr,
    strB = b.selectorStr;
  return strA === strB ? 0 : strA < strB ? -1 : 1;
}

/**
 * Walk an AST produced by a custom parser, calling handlers of compiled visitor.
 *
 * Sets `parent` property on every visited node (like ESLint does),
 * with the root node's `parent` set to `null`.
 *
 * @param ast - Root AST node (`Program`)
 * @param visitorKeys - Visitor keys to traverse the AST with
 *   (merged parser-provided + default keys; unknown node types fall back to iterating object keys)
 * @param compiled - Compiled visitor
 */
export function walkParserAst(
  ast: JsParserNode,
  visitorKeys: Record<string, readonly string[]>,
  compiled: CompiledJsVisitor,
): void {
  const { enterTypes, exitTypes, enterSelectors, exitSelectors } = compiled;

  // Options for `esquery.matches`.
  // `fallback` provides keys for node types which `visitorKeys` doesn't cover.
  const esqueryOptions: ESQueryOptions = {
    nodeTypeKey: "type",
    visitorKeys: visitorKeys as Record<string, string[]>,
    fallback: getFallbackKeys as unknown as (node: EsqueryNode) => string[],
  };

  // Ancestors of node currently being visited, nearest parent first (order `esquery` expects)
  const ancestry: JsParserNode[] = [];

  /**
   * Call all handlers matching `node`, in ascending order of specificity.
   *
   * `typeHandlers` and `selectorHandlers` are each sorted by (specificity, selectorStr),
   * so a two-pointer merge visits handlers in the correct order.
   *
   * @param node - AST node
   * @param typeHandlers - Handlers for `node`'s type, or `undefined` if none
   * @param selectorHandlers - Selector / `*` handlers
   */
  function callHandlers(
    node: JsParserNode,
    typeHandlers: Handler[] | undefined,
    selectorHandlers: Handler[],
  ): void {
    const typesLen = typeHandlers === undefined ? 0 : typeHandlers.length,
      selectorsLen = selectorHandlers.length;

    let typeIndex = 0,
      selectorIndex = 0;
    while (typeIndex < typesLen || selectorIndex < selectorsLen) {
      let handler: Handler;
      if (
        typeIndex < typesLen &&
        (selectorIndex === selectorsLen ||
          compareHandlers(typeHandlers![typeIndex], selectorHandlers[selectorIndex]) <= 0)
      ) {
        handler = typeHandlers![typeIndex++];
      } else {
        handler = selectorHandlers[selectorIndex++];
        // Selector handlers must match the node.
        // `esquerySelector` is `null` for the universal selector `*`, which matches all nodes.
        const { esquerySelector } = handler;
        if (
          esquerySelector !== null &&
          !esqueryMatches(
            node as unknown as EsqueryNode,
            esquerySelector,
            ancestry as unknown as EsqueryNode[],
            esqueryOptions,
          )
        ) {
          continue;
        }
      }

      (0, handler.fn)(node);
    }
  }

  /**
   * Visit a node and its children, depth-first.
   * @param node - AST node
   * @param parent - Parent node, or `null` for the root node
   */
  function visitNode(node: JsParserNode, parent: JsParserNode | null): void {
    // Set `parent` on the node, like ESLint does (plain enumerable assignment)
    node.parent = parent;

    callHandlers(node, enterTypes.get(node.type), enterSelectors);

    const keys = visitorKeys[node.type] ?? getFallbackKeys(node);

    ancestry.unshift(node);
    for (let i = 0, keysLen = keys.length; i < keysLen; i++) {
      const child = node[keys[i]];
      if (Array.isArray(child)) {
        for (let j = 0, childLen = child.length; j < childLen; j++) {
          const element: unknown = child[j];
          if (isNode(element)) visitNode(element, node);
        }
      } else if (isNode(child)) {
        visitNode(child, node);
      }
    }
    ancestry.shift();

    callHandlers(node, exitTypes.get(node.type), exitSelectors);
  }

  visitNode(ast, null);

  debugAssert(ancestry.length === 0, "`ancestry` should be empty after walking AST");
}

/**
 * Check if a value is an AST node (object with a string `type` property).
 * Same check as ESLint's `Traverser` uses.
 * @param value - Value to check
 * @returns `true` if `value` is an AST node
 */
function isNode(value: unknown): value is JsParserNode {
  return (
    value !== null &&
    typeof value === "object" &&
    typeof (value as { type?: unknown }).type === "string"
  );
}

/**
 * Get visitor keys for a node whose type is not covered by visitor keys.
 *
 * Returns the node's own enumerable keys, minus `parent`, `range`, `loc`, `type`,
 * `leadingComments`, `trailingComments`, and keys starting with `_`.
 * The last three exclusions match `eslint-visitor-keys`' `getKeys`, which ESLint uses as
 * its fallback - they prevent descending into comment attachments (comment objects have
 * a string `type`, so would otherwise be visited as nodes) and parser-internal properties.
 * (Values which are not nodes / arrays of nodes are filtered out during the walk.)
 *
 * @param node - AST node
 * @returns Keys to visit
 */
export function getFallbackKeys(node: JsParserNode): string[] {
  const keys = [];
  for (const key of Object.keys(node)) {
    if (
      key !== "parent" &&
      key !== "range" &&
      key !== "loc" &&
      key !== "type" &&
      key !== "leadingComments" &&
      key !== "trailingComments" &&
      !key.startsWith("_")
    ) {
      keys.push(key);
    }
  }
  return keys;
}
