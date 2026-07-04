/*
 * Compute "masked regions" for a file parsed by a custom (JS) parser.
 *
 * A masked region is the span of a top-most AST node whose type is unknown to Oxc
 * (e.g. Ember's `GlimmerTemplate`). Rust uses the regions to build a "shadow source" -
 * the original source text with each masked region replaced in-place by a valid,
 * same-byte-length JS placeholder - which is then parsed by the native parser so that
 * native rules can run on the file. Diagnostics inside masked regions are discarded
 * on Rust side.
 *
 * Each region also carries `refs`: expressions which must appear in the placeholder
 * for native rules to see usage that really occurs inside the custom syntax. Rust
 * injects these into the placeholder (e.g. `${ref}` inside a template literal).
 * A ref is one of:
 *
 * - A variable name, for variables referenced inside the region (per the parser's
 *   scope manager) but declared outside all regions - so `no-unused-vars` sees a
 *   component referenced only inside an Ember `<template>` as used.
 * - `this`, when the region contains a `this` usage (a `ThisExpression`, or Glimmer's
 *   `ThisHead` for `{{this.foo}}` paths) and `this` is lexically valid at the region -
 *   so `class-methods-use-this` doesn't flag a method whose only `this` usage is
 *   inside its template.
 * - `this.#name`, when the region contains a `PrivateIdentifier` usage and `#name` is
 *   declared by an enclosing class outside all regions - so
 *   `no-unused-private-class-members` sees the usage. (Glimmer `{{this.#x}}` paths do
 *   NOT produce this - Glimmer resolves `#x` as a plain property name, not a private
 *   field, so `tail` entries starting with `#` are deliberately ignored.)
 *
 * Spans are UTF-16 offsets into the source text (converted to UTF-8 on Rust side).
 */

import oxcVisitorKeys from "../generated/keys.ts";
import { getFallbackKeys } from "./js_ast_walk.ts";

import type { JsParserNode, JsParserProgram, JsParserScopeManager } from "./parsers.ts";

// Masked region in form sent to Rust (`JsMaskedRegion` on Rust side). Spans are UTF-16 offsets.
export interface MaskedRegionReport {
  start: number;
  end: number;
  /** `true` if the region is a class element (parent node is a `ClassBody`) */
  classMember: boolean;
  /**
   * Expressions to inject into the placeholder so native rules see usage occurring
   * inside the region: variable names referenced inside the region but declared
   * outside all regions, `this`, or `this.#name` (see module doc)
   */
  refs: string[];
}

// Cap on number of injected refs per region, to bound the work Rust does fitting them
// into the placeholder. Real-world templates reference far fewer variables than this.
const MAX_REFS_PER_REGION = 64;

// Valid JS identifier that can be injected into a placeholder without breaking its syntax.
// Restricted to ASCII so the injected bytes can never exceed the region's byte length
// budget in surprising ways, and conservatively excludes reserved words that are valid
// binding names only in sloppy mode (the shadow source is always parsed as a module).
const INJECTABLE_IDENT_REGEX = /^[A-Za-z_$][A-Za-z0-9_$]*$/;
const NON_INJECTABLE_NAMES = new Set(["arguments", "eval", "yield", "await", "let", "static"]);

/**
 * Set `parent` on every node of an AST produced by a custom parser, and find the
 * AST's masked regions.
 *
 * Must be called BEFORE running rules. ESLint pre-computes its traversal steps, so by
 * the time any rule listener runs, every node in the AST has `parent` set - including
 * nodes which come later in the file than the node being visited. Rules rely on this
 * (e.g. `ember/template-no-let-reference` reads `identifier.parent.parent` on the
 * declaration of a variable which may be declared after the template referencing it).
 * Setting all parents up front matches that behavior; the rule walk's own `parent`
 * assignment then just re-assigns the same values.
 *
 * A node whose type is not in Oxc's own visitor keys table (i.e. cannot be represented
 * in Oxc's AST) becomes a masked region. Its children are still descended into (they
 * need `parent` set too), but nested custom nodes are not recorded - they are already
 * covered by the top-most region.
 *
 * Returns `null` for the regions if they cannot be reliably determined (a custom node
 * has an invalid or overlapping range) - Rust then skips native linting for the file,
 * preserving the previous behavior (only JS plugin rules run). Parents are set
 * regardless.
 *
 * @param program - `Program` AST node produced by the parser
 * @param visitorKeys - Merged visitor keys for the file (parser keys over defaults)
 * @param scopeManager - Scope manager from parser output, or `null`
 * @param sourceTextLength - Length of source text in UTF-16 code units
 * @returns Masked regions sorted by start offset, or `null` if they cannot be determined
 */
export function setParentsAndGetMaskedRegions(
  program: JsParserProgram,
  visitorKeys: Record<string, readonly string[]>,
  scopeManager: JsParserScopeManager | null,
  sourceTextLength: number,
): MaskedRegionReport[] | null {
  const regions: MaskedRegionReport[] = [];
  let valid = true;

  // Usage detected inside each region which `collectRefs` cannot see (`this` and
  // private names are not scope-manager references), plus the region's root node -
  // its parent chain determines what is injectable at the region's position
  const detections: {
    region: MaskedRegionReport;
    root: JsParserNode;
    usesThis: boolean;
    privateNames: Set<string>;
  }[] = [];

  function visitNode(node: JsParserNode, parent: JsParserNode | null, insideCustom: boolean): void {
    // Set `parent` on the node, like ESLint does (plain enumerable assignment)
    node.parent = parent;

    let isCustom = false;
    if (!Object.hasOwn(oxcVisitorKeys, node.type)) {
      isCustom = true;
      // Top-most custom node - record its span as a masked region.
      // `range` should be present (`range: true` is forced in parser options), with
      // `[node.start, node.end]` as fallback for parsers that ignore the option.
      if (!insideCustom && valid) {
        const { range } = node;
        const start = Array.isArray(range) ? range[0] : (node.start as unknown);
        const end = Array.isArray(range) ? range[1] : (node.end as unknown);
        if (
          !Number.isInteger(start) ||
          !Number.isInteger(end) ||
          (start as number) < 0 ||
          (end as number) <= (start as number) ||
          (end as number) > sourceTextLength
        ) {
          valid = false;
        } else {
          const region: MaskedRegionReport = {
            start: start as number,
            end: end as number,
            classMember: parent !== null && parent.type === "ClassBody",
            refs: [],
          };
          regions.push(region);
          detections.push(detectInjectables(node, region));
        }
      }
    }

    const keys = visitorKeys[node.type] ?? getFallbackKeys(node);
    for (let i = 0, keysLen = keys.length; i < keysLen; i++) {
      const child = node[keys[i]];
      if (Array.isArray(child)) {
        for (let j = 0, childLen = child.length; j < childLen; j++) {
          const element: unknown = child[j];
          if (isNode(element)) visitNode(element, node, insideCustom || isCustom);
        }
      } else if (isNode(child)) {
        visitNode(child, node, insideCustom || isCustom);
      }
    }
  }

  visitNode(program, null, false);

  if (!valid) return null;

  // Top-most custom nodes are visited in source order for well-formed ASTs, but a
  // misbehaving parser could produce out-of-order or overlapping sibling ranges.
  // Sort, then reject overlaps - the shadow source cannot be built from them.
  regions.sort((a, b) => a.start - b.start);
  for (let i = 1; i < regions.length; i++) {
    if (regions[i].start < regions[i - 1].end) return null;
  }

  // Inject detected `this` / private-name usage. Runs before `collectRefs` so these
  // deterministic usage facts get placeholder space first. The region root's parent
  // chain is fully set by now, and every ancestor of a root is outside all regions
  // (an ancestor inside another region would make that region overlap this one).
  for (const { region, root, usesThis, privateNames } of detections) {
    if (usesThis && hasThisContext(root)) region.refs.push("this");
    for (const name of privateNames) {
      // Restrict to ASCII identifiers, same as `collectRefs` (see INJECTABLE_IDENT_REGEX)
      if (!INJECTABLE_IDENT_REGEX.test(name)) continue;
      if (region.refs.length >= MAX_REFS_PER_REGION) break;
      // Only inject if a class enclosing the region declares `#name` - otherwise the
      // shadow source would reference a private name that doesn't exist in it, which
      // is a parse error (killing native linting for the whole file).
      if (!enclosingClassDeclaresPrivate(root, name)) continue;
      region.refs.push(`this.#${name}`);
    }
  }

  if (regions.length > 0 && scopeManager !== null) collectRefs(regions, scopeManager);

  return regions;
}

/**
 * Walk a masked region's subtree detecting usage of `this` and private names.
 *
 * This walk is separate from the parent-setting walk because it must see MORE of the
 * tree: the parent-setting walk honors the parser's visitor keys (matching ESLint's
 * traversal), but parsers often give custom nodes visitor keys that don't cover all
 * their children (e.g. `ember-eslint-parser` declares `GlimmerPathExpression: []`,
 * hiding the `ThisHead` node in its `head` property). Here every node-valued own
 * property is descended into ({@link getFallbackKeys}), with a visited set guarding
 * against cyclic references. `parent` is NOT set here - only the ESLint-equivalent
 * traversal above assigns parents.
 *
 * Detected:
 * - `ThisExpression` (standard ESTree, for parsers embedding real JS expressions) and
 *   `ThisHead` (the Glimmer AST head of `{{this.foo}}` paths). Not counted inside
 *   nested non-arrow functions - their `this` is not the region's `this`.
 * - `PrivateIdentifier` (standard ESTree). Glimmer's `{{this.#x}}` deliberately does
 *   NOT count: it puts `"#x"` in `GlimmerPathExpression.tail` as a plain string,
 *   because Glimmer resolves it as a property named `"#x"`, not a private field.
 *
 * @param root - The region's root node (the top-most custom node)
 * @param region - The masked region
 * @returns Detection result for the region
 */
function detectInjectables(
  root: JsParserNode,
  region: MaskedRegionReport,
): {
  region: MaskedRegionReport;
  root: JsParserNode;
  usesThis: boolean;
  privateNames: Set<string>;
} {
  let usesThis = false;
  const privateNames = new Set<string>();
  const visited = new Set<JsParserNode>();

  function walk(node: JsParserNode, thisCountable: boolean): void {
    if (visited.has(node)) return;
    visited.add(node);

    const { type } = node;
    if (thisCountable && (type === "ThisExpression" || type === "ThisHead")) {
      usesThis = true;
    } else if (type === "PrivateIdentifier") {
      const name = (node as { name?: unknown }).name;
      if (typeof name === "string" && privateNames.size < MAX_REFS_PER_REGION) {
        privateNames.add(name);
      }
    }

    // A nested non-arrow function has its own `this`; private access inside it still
    // counts (a private name resolves lexically, across function boundaries)
    const countable =
      thisCountable && type !== "FunctionExpression" && type !== "FunctionDeclaration";

    const keys = getFallbackKeys(node);
    for (let i = 0, keysLen = keys.length; i < keysLen; i++) {
      const child = node[keys[i]];
      if (Array.isArray(child)) {
        for (let j = 0, childLen = child.length; j < childLen; j++) {
          const element: unknown = child[j];
          if (isNode(element)) walk(element, countable);
        }
      } else if (isNode(child)) {
        walk(child, countable);
      }
    }
  }

  walk(root, true);

  return { region, root, usesThis, privateNames };
}

/**
 * Check if `this` is lexically valid at a region's position: some ancestor is a
 * non-arrow function or a class body (class fields, static blocks and methods all
 * have a `this` context; arrow functions inherit one, so the walk continues
 * through them). At the top level of a module - where the shadow source is always
 * parsed - `this` would trip rules like `no-invalid-this`, so it is not injected.
 *
 * @param root - The region's root node (with `parent` set on it and all ancestors)
 * @returns `true` if `this` is valid at the region's position
 */
function hasThisContext(root: JsParserNode): boolean {
  for (let cur = root.parent; cur != null; cur = cur.parent) {
    const { type } = cur;
    if (type === "FunctionDeclaration" || type === "FunctionExpression" || type === "ClassBody") {
      return true;
    }
  }
  return false;
}

/**
 * Check if a class enclosing a region declares private name `#name`. Ancestors of a
 * region root are always outside all masked regions, so a declaration found here
 * exists in the shadow source too, and an enclosing class body also guarantees a
 * `this.#name` expression is valid at the region's position.
 *
 * @param root - The region's root node (with `parent` set on it and all ancestors)
 * @param name - The private name, without `#`
 * @returns `true` if an enclosing class declares `#name`
 */
function enclosingClassDeclaresPrivate(root: JsParserNode, name: string): boolean {
  for (let cur = root.parent; cur != null; cur = cur.parent) {
    if (cur.type !== "ClassBody") continue;
    const body = (cur as { body?: unknown }).body;
    if (!Array.isArray(body)) continue;
    for (const element of body) {
      if (!isNode(element)) continue;
      const key = (element as { key?: unknown }).key;
      if (
        isNode(key) &&
        key.type === "PrivateIdentifier" &&
        (key as { name?: unknown }).name === name
      ) {
        return true;
      }
    }
  }
  return false;
}

/**
 * Collect names of variables referenced inside masked regions but declared outside
 * all of them, into each region's `refs`.
 *
 * Uses the parser's scope manager - parsers that support custom syntax register
 * references for identifiers inside it (e.g. `ember-eslint-parser` registers `{{foo}}`
 * in a `<template>` as a reference to `foo`). Without a parser-provided scope manager
 * there is nothing to collect: a scope manager built from the AST alone cannot see
 * into custom nodes.
 *
 * @param regions - Masked regions, sorted by start offset
 * @param scopeManager - Scope manager from parser output
 */
function collectRefs(regions: MaskedRegionReport[], scopeManager: JsParserScopeManager): void {
  const { scopes } = scopeManager;
  if (!Array.isArray(scopes)) return;

  // Refs already added, per region (regions are few, so parallel array of sets)
  const seenNames: (Set<string> | null)[] = regions.map(() => null);

  for (const scope of scopes) {
    const references = scope?.references;
    if (!Array.isArray(references)) continue;

    for (const reference of references) {
      const { resolved } = reference;
      if (resolved == null) continue;

      const identifier = reference.identifier as unknown as JsParserNode | null;
      const identifierStart = getRangeStart(identifier);
      if (identifierStart === null) continue;

      const regionIndex = findRegionContaining(regions, identifierStart);
      if (regionIndex === -1) continue;

      const { name } = resolved;
      if (
        typeof name !== "string" ||
        !INJECTABLE_IDENT_REGEX.test(name) ||
        NON_INJECTABLE_NAMES.has(name)
      ) {
        continue;
      }

      const region = regions[regionIndex];
      let seen = seenNames[regionIndex];
      if (seen === null) seen = seenNames[regionIndex] = new Set();
      if (seen.has(name) || seen.size >= MAX_REFS_PER_REGION) continue;

      // Only inject variables declared outside all masked regions. A variable declared
      // inside a region (e.g. an Ember template block param `as |item|`) does not exist
      // in the shadow source, so injecting a reference to it would create a false
      // `no-undef` error in native linting.
      if (!isDeclaredOutsideRegions(resolved.identifiers, regions)) continue;

      seen.add(name);
      region.refs.push(name);
    }
  }
}

/**
 * Check if a variable has at least one declaration site outside all masked regions.
 * @param identifiers - The variable's declaration identifiers
 * @param regions - Masked regions, sorted by start offset
 * @returns `true` if at least one declaration is outside all regions
 */
function isDeclaredOutsideRegions(identifiers: unknown, regions: MaskedRegionReport[]): boolean {
  if (!Array.isArray(identifiers)) return false;
  for (const identifier of identifiers) {
    const start = getRangeStart(identifier as JsParserNode | null);
    if (start !== null && findRegionContaining(regions, start) === -1) return true;
  }
  return false;
}

/**
 * Find the masked region containing an offset.
 * @param regions - Masked regions, sorted by start offset
 * @param offset - UTF-16 offset
 * @returns Index of the region containing `offset`, or `-1` if none does
 */
function findRegionContaining(regions: MaskedRegionReport[], offset: number): number {
  // Regions are few (one per custom syntax block), so linear scan is fine
  for (let i = 0; i < regions.length; i++) {
    const region = regions[i];
    if (offset >= region.start && offset < region.end) return i;
    if (region.start > offset) break;
  }
  return -1;
}

/**
 * Get the start offset of a node's range, or `null` if it doesn't have a valid one.
 * @param node - AST node, or `null` / `undefined`
 * @returns Start offset, or `null`
 */
function getRangeStart(node: JsParserNode | null | undefined): number | null {
  if (node === null || node === undefined || typeof node !== "object") return null;
  const { range } = node;
  const start = Array.isArray(range) ? range[0] : (node.start as unknown);
  return Number.isInteger(start) && (start as number) >= 0 ? (start as number) : null;
}

/**
 * Check if a value is an AST node (object with a string `type` property).
 * Same check as `js_ast_walk.ts` uses.
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
