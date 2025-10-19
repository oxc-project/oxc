/*
 * `SourceCode` methods related to scopes.
 */

import type * as ESTree from '../generated/types.d.ts';

import type { Node } from './types.ts';

type Identifier =
  | ESTree.IdentifierName
  | ESTree.IdentifierReference
  | ESTree.BindingIdentifier
  | ESTree.LabelIdentifier
  | ESTree.TSThisParameter
  | ESTree.TSIndexSignatureName;

export class ScopeManager {
  // TODO
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
