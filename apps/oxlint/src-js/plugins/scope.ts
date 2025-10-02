import type * as ESTree from '@oxc-project/types';

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
