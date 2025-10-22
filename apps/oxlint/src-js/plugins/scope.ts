/*
 * `SourceCode` methods related to scopes.
 */

import type * as ESTree from '../generated/types.d.ts';

import {
  analyze,
  type AnalyzeOptions,
  type ScopeManager as TSESLintScopeManager,
} from '@typescript-eslint/scope-manager';
import { SOURCE_CODE } from './source_code.js';

type Identifier =
  | ESTree.IdentifierName
  | ESTree.IdentifierReference
  | ESTree.BindingIdentifier
  | ESTree.LabelIdentifier
  | ESTree.TSThisParameter
  | ESTree.TSIndexSignatureName;

/**
 * @see https://eslint.org/docs/latest/developer-guide/scope-manager-interface#scopemanager-interface
 */
// This is a wrapper class around the @typescript-eslint/scope-manager package.
// We want to control what APIs are exposed to the user to limit breaking changes when we switch our implementation.
export class ScopeManager {
  #scopeManager: TSESLintScopeManager;

  constructor(ast: ESTree.Program) {
    const defaultOptions: AnalyzeOptions = {
      globalReturn: false,
      jsxFragmentName: null,
      jsxPragma: 'React',
      lib: ['esnext'],
      sourceType: ast.sourceType,
    };
    // The effectiveness of this assertion depends on our alignment with ESTree.
    // It could eventually be removed as we align the remaining corner cases and the typegen.
    this.#scopeManager = analyze(ast as any, defaultOptions);
  }

  /**
   * All scopes
   */
  get scopes(): Scope[] {
    return this.#scopeManager.scopes as any;
  }

  /**
   * The root scope
   */
  get globalScope(): Scope | null {
    return this.#scopeManager.globalScope as any;
  }

  /**
   * Get the variables that a given AST node defines. The gotten variables' `def[].node`/`def[].parent` property is the node.
   * If the node does not define any variable, this returns an empty array.
   * @param node An AST node to get their variables.
   */
  getDeclaredVariables(node: ESTree.Node): Variable[] {
    return this.#scopeManager.getDeclaredVariables(node as any) as any;
  }

  /**
   * Get the scope of a given AST node. The gotten scope's `block` property is the node.
   * This method never returns `function-expression-name` scope. If the node does not have their scope, this returns `null`.
   *
   * @param node An AST node to get their scope.
   * @param inner If the node has multiple scopes, this returns the outermost scope normally.
   *                If `inner` is `true` then this returns the innermost scope.
   */
  acquire(node: ESTree.Node, inner?: boolean): Scope | null {
    return this.#scopeManager.acquire(node as any, inner) as any;
  }
}

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
  | 'block'
  | 'catch'
  | 'class'
  | 'class-field-initializer'
  | 'class-static-block'
  | 'for'
  | 'function'
  | 'function-expression-name'
  | 'global'
  | 'module'
  | 'switch'
  | 'with';

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
  | 'CatchClause'
  | 'ClassName'
  | 'FunctionName'
  | 'ImplicitGlobalVariable'
  | 'ImportBinding'
  | 'Parameter'
  | 'Variable';

/**
 * Determine whether the given identifier node is a reference to a global variable.
 * @param node - `Identifier` node to check.
 * @returns `true` if the identifier is a reference to a global variable.
 */
export function isGlobalReference(node: ESTree.Node): boolean {
  // ref: https://github.com/eslint/eslint/blob/e7cda3bdf1bdd664e6033503a3315ad81736b200/lib/languages/js/source-code/source-code.js#L934-L962
  if (!node) {
    throw new TypeError('Missing required argument: node.');
  }

  if (node.type !== 'Identifier') {
    return false;
  }

  const { name } = node;
  if (typeof name !== 'string') {
    return false;
  }

  // If the identifier is a reference to a global variable, the global scope should have a variable with the name.
  const variable = SOURCE_CODE.scopeManager.globalScope.set.get(name);

  // Global variables are not defined by any node, so they should have no definitions.
  if (!variable || variable.defs.length > 0) {
    return false;
  }

  // If there is a variable by the same name exists in the global scope, we need to check our node is one of its references.
  const { references } = variable;

  for (let i = 0; i < references.length; i++) {
    const reference = references[i];
    if (reference.identifier === node) {
      return true;
    }
  }

  return false;
}

/**
 * Get the variables that `node` defines.
 * This is a convenience method that passes through to the same method on the `scopeManager`.
 * @param node - The node for which the variables are obtained.
 * @returns An array of variable nodes representing the variables that `node` defines.
 */
export function getDeclaredVariables(node: ESTree.Node): Variable[] {
  // ref: https://github.com/eslint/eslint/blob/e7cda3bdf1bdd664e6033503a3315ad81736b200/lib/languages/js/source-code/source-code.js#L904
  return SOURCE_CODE.scopeManager.getDeclaredVariables(node);
}

/**
 * Get the scope for the given node
 * @param node - The node to get the scope of.
 * @returns The scope information for this node.
 */
export function getScope(node: ESTree.Node): Scope {
  // ref: https://github.com/eslint/eslint/blob/e7cda3bdf1bdd664e6033503a3315ad81736b200/lib/languages/js/source-code/source-code.js#L862-L892
  if (!node) {
    throw new TypeError('Missing required argument: node.');
  }

  const { scopeManager } = SOURCE_CODE;
  const inner = node.type !== 'Program';

  // Traverse up the AST to find a `Node` whose scope can be acquired.
  for (let current: any = node; current; current = current.parent) {
    const scope = scopeManager.acquire(current, inner);

    if (scope) {
      if (scope.type === 'function-expression-name') {
        return scope.childScopes[0];
      }

      return scope;
    }
  }

  return scopeManager.scopes[0];
}
