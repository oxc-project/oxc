/*
 * `SourceCode` methods related to scopes.
 */

import { analyze, Variable as TSVariable } from "@typescript-eslint/scope-manager";
import { ecmaFeaturesOverride } from "./context.ts";
import { ast, initAst } from "./source_code.ts";
import { globals, envs, initGlobals } from "./globals.ts";
import { ENVS } from "../generated/envs.ts";
import { debugAssert, debugAssertIsNonNull } from "../utils/asserts.ts";

import type {
  AnalyzeOptions,
  ScopeManager as TSESLintScopeManager,
  Scope as TSScope,
} from "@typescript-eslint/scope-manager";
import type { Writable } from "type-fest";
import type * as ESTree from "../generated/types.d.ts";

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
const analyzeOptions: AnalyzeOptions = {
  childVisitorKeys: undefined,
  globalReturn: false,
  impliedStrict: false,
  jsxPragma: "React",
  jsxFragmentName: null,
  lib: [],
  sourceType: "module",
  emitDecoratorMetadata: false,
};

/**
 * Initialize TS-ESLint `ScopeManager` for current file.
 */
function initTsScopeManager() {
  if (ast === null) initAst();
  debugAssertIsNonNull(ast);

  const { sourceType } = ast;
  analyzeOptions.sourceType = sourceType;
  analyzeOptions.globalReturn = sourceType === "commonjs";
  analyzeOptions.impliedStrict = sourceType === "module";

  // Override `globalReturn` and `impliedStrict` if in conformance build
  if (CONFORMANCE) {
    const { globalReturn, impliedStrict } = ecmaFeaturesOverride;
    if (globalReturn !== null) analyzeOptions.globalReturn = globalReturn;
    if (impliedStrict !== null) analyzeOptions.impliedStrict = impliedStrict;
  }

  // @ts-expect-error - TODO: Our types don't quite align yet
  tsScopeManager = analyze(ast, analyzeOptions);

  fixCatchClauseDefinitions();

  // Add globals from configuration and resolve references
  addGlobals();
}

/**
 * Fix `CatchClause` definitions to match `eslint-scope` behavior.
 *
 * TS-ESLint's scope manager has a bug where for destructuring patterns in `CatchClause`s
 * (e.g., `catch ([a, b])` or `catch ({ message })`), the definition's `name` property is set to
 * the entire pattern (`ArrayPattern` or `ObjectPattern`) instead of the individual `Identifier`.
 *
 * Correct this bug by setting `def.name` to the `Identifier`.
 *
 * @see https://github.com/typescript-eslint/typescript-eslint/issues/11981
 */
function fixCatchClauseDefinitions(): void {
  debugAssertIsNonNull(tsScopeManager);

  const { scopes } = tsScopeManager;
  for (let scopeIndex = 0; scopeIndex < scopes.length; scopeIndex++) {
    const scope = scopes[scopeIndex];
    if (scope.type !== "catch") continue;

    const { param } = scope.block;
    if (param === null || param.type === "Identifier") continue;

    // `CatchClause` scope with an `ObjectPattern` or `ArrayPattern` parameter - fix it
    const { variables } = scope;
    for (let varIndex = 0; varIndex < variables.length; varIndex++) {
      const variable = variables[varIndex];

      // Variables defined in a catch clause are block-scoped, therefore can only have a single definition.
      // If there were more, parser would have errored already.
      debugAssert(variable.defs.length === 1, "Expected `defs.length === 1`");
      debugAssert(variable.identifiers.length === 1, "Expected `identifiers.length === 1`");
      debugAssert(variable.defs[0].name === param, "Expected `def.name === param`");

      (variable.defs[0] as Writable<(typeof variable.defs)[number]>).name = variable.identifiers[0];
    }
  }
}

/**
 * Add global variables from configuration and resolve references to them.
 *
 * With `lib: []`, no lib globals are created during analysis, so all global references
 * end up in `globalScope.through`. This function creates Variables for each configured
 * global and resolves references from `through`.
 *
 * This replicates ESLint's `scopeManager.addGlobals()` behavior.
 */
function addGlobals(): void {
  debugAssertIsNonNull(tsScopeManager);
  const globalScope = tsScopeManager.scopes[0];

  // Ensure globals are initialized
  if (globals === null) initGlobals();
  debugAssertIsNonNull(globals);
  debugAssertIsNonNull(envs);

  // Create variables for enabled `envs`.
  // All properties of `envs` are `true`, so no need to check the values.
  // `envs` from JSON, so we can use simple `for..in` loop.
  for (const envName in envs) {
    // Get vars defined for this env.
    // Rust side code ignores invalid env names, and passes them through to JS,
    // so we can't assume they're valid here. But debug assert they're valid for tests.
    const preset = ENVS.get(envName);
    debugAssertIsNonNull(preset, `Unknown env: ${envName}`);
    if (preset === undefined) continue;

    const { readonly, writable } = preset;

    for (let i = 0, len = readonly.length; i < len; i++) {
      const varName = readonly[i];
      // Skip vars that are defined in `globals`. They might be `"off"`.
      if (!Object.hasOwn(globals, varName)) createGlobalVariable(varName, globalScope, false);
    }

    for (let i = 0, len = writable.length; i < len; i++) {
      const varName = writable[i];
      // Skip vars that are defined in `globals`. They might be `"off"`.
      if (!Object.hasOwn(globals, varName)) createGlobalVariable(varName, globalScope, true);
    }
  }

  // Create variables for enabled `globals`.
  // `globals` from JSON, so we can use simple `for..in` loop.
  for (const name in globals) {
    const value = globals[name];
    if (value !== "off") createGlobalVariable(name, globalScope, value === "writable");
  }

  // Resolve references from `through`
  (globalScope as Writable<typeof globalScope>).through = globalScope.through.filter((ref) => {
    const { name } = ref.identifier;
    const variable = globalScope.set.get(name);
    if (!variable) return true; // Keep in `through` (truly undefined)

    // Resolve the reference, remove from `through`
    ref.resolved = variable;
    variable.references.push(ref);
    return false;
  });

  // Clean up implicit globals.
  // "implicit" contains information about implicit global variables (those created
  // implicitly by assigning values to undeclared variables in non-strict code).
  // Since we augment the global scope, we need to remove the ones that match declared globals.
  // This matches eslint-scope's `__addVariables` behavior.
  // @ts-expect-error - `implicit` is private but accessible at runtime
  const { implicit } = globalScope;
  implicit.variables = implicit.variables.filter((variable: Variable) => {
    const { name } = variable;
    if (globalScope.set.has(name)) {
      implicit.set.delete(name);
      return false;
    }
    return true;
  });
  // typescript-eslint uses `leftToBeResolved`, eslint-scope uses `left`
  implicit.leftToBeResolved = implicit.leftToBeResolved.filter(
    (ref: Reference) => !globalScope.set.has(ref.identifier.name),
  );
}

/**
 * Create a global variable with the given name and add it to the global scope.
 * @param name - Var name
 * @param globalScope - Global scope object
 * @param isWritable - `true` if the variable is writable, `false` otherwise
 */
function createGlobalVariable(name: string, globalScope: TSScope, isWritable: boolean): void {
  // If var already exists in the scope (from code declarations or previous envs), don't create a new one.
  // TS-ESLint's scope manager doesn't resolve references in the global scope for `sourceType: "script"`,
  // so we mustn't overwrite local `var` declarations with globals of the same name.
  // But set properties on the existing variable to indicate it's a configured global.
  let variable = globalScope.set.get(name);
  if (variable === undefined) {
    variable = new TSVariable(name, globalScope);
    globalScope.set.set(name, variable);
    globalScope.variables.push(variable);

    // All globals are type + value
    debugAssert(variable.isTypeVariable, "variable should have `isTypeVariable` set by default");
    debugAssert(variable.isValueVariable, "variable should have `isValueVariable` set by default");
  }

  // @ts-expect-error - not present in types
  variable.writeable = isWritable;
  // @ts-expect-error - not present in types
  variable.eslintImplicitGlobalSetting = isWritable ? "writable" : "readonly";
  // @ts-expect-error - not present in types
  variable.eslintExplicitGlobal = false;
  // @ts-expect-error - not present in types
  variable.eslintExplicitGlobalComments = undefined;
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
