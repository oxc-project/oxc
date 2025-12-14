import esquery from "esquery";
import visitorKeys from "../generated/keys.ts";
import { FUNCTION_NODE_TYPE_IDS, NODE_TYPE_IDS_MAP } from "../generated/type_ids.ts";
// @ts-expect-error - TODO: We need to generate `.d.ts` file for this module
import { ancestors } from "../generated/walk.js";
import { debugAssert } from "../utils/asserts.ts";

import type { ESQueryOptions, Selector as EsquerySelector } from "esquery";
import type { Node as EsqueryNode } from "estree";
import type { Node } from "./types.ts";
import type { VisitFn } from "./visitor.ts";

const ObjectKeys = Object.keys;

const { matches: esqueryMatches, parse: esqueryParse } = esquery;

type NodeTypeId = number;

// Options to call `esquery.matches` with.
const ESQUERY_OPTIONS: ESQueryOptions = {
  nodeTypeKey: "type",
  visitorKeys,
  fallback: (node: EsqueryNode) => ObjectKeys(node).filter(filterKey),
  matchClass: (_className: unknown, _node: EsqueryNode, _ancestors: EsqueryNode[]) => false, // TODO: Is this right?
};
const filterKey = (key: string) => key !== "parent" && key !== "range" && key !== "loc";

// Specificity is a combination of:
//
// 1. Identifier count
// 2. Attribute count
// 3. Exit flag (set for exit visit fns)
//
// ESLint stores identifier count and attribute count in separate properties.
// As an optimization, we store them together in a single integer.
// This makes sorting an array of visit fns by specificity faster.
//
// Attribute count takes precedence in sorting, so goes in the higher bits.
//
// V8 stores small integers ("SMI"s) inline in objects, instead of on heap.
// When V8 pointer compression is enabled, SMIs are 31-bit signed integers.
// Here we're using signed integers, so are limited to 30 bits.
//
// We use:
// * 15 bits for identifier count (32767 max)
// * 14 bits for attribute count (16383 max)
// * 1 bit for exit flag.
//
// It seems inconceivable that a selector could exceed these limits.
const IDENTIFIER_COUNT_BITS = 15;
const ATTRIBUTE_COUNT_BITS = 14;

const IDENTIFIER_COUNT_SHIFT = 0;
const ATTRIBUTE_COUNT_SHIFT = IDENTIFIER_COUNT_BITS;
const EXIT_FLAG_SHIFT = IDENTIFIER_COUNT_BITS + ATTRIBUTE_COUNT_BITS;

export const IDENTIFIER_COUNT_INCREMENT = 1 << IDENTIFIER_COUNT_SHIFT;
const ATTRIBUTE_COUNT_INCREMENT = 1 << ATTRIBUTE_COUNT_SHIFT;
export const EXIT_FLAG = 1 << EXIT_FLAG_SHIFT;

const IDENTIFIER_COUNT_MAX = (1 << IDENTIFIER_COUNT_BITS) - 1;
const ATTRIBUTE_COUNT_MAX = (1 << ATTRIBUTE_COUNT_BITS) - 1;

function identifierCount(specificity: number): number {
  return (specificity >> IDENTIFIER_COUNT_SHIFT) & IDENTIFIER_COUNT_MAX;
}

function attributeCount(specificity: number): number {
  return (specificity >> ATTRIBUTE_COUNT_SHIFT) & ATTRIBUTE_COUNT_MAX;
}

// Parsed selector.
interface Selector {
  // Array of IDs of types this selector matches, or `null` if selector matches all types.
  typeIds: NodeTypeId[] | null;
  // `esquery` selector object for this selector.
  esquerySelector: EsquerySelector;
  // `true` if selector applies matching beyond just filtering on node type.
  // * `FunctionExpression > Identifier` is complex.
  // * `:matches(FunctionExpression, FunctionDeclaration)` is not complex.
  // Primarily this exists to make simple `:matches` faster.
  isComplex: boolean;
  // Specificity of selector.
  // See comment above `IDENTIFIER_COUNT_BITS` for more details.
  specificity: number;
}

// Cache of parsed `Selector`s.
const cache: Map<string, Selector> = new Map([]);

const EMPTY_TYPE_IDS_ARRAY: NodeTypeId[] = [];

/**
 * Parse a selector string and return a `Selector` object which represents it.
 *
 * @param key - Selector string e.g. `Program > VariableDeclaration`
 * @returns `Selector` object
 */
export function parseSelector(key: string): Selector {
  // Used cached object if we've parsed this key before
  let selector = cache.get(key);
  if (selector !== undefined) return selector;

  // Parse with `esquery` and analyse
  const esquerySelector = esqueryParse(key);

  selector = {
    typeIds: null,
    esquerySelector,
    isComplex: false,
    specificity: 0,
  };
  selector.typeIds = analyzeSelector(esquerySelector, selector);

  // Store in cache for next time
  cache.set(key, selector);

  return selector;
}

/**
 * Analyse an `EsquerySelector` to determine:
 *
 * 1. What node types it matches on.
 * 2. Whether it is "simple" or "complex" - "simple" matches a subset of node types without further conditions.
 * 3. It's specificity (number of identifiers and attributes).
 *
 * This function traverses the `EsquerySelector` and calls itself recursively.
 * It returns an array of node type IDs which the selector may match.
 *
 * @param esquerySelector - `EsquerySelector` to analyse.
 * @param selector - `Selector` which has its `isComplex` and `specificity` properties updated.
 * @returns Array of node type IDs the selector matches, or `null` if it matches all nodes.
 */
function analyzeSelector(
  esquerySelector: EsquerySelector,
  selector: Selector,
): NodeTypeId[] | null {
  switch (esquerySelector.type) {
    case "identifier": {
      debugAssert(identifierCount(selector.specificity) < IDENTIFIER_COUNT_MAX);
      selector.specificity += IDENTIFIER_COUNT_INCREMENT;

      const typeId = NODE_TYPE_IDS_MAP.get(esquerySelector.value);
      // If the type is invalid, just treat this selector as not matching any types.
      // But still increment identifier count.
      // This matches ESLint's behavior.
      return typeId === undefined ? EMPTY_TYPE_IDS_ARRAY : [typeId];
    }

    case "not":
      for (
        let i = 0, childSelectors = esquerySelector.selectors, len = childSelectors.length;
        i < len;
        i++
      ) {
        analyzeSelector(childSelectors[i], selector);
      }
      selector.isComplex = true;
      return null;

    case "matches": {
      // OR matcher. Matches a node if any of child selectors matches it.
      let nodeTypes: NodeTypeId[] | null = [];
      for (
        let i = 0, childSelectors = esquerySelector.selectors, len = childSelectors.length;
        i < len;
        i++
      ) {
        const childNodeTypes = analyzeSelector(childSelectors[i], selector);
        if (childNodeTypes === null) {
          nodeTypes = null;
        } else if (nodeTypes !== null) {
          nodeTypes.push(...childNodeTypes);
        }
      }
      if (nodeTypes === null) return null;
      // De-duplicate
      // TODO: Faster way to do this? Sort and then dedupe manually?
      return [...new Set(nodeTypes)];
    }

    case "compound": {
      // AND matcher. Only matches a node if all child selectors match it.
      const childSelectors = esquerySelector.selectors,
        len = childSelectors.length;
      // TODO: Can `childSelectors` have 0 length?
      if (len === 0) return [];

      let nodeTypes: NodeTypeId[] | null = null;
      for (let i = 0; i < len; i++) {
        const childNodeTypes = analyzeSelector(childSelectors[i], selector);

        // If child selector matches all types, does not narrow the types the selector matches
        if (childNodeTypes === null) continue;

        if (nodeTypes === null) {
          // First child selector which matches specific types
          nodeTypes = childNodeTypes;
        } else {
          // Selector only matches intersection of all child selectors.
          // TODO: Could make this faster if `analyzeSelector` always returned an ordered array.
          nodeTypes = childNodeTypes.filter((nodeType) => nodeTypes!.includes(nodeType));
        }
      }
      return nodeTypes;
    }

    case "attribute":
    case "field":
    case "nth-child":
    case "nth-last-child":
      selector.isComplex = true;
      debugAssert(attributeCount(selector.specificity) < ATTRIBUTE_COUNT_MAX);
      selector.specificity += ATTRIBUTE_COUNT_INCREMENT;
      return null;

    case "child":
    case "descendant":
    case "sibling":
    case "adjacent":
      selector.isComplex = true;
      analyzeSelector(esquerySelector.left, selector);
      return analyzeSelector(esquerySelector.right, selector);

    case "class":
      // TODO: Should TS function types be included in `FUNCTION_NODE_TYPE_IDS`?
      // This TODO comment is from ESLint's implementation. Not sure what it means!
      // TODO: Abstract into JSLanguage somehow.
      if (esquerySelector.name === "function") return FUNCTION_NODE_TYPE_IDS;
      selector.isComplex = true;
      return null;

    case "wildcard":
      return null;

    default:
      selector.isComplex = true;
      return null;
  }
}

/**
 * Wrap a visit function so it's only called if the provided `EsquerySelector` matches the AST node.
 *
 * IMPORTANT: Selector matching will only be correct if `ancestors` from `generated/walk.js`
 * contains the ancestors of the AST node passed to the returned visit function.
 * Therefore, the returned visit function can only be called during AST traversal.
 *
 * @param visitFn - Visit function to wrap
 * @param esquerySelector - `EsquerySelector` object
 * @returns Wrapped visit function
 */
export function wrapVisitFnWithSelectorMatch(
  visitFn: VisitFn,
  esquerySelector: EsquerySelector,
): VisitFn {
  return (node: Node) => {
    if (
      esqueryMatches(node as unknown as EsqueryNode, esquerySelector, ancestors, ESQUERY_OPTIONS)
    ) {
      visitFn(node);
    }
  };
}
