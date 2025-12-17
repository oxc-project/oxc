/*
 * `SourceCode` methods related to scopes.
 */

import {
  analyze,
  type AnalyzeOptions,
  type ScopeManager as TSESLintScopeManager,
} from "@typescript-eslint/scope-manager";
import { ast, initAst } from "./source_code.ts";
import { typeAssertIs, debugAssertIsNonNull } from "../utils/asserts.ts";

import type * as ESTree from "../generated/types.d.ts";
import type { SetNullable } from "../utils/types.ts";

export interface Scope {
  type: ScopeType;
  isStrict: boolean;
  upper: Scope | null;
  childScopes: Scope[];
  variableScope: Scope;
  block: ESTree.Node;
  variables: Variable[];
  set: Map<string, Variable>;
  references: Reference[];
  through: Reference[];
  functionExpressionScope: boolean;
  implicit?: {
    variables: Variable[];
    set: Map<string, Variable>;
  };
}

export type ScopeType =
  | "block"
  | "catch"
  | "class"
  | "class-field-initializer"
  | "class-static-block"
  | "for"
  | "function"
  | "function-expression-name"
  | "global"
  | "module"
  | "switch"
  | "with";

export interface Variable {
  name: string;
  scope: Scope;
  identifiers: Identifier[];
  references: Reference[];
  defs: Definition[];
}

export interface Reference {
  identifier: Identifier;
  from: Scope;
  resolved: Variable | null;
  writeExpr: ESTree.Expression | null;
  init: boolean;
  isWrite(): boolean;
  isRead(): boolean;
  isReadOnly(): boolean;
  isWriteOnly(): boolean;
  isReadWrite(): boolean;
}

export interface Definition {
  type: DefinitionType;
  name: Identifier;
  node: ESTree.Node;
  parent: ESTree.Node | null;
}

export type DefinitionType =
  | "CatchClause"
  | "ClassName"
  | "FunctionName"
  | "ImplicitGlobalVariable"
  | "ImportBinding"
  | "Parameter"
  | "Variable";

type Identifier =
  | ESTree.IdentifierName
  | ESTree.IdentifierReference
  | ESTree.BindingIdentifier
  | ESTree.LabelIdentifier
  | ESTree.TSThisParameter
  | ESTree.TSIndexSignatureName;

// TS-ESLint `ScopeManager` for current file.
// Created lazily only when needed.
let tsScopeManager: TSESLintScopeManager | null = null;

// Options for TS-ESLint's `analyze` method.
// `sourceType` property is set before calling `analyze`.
const analyzeOptions: SetNullable<AnalyzeOptions, "sourceType"> = {
  globalReturn: false,
  jsxFragmentName: null,
  jsxPragma: "React",
  lib: ["esnext"],
  sourceType: null,
};

/**
 * Initialize TS-ESLint `ScopeManager` for current file.
 */
function initTsScopeManager() {
  if (ast === null) initAst();
  debugAssertIsNonNull(ast);

  analyzeOptions.sourceType = ast.sourceType;
  typeAssertIs<AnalyzeOptions>(analyzeOptions);
  // The effectiveness of this assertion depends on our alignment with ESTree.
  // It could eventually be removed as we align the remaining corner cases and the typegen.
  // @ts-expect-error - TODO: Our types don't quite align yet
  tsScopeManager = analyze(ast, analyzeOptions);
}

/**
 * Discard TS-ESLint `ScopeManager`, to free memory.
 */
export function resetScopeManager() {
  tsScopeManager = null;
}

/**
 * @see https://eslint.org/docs/latest/developer-guide/scope-manager-interface#scopemanager-interface
 */
// This is a wrapper around `@typescript-eslint/scope-manager` package's `ScopeManager` class.
// We want to control what APIs are exposed to the user to limit breaking changes when we switch our implementation.
//
// Only one file is linted at a time, so we can reuse a single object for all files.
//
// This has advantages:
// 1. Reduce object creation.
// 2. Property accesses don't need to go up prototype chain, as they would for instances of a class.
// 3. No need for private properties, which are somewhat expensive to access - use top-level variables instead.
//
// Freeze the object to prevent user mutating it.
export const SCOPE_MANAGER = Object.freeze({
  /**
   * All scopes.
   */
  get scopes(): Scope[] {
    if (tsScopeManager === null) initTsScopeManager();
    // @ts-expect-error - TODO: Our types don't quite align yet
    return tsScopeManager.scopes;
  },

  /**
   * The root scope.
   */
  get globalScope(): Scope | null {
    if (tsScopeManager === null) initTsScopeManager();
    // @ts-expect-error - TODO: Our types don't quite align yet
    return tsScopeManager.globalScope;
  },

  /**
   * Get the variables that a given AST node defines.
   * The returned variables' `def[].node` / `def[].parent` property is the node.
   * If the node does not define any variable, this returns an empty array.
   * @param node AST node to get variables of.
   */
  getDeclaredVariables(node: ESTree.Node): Variable[] {
    if (tsScopeManager === null) initTsScopeManager();
    // @ts-expect-error - TODO: Our types don't quite align yet
    return tsScopeManager.getDeclaredVariables(node);
  },

  /**
   * Get the scope of a given AST node. The returned scope's `block` property is the node.
   * This method never returns `function-expression-name` scope.
   * If the node does not have a scope, returns `null`.
   *
   * @param node An AST node to get their scope.
   * @param inner If the node has multiple scopes, this returns the outermost scope normally.
   *   If `inner` is `true` then this returns the innermost scope.
   */
  acquire(node: ESTree.Node, inner?: boolean): Scope | null {
    if (tsScopeManager === null) initTsScopeManager();
    // @ts-expect-error - TODO: Our types don't quite align yet
    return tsScopeManager.acquire(node, inner);
  },
});

export type ScopeManager = typeof SCOPE_MANAGER;

/**
 * Determine whether the given identifier node is a reference to a global variable.
 * @param node - `Identifier` node to check.
 * @returns `true` if the identifier is a reference to a global variable.
 */
export function isGlobalReference(node: ESTree.Node): boolean {
  // ref: https://github.com/eslint/eslint/blob/e7cda3bdf1bdd664e6033503a3315ad81736b200/lib/languages/js/source-code/source-code.js#L934-L962
  if (!node) throw new TypeError("Missing required argument: `node`");
  if (node.type !== "Identifier") return false;

  if (tsScopeManager === null) initTsScopeManager();
  debugAssertIsNonNull(tsScopeManager);

  const { scopes } = tsScopeManager;
  if (scopes.length === 0) return false;
  const globalScope = scopes[0];

  // If the identifier is a reference to a global variable, the global scope should have a variable with the name
  const variable = globalScope.set.get(node.name);

  // Global variables are not defined by any node, so they should have no definitions
  if (variable === undefined || variable.defs.length > 0) return false;

  // If there is a variable by the same name exists in the global scope,
  // we need to check our node is one of its references
  const { references } = variable;
  for (let i = 0, len = references.length; i < len; i++) {
    if (references[i].identifier === node) return true;
  }

  return false;
}

/**
 * Get the variables that `node` defines.
 * This is a convenience method that passes through to the same method on the `ScopeManager`.
 * @param node - The node for which the variables are obtained.
 * @returns An array of variable nodes representing the variables that `node` defines.
 */
export function getDeclaredVariables(node: ESTree.Node): Variable[] {
  // ref: https://github.com/eslint/eslint/blob/e7cda3bdf1bdd664e6033503a3315ad81736b200/lib/languages/js/source-code/source-code.js#L904
  if (tsScopeManager === null) initTsScopeManager();
  debugAssertIsNonNull(tsScopeManager);

  // @ts-expect-error - TODO: Our types don't quite align yet
  return tsScopeManager.getDeclaredVariables(node);
}

/**
 * Get the scope for the given node.
 * @param node - The node to get the scope of.
 * @returns The scope information for this node.
 */
export function getScope(node: ESTree.Node): Scope {
  // ref: https://github.com/eslint/eslint/blob/e7cda3bdf1bdd664e6033503a3315ad81736b200/lib/languages/js/source-code/source-code.js#L862-L892
  if (!node) throw new TypeError("Missing required argument: `node`");

  if (tsScopeManager === null) initTsScopeManager();
  debugAssertIsNonNull(tsScopeManager);

  const inner = node.type !== "Program";

  // Traverse up the AST to find a `Node` whose scope can be acquired.
  do {
    // @ts-expect-error - TODO: Our types don't quite align yet
    const scope = tsScopeManager.acquire(node, inner) as Scope;
    if (scope !== null) {
      return scope.type === "function-expression-name" ? scope.childScopes[0] : scope;
    }

    // @ts-expect-error - Don't want to create a new variable just to make it nullable
    node = node.parent;
  } while (node !== null);

  // TODO: Is it possible to get here? Doesn't `Program` always have a scope?
  // @ts-expect-error - TODO: Our types don't quite align yet
  return tsScopeManager.scopes[0];
}

/**
 * Marks as used a variable with the given name in a scope indicated by the given reference node.
 * This affects the `no-unused-vars` rule.
 * @param name - Variable name
 * @param refNode - Reference node
 * @returns `true` if a variable with the given name was found and marked as used, otherwise `false`
 */
/* oxlint-disable no-unused-vars */
export function markVariableAsUsed(name: string, refNode: ESTree.Node): boolean {
  // TODO: Implement
  throw new Error("`context.markVariableAsUsed` not implemented yet");
}
/* oxlint-enable no-unused-vars */
