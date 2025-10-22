/*
 * `SourceCode` methods related to scopes.
 */

import type * as ESTree from '../generated/types.d.ts';
import type { Program } from '../generated/types.d.ts';

import {
  analyze,
  type AnalyzeOptions,
  GlobalScope,
  type ScopeManager as TSESLintScopeManager,
} from '@typescript-eslint/scope-manager';
import type { Node } from './types.ts';

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

  constructor(ast: Program) {
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
  get globalScope(): GlobalScope | null {
    return this.#scopeManager.globalScope;
  }

  /**
   * Get the variables that a given AST node defines. The gotten variables' `def[].node`/`def[].parent` property is the node.
   * Get the variables that a given AST node defines. The gotten variables' `def[].node`/`def[].parent` property is the node.
   * If the node does not define any variable, this returns an empty array.
   * @param node An AST node to get their variables.
   */
  getDeclaredVariables(node: Node): Variable[] {
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
  acquire(node: Node, inner?: boolean): Scope | null {
    return this.#scopeManager.acquire(node as any, inner) as any;
  }
}

export interface Scope {
  type: ScopeType;
  isStrict: boolean;
  upper: Scope | null;
  childScopes: Scope[];
  variableScope: Scope;
  block: Node;
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
  node: Node;
  parent: Node | null;
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
// oxlint-disable-next-line no-unused-vars
export function isGlobalReference(node: Node): boolean {
  throw new Error('`sourceCode.isGlobalReference` not implemented yet'); // TODO
}

/**
 * Get the variables that `node` defines.
 * This is a convenience method that passes through to the same method on the `scopeManager`.
 * @param node - The node for which the variables are obtained.
 * @returns An array of variable nodes representing the variables that `node` defines.
 */
// oxlint-disable-next-line no-unused-vars
export function getDeclaredVariables(node: Node): Variable[] {
  throw new Error('`sourceCode.getDeclaredVariables` not implemented yet'); // TODO
}

/**
 * Get the scope for the given node
 * @param node - The node to get the scope of.
 * @returns The scope information for this node.
 */
// oxlint-disable-next-line no-unused-vars
export function getScope(node: Node): Scope {
  throw new Error('`sourceCode.getScope` not implemented yet'); // TODO
}

/**
 * Mark a variable as used in the current scope
 * @param name - The name of the variable to mark as used.
 * @param refNode? - The closest node to the variable reference.
 * @returns `true` if the variable was found and marked as used, `false` if not.
 */
// oxlint-disable-next-line no-unused-vars
export function markVariableAsUsed(name: string, refNode: Node): boolean {
  throw new Error('`sourceCode.markVariableAsUsed` not implemented yet'); // TODO
}
