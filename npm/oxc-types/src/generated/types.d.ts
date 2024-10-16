// To edit this generated file you have to edit `tasks/ast_tools/src/generators/typescript.rs`
// Auto-generated code, DO NOT EDIT DIRECTLY!

export type BooleanLiteral = {
  type: "BooleanLiteral";
  value: boolean;
} & Span;

export type NullLiteral = {
  type: "NullLiteral";
} & Span;

export type NumericLiteral = {
  type: "NumericLiteral";
  value: number;
  raw: string;
} & Span;

export type BigIntLiteral = {
  type: "BigIntLiteral";
  raw: string;
} & Span;

export type RegExpLiteral = {
  type: "RegExpLiteral";
  value: EmptyObject;
  regex: RegExp;
} & Span;

export type RegExp = {
  pattern: RegExpPattern;
  flags: RegExpFlags;
};

export type RegExpPattern = string | string | Pattern;

export type EmptyObject = {};

export type StringLiteral = {
  type: "StringLiteral";
  value: string;
} & Span;

export type Program = {
  type: "Program";
  sourceType: SourceType;
  hashbang: Hashbang | null;
  directives: Array<Directive>;
  body: Array<Statement>;
} & Span;

export type Expression =
  | BooleanLiteral
  | NullLiteral
  | NumericLiteral
  | BigIntLiteral
  | RegExpLiteral
  | StringLiteral
  | TemplateLiteral
  | IdentifierReference
  | MetaProperty
  | Super
  | ArrayExpression
  | ArrowFunctionExpression
  | AssignmentExpression
  | AwaitExpression
  | BinaryExpression
  | CallExpression
  | ChainExpression
  | Class
  | ConditionalExpression
  | Function
  | ImportExpression
  | LogicalExpression
  | NewExpression
  | ObjectExpression
  | ParenthesizedExpression
  | SequenceExpression
  | TaggedTemplateExpression
  | ThisExpression
  | UnaryExpression
  | UpdateExpression
  | YieldExpression
  | PrivateInExpression
  | JSXElement
  | JSXFragment
  | TSAsExpression
  | TSSatisfiesExpression
  | TSTypeAssertion
  | TSNonNullExpression
  | TSInstantiationExpression
  | ComputedMemberExpression
  | StaticMemberExpression
  | PrivateFieldExpression;

export type IdentifierName = {
  type: "Identifier";
  name: string;
} & Span;

export type IdentifierReference = {
  type: "Identifier";
  name: string;
} & Span;

export type BindingIdentifier = {
  type: "Identifier";
  name: string;
} & Span;

export type LabelIdentifier = {
  type: "Identifier";
  name: string;
} & Span;

export type ThisExpression = {
  type: "ThisExpression";
} & Span;

export type ArrayExpression = {
  type: "ArrayExpression";
  elements: Array<SpreadElement | Expression | null>;
} & Span;

export type ArrayExpressionElement =
  | SpreadElement
  | Elision
  | BooleanLiteral
  | NullLiteral
  | NumericLiteral
  | BigIntLiteral
  | RegExpLiteral
  | StringLiteral
  | TemplateLiteral
  | IdentifierReference
  | MetaProperty
  | Super
  | ArrayExpression
  | ArrowFunctionExpression
  | AssignmentExpression
  | AwaitExpression
  | BinaryExpression
  | CallExpression
  | ChainExpression
  | Class
  | ConditionalExpression
  | Function
  | ImportExpression
  | LogicalExpression
  | NewExpression
  | ObjectExpression
  | ParenthesizedExpression
  | SequenceExpression
  | TaggedTemplateExpression
  | ThisExpression
  | UnaryExpression
  | UpdateExpression
  | YieldExpression
  | PrivateInExpression
  | JSXElement
  | JSXFragment
  | TSAsExpression
  | TSSatisfiesExpression
  | TSTypeAssertion
  | TSNonNullExpression
  | TSInstantiationExpression
  | ComputedMemberExpression
  | StaticMemberExpression
  | PrivateFieldExpression;

export type ObjectExpression = {
  type: "ObjectExpression";
  properties: Array<ObjectPropertyKind>;
} & Span;

export type ObjectPropertyKind = ObjectProperty | SpreadElement;

export type ObjectProperty = {
  type: "ObjectProperty";
  kind: PropertyKind;
  key: PropertyKey;
  value: Expression;
  init: Expression | null;
  method: boolean;
  shorthand: boolean;
  computed: boolean;
} & Span;

export type PropertyKey =
  | IdentifierName
  | PrivateIdentifier
  | BooleanLiteral
  | NullLiteral
  | NumericLiteral
  | BigIntLiteral
  | RegExpLiteral
  | StringLiteral
  | TemplateLiteral
  | IdentifierReference
  | MetaProperty
  | Super
  | ArrayExpression
  | ArrowFunctionExpression
  | AssignmentExpression
  | AwaitExpression
  | BinaryExpression
  | CallExpression
  | ChainExpression
  | Class
  | ConditionalExpression
  | Function
  | ImportExpression
  | LogicalExpression
  | NewExpression
  | ObjectExpression
  | ParenthesizedExpression
  | SequenceExpression
  | TaggedTemplateExpression
  | ThisExpression
  | UnaryExpression
  | UpdateExpression
  | YieldExpression
  | PrivateInExpression
  | JSXElement
  | JSXFragment
  | TSAsExpression
  | TSSatisfiesExpression
  | TSTypeAssertion
  | TSNonNullExpression
  | TSInstantiationExpression
  | ComputedMemberExpression
  | StaticMemberExpression
  | PrivateFieldExpression;

export type PropertyKind = "init" | "get" | "set";

export type TemplateLiteral = {
  type: "TemplateLiteral";
  quasis: Array<TemplateElement>;
  expressions: Array<Expression>;
} & Span;

export type TaggedTemplateExpression = {
  type: "TaggedTemplateExpression";
  tag: Expression;
  quasi: TemplateLiteral;
  typeParameters: TSTypeParameterInstantiation | null;
} & Span;

export type TemplateElement = {
  type: "TemplateElement";
  tail: boolean;
  value: TemplateElementValue;
} & Span;

export type TemplateElementValue = {
  raw: string;
  cooked: string | null;
};

export type MemberExpression =
  | ComputedMemberExpression
  | StaticMemberExpression
  | PrivateFieldExpression;

export type ComputedMemberExpression = {
  type: "ComputedMemberExpression";
  object: Expression;
  expression: Expression;
  optional: boolean;
} & Span;

export type StaticMemberExpression = {
  type: "StaticMemberExpression";
  object: Expression;
  property: IdentifierName;
  optional: boolean;
} & Span;

export type PrivateFieldExpression = {
  type: "PrivateFieldExpression";
  object: Expression;
  field: PrivateIdentifier;
  optional: boolean;
} & Span;

export type CallExpression = {
  type: "CallExpression";
  callee: Expression;
  typeParameters: TSTypeParameterInstantiation | null;
  arguments: Array<Argument>;
  optional: boolean;
} & Span;

export type NewExpression = {
  type: "NewExpression";
  callee: Expression;
  arguments: Array<Argument>;
  typeParameters: TSTypeParameterInstantiation | null;
} & Span;

export type MetaProperty = {
  type: "MetaProperty";
  meta: IdentifierName;
  property: IdentifierName;
} & Span;

export type SpreadElement = {
  type: "SpreadElement";
  argument: Expression;
} & Span;

export type Argument =
  | SpreadElement
  | BooleanLiteral
  | NullLiteral
  | NumericLiteral
  | BigIntLiteral
  | RegExpLiteral
  | StringLiteral
  | TemplateLiteral
  | IdentifierReference
  | MetaProperty
  | Super
  | ArrayExpression
  | ArrowFunctionExpression
  | AssignmentExpression
  | AwaitExpression
  | BinaryExpression
  | CallExpression
  | ChainExpression
  | Class
  | ConditionalExpression
  | Function
  | ImportExpression
  | LogicalExpression
  | NewExpression
  | ObjectExpression
  | ParenthesizedExpression
  | SequenceExpression
  | TaggedTemplateExpression
  | ThisExpression
  | UnaryExpression
  | UpdateExpression
  | YieldExpression
  | PrivateInExpression
  | JSXElement
  | JSXFragment
  | TSAsExpression
  | TSSatisfiesExpression
  | TSTypeAssertion
  | TSNonNullExpression
  | TSInstantiationExpression
  | ComputedMemberExpression
  | StaticMemberExpression
  | PrivateFieldExpression;

export type UpdateExpression = {
  type: "UpdateExpression";
  operator: UpdateOperator;
  prefix: boolean;
  argument: SimpleAssignmentTarget;
} & Span;

export type UnaryExpression = {
  type: "UnaryExpression";
  operator: UnaryOperator;
  argument: Expression;
} & Span;

export type BinaryExpression = {
  type: "BinaryExpression";
  left: Expression;
  operator: BinaryOperator;
  right: Expression;
} & Span;

export type PrivateInExpression = {
  type: "PrivateInExpression";
  left: PrivateIdentifier;
  operator: BinaryOperator;
  right: Expression;
} & Span;

export type LogicalExpression = {
  type: "LogicalExpression";
  left: Expression;
  operator: LogicalOperator;
  right: Expression;
} & Span;

export type ConditionalExpression = {
  type: "ConditionalExpression";
  test: Expression;
  consequent: Expression;
  alternate: Expression;
} & Span;

export type AssignmentExpression = {
  type: "AssignmentExpression";
  operator: AssignmentOperator;
  left: AssignmentTarget;
  right: Expression;
} & Span;

export type AssignmentTarget =
  | IdentifierReference
  | TSAsExpression
  | TSSatisfiesExpression
  | TSNonNullExpression
  | TSTypeAssertion
  | TSInstantiationExpression
  | ComputedMemberExpression
  | StaticMemberExpression
  | PrivateFieldExpression
  | ArrayAssignmentTarget
  | ObjectAssignmentTarget;

export type SimpleAssignmentTarget =
  | IdentifierReference
  | TSAsExpression
  | TSSatisfiesExpression
  | TSNonNullExpression
  | TSTypeAssertion
  | TSInstantiationExpression
  | ComputedMemberExpression
  | StaticMemberExpression
  | PrivateFieldExpression;

export type AssignmentTargetPattern =
  | ArrayAssignmentTarget
  | ObjectAssignmentTarget;

export type AssignmentTargetRest = {
  type: "RestElement";
  argument: AssignmentTarget;
} & Span;

export type AssignmentTargetMaybeDefault =
  | AssignmentTargetWithDefault
  | IdentifierReference
  | TSAsExpression
  | TSSatisfiesExpression
  | TSNonNullExpression
  | TSTypeAssertion
  | TSInstantiationExpression
  | ComputedMemberExpression
  | StaticMemberExpression
  | PrivateFieldExpression
  | ArrayAssignmentTarget
  | ObjectAssignmentTarget;

export type AssignmentTargetWithDefault = {
  type: "AssignmentTargetWithDefault";
  binding: AssignmentTarget;
  init: Expression;
} & Span;

export type AssignmentTargetProperty =
  | AssignmentTargetPropertyIdentifier
  | AssignmentTargetPropertyProperty;

export type AssignmentTargetPropertyIdentifier = {
  type: "AssignmentTargetPropertyIdentifier";
  binding: IdentifierReference;
  init: Expression | null;
} & Span;

export type AssignmentTargetPropertyProperty = {
  type: "AssignmentTargetPropertyProperty";
  name: PropertyKey;
  binding: AssignmentTargetMaybeDefault;
} & Span;

export type SequenceExpression = {
  type: "SequenceExpression";
  expressions: Array<Expression>;
} & Span;

export type Super = {
  type: "Super";
} & Span;

export type AwaitExpression = {
  type: "AwaitExpression";
  argument: Expression;
} & Span;

export type ChainExpression = {
  type: "ChainExpression";
  expression: ChainElement;
} & Span;

export type ChainElement =
  | CallExpression
  | ComputedMemberExpression
  | StaticMemberExpression
  | PrivateFieldExpression;

export type ParenthesizedExpression = {
  type: "ParenthesizedExpression";
  expression: Expression;
} & Span;

export type Statement =
  | BlockStatement
  | BreakStatement
  | ContinueStatement
  | DebuggerStatement
  | DoWhileStatement
  | EmptyStatement
  | ExpressionStatement
  | ForInStatement
  | ForOfStatement
  | ForStatement
  | IfStatement
  | LabeledStatement
  | ReturnStatement
  | SwitchStatement
  | ThrowStatement
  | TryStatement
  | WhileStatement
  | WithStatement
  | VariableDeclaration
  | Function
  | Class
  | TSTypeAliasDeclaration
  | TSInterfaceDeclaration
  | TSEnumDeclaration
  | TSModuleDeclaration
  | TSImportEqualsDeclaration
  | ImportDeclaration
  | ExportAllDeclaration
  | ExportDefaultDeclaration
  | ExportNamedDeclaration
  | TSExportAssignment
  | TSNamespaceExportDeclaration;

export type Directive = {
  type: "Directive";
  expression: StringLiteral;
  directive: string;
} & Span;

export type Hashbang = {
  type: "Hashbang";
  value: string;
} & Span;

export type BlockStatement = {
  type: "BlockStatement";
  body: Array<Statement>;
} & Span;

export type Declaration =
  | VariableDeclaration
  | Function
  | Class
  | TSTypeAliasDeclaration
  | TSInterfaceDeclaration
  | TSEnumDeclaration
  | TSModuleDeclaration
  | TSImportEqualsDeclaration;

export type VariableDeclaration = {
  type: "VariableDeclaration";
  kind: VariableDeclarationKind;
  declarations: Array<VariableDeclarator>;
  declare: boolean;
} & Span;

export type VariableDeclarationKind =
  | "var"
  | "const"
  | "let"
  | "using"
  | "await using";

export type VariableDeclarator = {
  type: "VariableDeclarator";
  id: BindingPattern;
  init: Expression | null;
  definite: boolean;
} & Span;

export type EmptyStatement = {
  type: "EmptyStatement";
} & Span;

export type ExpressionStatement = {
  type: "ExpressionStatement";
  expression: Expression;
} & Span;

export type IfStatement = {
  type: "IfStatement";
  test: Expression;
  consequent: Statement;
  alternate: Statement | null;
} & Span;

export type DoWhileStatement = {
  type: "DoWhileStatement";
  body: Statement;
  test: Expression;
} & Span;

export type WhileStatement = {
  type: "WhileStatement";
  test: Expression;
  body: Statement;
} & Span;

export type ForStatement = {
  type: "ForStatement";
  init: ForStatementInit | null;
  test: Expression | null;
  update: Expression | null;
  body: Statement;
} & Span;

export type ForStatementInit =
  | VariableDeclaration
  | BooleanLiteral
  | NullLiteral
  | NumericLiteral
  | BigIntLiteral
  | RegExpLiteral
  | StringLiteral
  | TemplateLiteral
  | IdentifierReference
  | MetaProperty
  | Super
  | ArrayExpression
  | ArrowFunctionExpression
  | AssignmentExpression
  | AwaitExpression
  | BinaryExpression
  | CallExpression
  | ChainExpression
  | Class
  | ConditionalExpression
  | Function
  | ImportExpression
  | LogicalExpression
  | NewExpression
  | ObjectExpression
  | ParenthesizedExpression
  | SequenceExpression
  | TaggedTemplateExpression
  | ThisExpression
  | UnaryExpression
  | UpdateExpression
  | YieldExpression
  | PrivateInExpression
  | JSXElement
  | JSXFragment
  | TSAsExpression
  | TSSatisfiesExpression
  | TSTypeAssertion
  | TSNonNullExpression
  | TSInstantiationExpression
  | ComputedMemberExpression
  | StaticMemberExpression
  | PrivateFieldExpression;

export type ForInStatement = {
  type: "ForInStatement";
  left: ForStatementLeft;
  right: Expression;
  body: Statement;
} & Span;

export type ForStatementLeft =
  | VariableDeclaration
  | IdentifierReference
  | TSAsExpression
  | TSSatisfiesExpression
  | TSNonNullExpression
  | TSTypeAssertion
  | TSInstantiationExpression
  | ComputedMemberExpression
  | StaticMemberExpression
  | PrivateFieldExpression
  | ArrayAssignmentTarget
  | ObjectAssignmentTarget;

export type ForOfStatement = {
  type: "ForOfStatement";
  await: boolean;
  left: ForStatementLeft;
  right: Expression;
  body: Statement;
} & Span;

export type ContinueStatement = {
  type: "ContinueStatement";
  label: LabelIdentifier | null;
} & Span;

export type BreakStatement = {
  type: "BreakStatement";
  label: LabelIdentifier | null;
} & Span;

export type ReturnStatement = {
  type: "ReturnStatement";
  argument: Expression | null;
} & Span;

export type WithStatement = {
  type: "WithStatement";
  object: Expression;
  body: Statement;
} & Span;

export type SwitchStatement = {
  type: "SwitchStatement";
  discriminant: Expression;
  cases: Array<SwitchCase>;
} & Span;

export type SwitchCase = {
  type: "SwitchCase";
  test: Expression | null;
  consequent: Array<Statement>;
} & Span;

export type LabeledStatement = {
  type: "LabeledStatement";
  label: LabelIdentifier;
  body: Statement;
} & Span;

export type ThrowStatement = {
  type: "ThrowStatement";
  argument: Expression;
} & Span;

export type TryStatement = {
  type: "TryStatement";
  block: BlockStatement;
  handler: CatchClause | null;
  finalizer: BlockStatement | null;
} & Span;

export type CatchClause = {
  type: "CatchClause";
  param: CatchParameter | null;
  body: BlockStatement;
} & Span;

export type CatchParameter = {
  type: "CatchParameter";
  pattern: BindingPattern;
} & Span;

export type DebuggerStatement = {
  type: "DebuggerStatement";
} & Span;

export type BindingPattern = {
  typeAnnotation: TSTypeAnnotation | null;
  optional: boolean;
} & (BindingIdentifier | ObjectPattern | ArrayPattern | AssignmentPattern);

export type BindingPatternKind =
  | BindingIdentifier
  | ObjectPattern
  | ArrayPattern
  | AssignmentPattern;

export type AssignmentPattern = {
  type: "AssignmentPattern";
  left: BindingPattern;
  right: Expression;
} & Span;

export type BindingProperty = {
  type: "BindingProperty";
  key: PropertyKey;
  value: BindingPattern;
  shorthand: boolean;
  computed: boolean;
} & Span;

export type BindingRestElement = {
  type: "RestElement";
  argument: BindingPattern;
} & Span;

export type Function = {
  type: FunctionType;
  id: BindingIdentifier | null;
  generator: boolean;
  async: boolean;
  declare: boolean;
  typeParameters: TSTypeParameterDeclaration | null;
  thisParam: TSThisParameter | null;
  params: FormalParameters;
  returnType: TSTypeAnnotation | null;
  body: FunctionBody | null;
} & Span;

export type FunctionType =
  | "FunctionDeclaration"
  | "FunctionExpression"
  | "TSDeclareFunction"
  | "TSEmptyBodyFunctionExpression";

export type FormalParameter = {
  type: "FormalParameter";
  decorators: Array<Decorator>;
  pattern: BindingPattern;
  accessibility: TSAccessibility | null;
  readonly: boolean;
  override: boolean;
} & Span;

export type FormalParameterKind =
  | "FormalParameter"
  | "UniqueFormalParameters"
  | "ArrowFormalParameters"
  | "Signature";

export type FunctionBody = {
  type: "FunctionBody";
  directives: Array<Directive>;
  statements: Array<Statement>;
} & Span;

export type ArrowFunctionExpression = {
  type: "ArrowFunctionExpression";
  expression: boolean;
  async: boolean;
  typeParameters: TSTypeParameterDeclaration | null;
  params: FormalParameters;
  returnType: TSTypeAnnotation | null;
  body: FunctionBody;
} & Span;

export type YieldExpression = {
  type: "YieldExpression";
  delegate: boolean;
  argument: Expression | null;
} & Span;

export type Class = {
  type: ClassType;
  decorators: Array<Decorator>;
  id: BindingIdentifier | null;
  typeParameters: TSTypeParameterDeclaration | null;
  superClass: Expression | null;
  superTypeParameters: TSTypeParameterInstantiation | null;
  implements: Array<TSClassImplements> | null;
  body: ClassBody;
  abstract: boolean;
  declare: boolean;
} & Span;

export type ClassType = "ClassDeclaration" | "ClassExpression";

export type ClassBody = {
  type: "ClassBody";
  body: Array<ClassElement>;
} & Span;

export type ClassElement =
  | StaticBlock
  | MethodDefinition
  | PropertyDefinition
  | AccessorProperty
  | TSIndexSignature;

export type MethodDefinition = {
  type: MethodDefinitionType;
  decorators: Array<Decorator>;
  key: PropertyKey;
  value: Function;
  kind: MethodDefinitionKind;
  computed: boolean;
  static: boolean;
  override: boolean;
  optional: boolean;
  accessibility: TSAccessibility | null;
} & Span;

export type MethodDefinitionType =
  | "MethodDefinition"
  | "TSAbstractMethodDefinition";

export type PropertyDefinition = {
  type: PropertyDefinitionType;
  decorators: Array<Decorator>;
  key: PropertyKey;
  value: Expression | null;
  computed: boolean;
  static: boolean;
  declare: boolean;
  override: boolean;
  optional: boolean;
  definite: boolean;
  readonly: boolean;
  typeAnnotation: TSTypeAnnotation | null;
  accessibility: TSAccessibility | null;
} & Span;

export type PropertyDefinitionType =
  | "PropertyDefinition"
  | "TSAbstractPropertyDefinition";

export type MethodDefinitionKind = "constructor" | "method" | "get" | "set";

export type PrivateIdentifier = {
  type: "PrivateIdentifier";
  name: string;
} & Span;

export type StaticBlock = {
  type: "StaticBlock";
  body: Array<Statement>;
} & Span;

export type ModuleDeclaration =
  | ImportDeclaration
  | ExportAllDeclaration
  | ExportDefaultDeclaration
  | ExportNamedDeclaration
  | TSExportAssignment
  | TSNamespaceExportDeclaration;

export type AccessorPropertyType =
  | "AccessorProperty"
  | "TSAbstractAccessorProperty";

export type AccessorProperty = {
  type: AccessorPropertyType;
  decorators: Array<Decorator>;
  key: PropertyKey;
  value: Expression | null;
  computed: boolean;
  static: boolean;
  definite: boolean;
  typeAnnotation: TSTypeAnnotation | null;
  accessibility: TSAccessibility | null;
} & Span;

export type ImportExpression = {
  type: "ImportExpression";
  source: Expression;
  arguments: Array<Expression>;
} & Span;

export type ImportDeclaration = {
  type: "ImportDeclaration";
  specifiers: Array<ImportDeclarationSpecifier> | null;
  source: StringLiteral;
  withClause: WithClause | null;
  importKind: ImportOrExportKind;
} & Span;

export type ImportDeclarationSpecifier =
  | ImportSpecifier
  | ImportDefaultSpecifier
  | ImportNamespaceSpecifier;

export type ImportSpecifier = {
  type: "ImportSpecifier";
  imported: ModuleExportName;
  local: BindingIdentifier;
  importKind: ImportOrExportKind;
} & Span;

export type ImportDefaultSpecifier = {
  type: "ImportDefaultSpecifier";
  local: BindingIdentifier;
} & Span;

export type ImportNamespaceSpecifier = {
  type: "ImportNamespaceSpecifier";
  local: BindingIdentifier;
} & Span;

export type WithClause = {
  type: "WithClause";
  attributesKeyword: IdentifierName;
  withEntries: Array<ImportAttribute>;
} & Span;

export type ImportAttribute = {
  type: "ImportAttribute";
  key: ImportAttributeKey;
  value: StringLiteral;
} & Span;

export type ImportAttributeKey = IdentifierName | StringLiteral;

export type ExportNamedDeclaration = {
  type: "ExportNamedDeclaration";
  declaration: Declaration | null;
  specifiers: Array<ExportSpecifier>;
  source: StringLiteral | null;
  exportKind: ImportOrExportKind;
  withClause: WithClause | null;
} & Span;

export type ExportDefaultDeclaration = {
  type: "ExportDefaultDeclaration";
  declaration: ExportDefaultDeclarationKind;
  exported: ModuleExportName;
} & Span;

export type ExportAllDeclaration = {
  type: "ExportAllDeclaration";
  exported: ModuleExportName | null;
  source: StringLiteral;
  withClause: WithClause | null;
  exportKind: ImportOrExportKind;
} & Span;

export type ExportSpecifier = {
  type: "ExportSpecifier";
  local: ModuleExportName;
  exported: ModuleExportName;
  exportKind: ImportOrExportKind;
} & Span;

export type ExportDefaultDeclarationKind =
  | Function
  | Class
  | TSInterfaceDeclaration
  | BooleanLiteral
  | NullLiteral
  | NumericLiteral
  | BigIntLiteral
  | RegExpLiteral
  | StringLiteral
  | TemplateLiteral
  | IdentifierReference
  | MetaProperty
  | Super
  | ArrayExpression
  | ArrowFunctionExpression
  | AssignmentExpression
  | AwaitExpression
  | BinaryExpression
  | CallExpression
  | ChainExpression
  | Class
  | ConditionalExpression
  | Function
  | ImportExpression
  | LogicalExpression
  | NewExpression
  | ObjectExpression
  | ParenthesizedExpression
  | SequenceExpression
  | TaggedTemplateExpression
  | ThisExpression
  | UnaryExpression
  | UpdateExpression
  | YieldExpression
  | PrivateInExpression
  | JSXElement
  | JSXFragment
  | TSAsExpression
  | TSSatisfiesExpression
  | TSTypeAssertion
  | TSNonNullExpression
  | TSInstantiationExpression
  | ComputedMemberExpression
  | StaticMemberExpression
  | PrivateFieldExpression;

export type ModuleExportName =
  | IdentifierName
  | IdentifierReference
  | StringLiteral;

export type TSThisParameter = {
  type: "TSThisParameter";
  thisSpan: Span;
  typeAnnotation: TSTypeAnnotation | null;
} & Span;

export type TSEnumDeclaration = {
  type: "TSEnumDeclaration";
  id: BindingIdentifier;
  members: Array<TSEnumMember>;
  const: boolean;
  declare: boolean;
} & Span;

export type TSEnumMember = {
  type: "TSEnumMember";
  id: TSEnumMemberName;
  initializer: Expression | null;
} & Span;

export type TSEnumMemberName =
  | IdentifierName
  | StringLiteral
  | TemplateLiteral
  | NumericLiteral
  | BooleanLiteral
  | NullLiteral
  | NumericLiteral
  | BigIntLiteral
  | RegExpLiteral
  | StringLiteral
  | TemplateLiteral
  | IdentifierReference
  | MetaProperty
  | Super
  | ArrayExpression
  | ArrowFunctionExpression
  | AssignmentExpression
  | AwaitExpression
  | BinaryExpression
  | CallExpression
  | ChainExpression
  | Class
  | ConditionalExpression
  | Function
  | ImportExpression
  | LogicalExpression
  | NewExpression
  | ObjectExpression
  | ParenthesizedExpression
  | SequenceExpression
  | TaggedTemplateExpression
  | ThisExpression
  | UnaryExpression
  | UpdateExpression
  | YieldExpression
  | PrivateInExpression
  | JSXElement
  | JSXFragment
  | TSAsExpression
  | TSSatisfiesExpression
  | TSTypeAssertion
  | TSNonNullExpression
  | TSInstantiationExpression
  | ComputedMemberExpression
  | StaticMemberExpression
  | PrivateFieldExpression;

export type TSTypeAnnotation = {
  type: "TSTypeAnnotation";
  typeAnnotation: TSType;
} & Span;

export type TSLiteralType = {
  type: "TSLiteralType";
  literal: TSLiteral;
} & Span;

export type TSLiteral =
  | BooleanLiteral
  | NullLiteral
  | NumericLiteral
  | BigIntLiteral
  | RegExpLiteral
  | StringLiteral
  | TemplateLiteral
  | UnaryExpression;

export type TSType =
  | TSAnyKeyword
  | TSBigIntKeyword
  | TSBooleanKeyword
  | TSIntrinsicKeyword
  | TSNeverKeyword
  | TSNullKeyword
  | TSNumberKeyword
  | TSObjectKeyword
  | TSStringKeyword
  | TSSymbolKeyword
  | TSUndefinedKeyword
  | TSUnknownKeyword
  | TSVoidKeyword
  | TSArrayType
  | TSConditionalType
  | TSConstructorType
  | TSFunctionType
  | TSImportType
  | TSIndexedAccessType
  | TSInferType
  | TSIntersectionType
  | TSLiteralType
  | TSMappedType
  | TSNamedTupleMember
  | TSQualifiedName
  | TSTemplateLiteralType
  | TSThisType
  | TSTupleType
  | TSTypeLiteral
  | TSTypeOperator
  | TSTypePredicate
  | TSTypeQuery
  | TSTypeReference
  | TSUnionType
  | TSParenthesizedType
  | JSDocNullableType
  | JSDocNonNullableType
  | JSDocUnknownType;

export type TSConditionalType = {
  type: "TSConditionalType";
  checkType: TSType;
  extendsType: TSType;
  trueType: TSType;
  falseType: TSType;
} & Span;

export type TSUnionType = {
  type: "TSUnionType";
  types: Array<TSType>;
} & Span;

export type TSIntersectionType = {
  type: "TSIntersectionType";
  types: Array<TSType>;
} & Span;

export type TSParenthesizedType = {
  type: "TSParenthesizedType";
  typeAnnotation: TSType;
} & Span;

export type TSTypeOperator = {
  type: "TSTypeOperator";
  operator: TSTypeOperatorOperator;
  typeAnnotation: TSType;
} & Span;

export type TSTypeOperatorOperator = "keyof" | "unique" | "readonly";

export type TSArrayType = {
  type: "TSArrayType";
  elementType: TSType;
} & Span;

export type TSIndexedAccessType = {
  type: "TSIndexedAccessType";
  objectType: TSType;
  indexType: TSType;
} & Span;

export type TSTupleType = {
  type: "TSTupleType";
  elementTypes: Array<TSTupleElement>;
} & Span;

export type TSNamedTupleMember = {
  type: "TSNamedTupleMember";
  elementType: TSTupleElement;
  label: IdentifierName;
  optional: boolean;
} & Span;

export type TSOptionalType = {
  type: "TSOptionalType";
  typeAnnotation: TSType;
} & Span;

export type TSRestType = {
  type: "TSRestType";
  typeAnnotation: TSType;
} & Span;

export type TSTupleElement =
  | TSOptionalType
  | TSRestType
  | TSAnyKeyword
  | TSBigIntKeyword
  | TSBooleanKeyword
  | TSIntrinsicKeyword
  | TSNeverKeyword
  | TSNullKeyword
  | TSNumberKeyword
  | TSObjectKeyword
  | TSStringKeyword
  | TSSymbolKeyword
  | TSUndefinedKeyword
  | TSUnknownKeyword
  | TSVoidKeyword
  | TSArrayType
  | TSConditionalType
  | TSConstructorType
  | TSFunctionType
  | TSImportType
  | TSIndexedAccessType
  | TSInferType
  | TSIntersectionType
  | TSLiteralType
  | TSMappedType
  | TSNamedTupleMember
  | TSQualifiedName
  | TSTemplateLiteralType
  | TSThisType
  | TSTupleType
  | TSTypeLiteral
  | TSTypeOperator
  | TSTypePredicate
  | TSTypeQuery
  | TSTypeReference
  | TSUnionType
  | TSParenthesizedType
  | JSDocNullableType
  | JSDocNonNullableType
  | JSDocUnknownType;

export type TSAnyKeyword = {
  type: "TSAnyKeyword";
} & Span;

export type TSStringKeyword = {
  type: "TSStringKeyword";
} & Span;

export type TSBooleanKeyword = {
  type: "TSBooleanKeyword";
} & Span;

export type TSNumberKeyword = {
  type: "TSNumberKeyword";
} & Span;

export type TSNeverKeyword = {
  type: "TSNeverKeyword";
} & Span;

export type TSIntrinsicKeyword = {
  type: "TSIntrinsicKeyword";
} & Span;

export type TSUnknownKeyword = {
  type: "TSUnknownKeyword";
} & Span;

export type TSNullKeyword = {
  type: "TSNullKeyword";
} & Span;

export type TSUndefinedKeyword = {
  type: "TSUndefinedKeyword";
} & Span;

export type TSVoidKeyword = {
  type: "TSVoidKeyword";
} & Span;

export type TSSymbolKeyword = {
  type: "TSSymbolKeyword";
} & Span;

export type TSThisType = {
  type: "TSThisType";
} & Span;

export type TSObjectKeyword = {
  type: "TSObjectKeyword";
} & Span;

export type TSBigIntKeyword = {
  type: "TSBigIntKeyword";
} & Span;

export type TSTypeReference = {
  type: "TSTypeReference";
  typeName: TSTypeName;
  typeParameters: TSTypeParameterInstantiation | null;
} & Span;

export type TSTypeName = IdentifierReference | TSQualifiedName;

export type TSQualifiedName = {
  type: "TSQualifiedName";
  left: TSTypeName;
  right: IdentifierName;
} & Span;

export type TSTypeParameterInstantiation = {
  type: "TSTypeParameterInstantiation";
  params: Array<TSType>;
} & Span;

export type TSTypeParameter = {
  type: "TSTypeParameter";
  name: BindingIdentifier;
  constraint: TSType | null;
  default: TSType | null;
  in: boolean;
  out: boolean;
  const: boolean;
} & Span;

export type TSTypeParameterDeclaration = {
  type: "TSTypeParameterDeclaration";
  params: Array<TSTypeParameter>;
} & Span;

export type TSTypeAliasDeclaration = {
  type: "TSTypeAliasDeclaration";
  id: BindingIdentifier;
  typeParameters: TSTypeParameterDeclaration | null;
  typeAnnotation: TSType;
  declare: boolean;
} & Span;

export type TSAccessibility = "private" | "protected" | "public";

export type TSClassImplements = {
  type: "TSClassImplements";
  expression: TSTypeName;
  typeParameters: TSTypeParameterInstantiation | null;
} & Span;

export type TSInterfaceDeclaration = {
  type: "TSInterfaceDeclaration";
  id: BindingIdentifier;
  extends: Array<TSInterfaceHeritage> | null;
  typeParameters: TSTypeParameterDeclaration | null;
  body: TSInterfaceBody;
  declare: boolean;
} & Span;

export type TSInterfaceBody = {
  type: "TSInterfaceBody";
  body: Array<TSSignature>;
} & Span;

export type TSPropertySignature = {
  type: "TSPropertySignature";
  computed: boolean;
  optional: boolean;
  readonly: boolean;
  key: PropertyKey;
  typeAnnotation: TSTypeAnnotation | null;
} & Span;

export type TSSignature =
  | TSIndexSignature
  | TSPropertySignature
  | TSCallSignatureDeclaration
  | TSConstructSignatureDeclaration
  | TSMethodSignature;

export type TSIndexSignature = {
  type: "TSIndexSignature";
  parameters: Array<TSIndexSignatureName>;
  typeAnnotation: TSTypeAnnotation;
  readonly: boolean;
} & Span;

export type TSCallSignatureDeclaration = {
  type: "TSCallSignatureDeclaration";
  thisParam: TSThisParameter | null;
  params: FormalParameters;
  returnType: TSTypeAnnotation | null;
  typeParameters: TSTypeParameterDeclaration | null;
} & Span;

export type TSMethodSignatureKind = "method" | "get" | "set";

export type TSMethodSignature = {
  type: "TSMethodSignature";
  key: PropertyKey;
  computed: boolean;
  optional: boolean;
  kind: TSMethodSignatureKind;
  thisParam: TSThisParameter | null;
  params: FormalParameters;
  returnType: TSTypeAnnotation | null;
  typeParameters: TSTypeParameterDeclaration | null;
} & Span;

export type TSConstructSignatureDeclaration = {
  type: "TSConstructSignatureDeclaration";
  params: FormalParameters;
  returnType: TSTypeAnnotation | null;
  typeParameters: TSTypeParameterDeclaration | null;
} & Span;

export type TSIndexSignatureName = {
  type: "Identifier";
  name: string;
  typeAnnotation: TSTypeAnnotation;
} & Span;

export type TSInterfaceHeritage = {
  type: "TSInterfaceHeritage";
  expression: Expression;
  typeParameters: TSTypeParameterInstantiation | null;
} & Span;

export type TSTypePredicate = {
  type: "TSTypePredicate";
  parameterName: TSTypePredicateName;
  asserts: boolean;
  typeAnnotation: TSTypeAnnotation | null;
} & Span;

export type TSTypePredicateName = IdentifierName | TSThisType;

export type TSModuleDeclaration = {
  type: "TSModuleDeclaration";
  id: TSModuleDeclarationName;
  body: TSModuleDeclarationBody | null;
  kind: TSModuleDeclarationKind;
  declare: boolean;
} & Span;

export type TSModuleDeclarationKind = "global" | "module" | "namespace";

export type TSModuleDeclarationName = BindingIdentifier | StringLiteral;

export type TSModuleDeclarationBody = TSModuleDeclaration | TSModuleBlock;

export type TSTypeLiteral = {
  type: "TSTypeLiteral";
  members: Array<TSSignature>;
} & Span;

export type TSInferType = {
  type: "TSInferType";
  typeParameter: TSTypeParameter;
} & Span;

export type TSTypeQuery = {
  type: "TSTypeQuery";
  exprName: TSTypeQueryExprName;
  typeParameters: TSTypeParameterInstantiation | null;
} & Span;

export type TSTypeQueryExprName =
  | TSImportType
  | IdentifierReference
  | TSQualifiedName;

export type TSImportType = {
  type: "TSImportType";
  isTypeOf: boolean;
  parameter: TSType;
  qualifier: TSTypeName | null;
  attributes: TSImportAttributes | null;
  typeParameters: TSTypeParameterInstantiation | null;
} & Span;

export type TSImportAttributes = {
  type: "TSImportAttributes";
  attributesKeyword: IdentifierName;
  elements: Array<TSImportAttribute>;
} & Span;

export type TSImportAttribute = {
  type: "TSImportAttribute";
  name: TSImportAttributeName;
  value: Expression;
} & Span;

export type TSImportAttributeName = IdentifierName | StringLiteral;

export type TSFunctionType = {
  type: "TSFunctionType";
  thisParam: TSThisParameter | null;
  params: FormalParameters;
  returnType: TSTypeAnnotation;
  typeParameters: TSTypeParameterDeclaration | null;
} & Span;

export type TSConstructorType = {
  type: "TSConstructorType";
  abstract: boolean;
  params: FormalParameters;
  returnType: TSTypeAnnotation;
  typeParameters: TSTypeParameterDeclaration | null;
} & Span;

export type TSMappedType = {
  type: "TSMappedType";
  typeParameter: TSTypeParameter;
  nameType: TSType | null;
  typeAnnotation: TSType | null;
  optional: TSMappedTypeModifierOperator;
  readonly: TSMappedTypeModifierOperator;
} & Span;

export type TSMappedTypeModifierOperator = "true" | "+" | "-" | "none";

export type TSTemplateLiteralType = {
  type: "TSTemplateLiteralType";
  quasis: Array<TemplateElement>;
  types: Array<TSType>;
} & Span;

export type TSAsExpression = {
  type: "TSAsExpression";
  expression: Expression;
  typeAnnotation: TSType;
} & Span;

export type TSSatisfiesExpression = {
  type: "TSSatisfiesExpression";
  expression: Expression;
  typeAnnotation: TSType;
} & Span;

export type TSTypeAssertion = {
  type: "TSTypeAssertion";
  expression: Expression;
  typeAnnotation: TSType;
} & Span;

export type TSImportEqualsDeclaration = {
  type: "TSImportEqualsDeclaration";
  id: BindingIdentifier;
  moduleReference: TSModuleReference;
  importKind: ImportOrExportKind;
} & Span;

export type TSModuleReference =
  | TSExternalModuleReference
  | IdentifierReference
  | TSQualifiedName;

export type TSExternalModuleReference = {
  type: "TSExternalModuleReference";
  expression: StringLiteral;
} & Span;

export type TSNonNullExpression = {
  type: "TSNonNullExpression";
  expression: Expression;
} & Span;

export type Decorator = {
  type: "Decorator";
  expression: Expression;
} & Span;

export type TSExportAssignment = {
  type: "TSExportAssignment";
  expression: Expression;
} & Span;

export type TSNamespaceExportDeclaration = {
  type: "TSNamespaceExportDeclaration";
  id: IdentifierName;
} & Span;

export type TSInstantiationExpression = {
  type: "TSInstantiationExpression";
  expression: Expression;
  typeParameters: TSTypeParameterInstantiation;
} & Span;

export type ImportOrExportKind = "value" | "type";

export type JSDocNullableType = {
  type: "JSDocNullableType";
  typeAnnotation: TSType;
  postfix: boolean;
} & Span;

export type JSDocNonNullableType = {
  type: "JSDocNonNullableType";
  typeAnnotation: TSType;
  postfix: boolean;
} & Span;

export type JSDocUnknownType = {
  type: "JSDocUnknownType";
} & Span;

export type JSXElement = {
  type: "JSXElement";
  openingElement: JSXOpeningElement;
  closingElement: JSXClosingElement | null;
  children: Array<JSXChild>;
} & Span;

export type JSXOpeningElement = {
  type: "JSXOpeningElement";
  selfClosing: boolean;
  name: JSXElementName;
  attributes: Array<JSXAttributeItem>;
  typeParameters: TSTypeParameterInstantiation | null;
} & Span;

export type JSXClosingElement = {
  type: "JSXClosingElement";
  name: JSXElementName;
} & Span;

export type JSXFragment = {
  type: "JSXFragment";
  openingFragment: JSXOpeningFragment;
  closingFragment: JSXClosingFragment;
  children: Array<JSXChild>;
} & Span;

export type JSXOpeningFragment = {
  type: "JSXOpeningFragment";
} & Span;

export type JSXClosingFragment = {
  type: "JSXClosingFragment";
} & Span;

export type JSXNamespacedName = {
  type: "JSXNamespacedName";
  namespace: JSXIdentifier;
  property: JSXIdentifier;
} & Span;

export type JSXMemberExpression = {
  type: "JSXMemberExpression";
  object: JSXMemberExpressionObject;
  property: JSXIdentifier;
} & Span;

export type JSXExpressionContainer = {
  type: "JSXExpressionContainer";
  expression: JSXExpression;
} & Span;

export type JSXExpression =
  | JSXEmptyExpression
  | BooleanLiteral
  | NullLiteral
  | NumericLiteral
  | BigIntLiteral
  | RegExpLiteral
  | StringLiteral
  | TemplateLiteral
  | IdentifierReference
  | MetaProperty
  | Super
  | ArrayExpression
  | ArrowFunctionExpression
  | AssignmentExpression
  | AwaitExpression
  | BinaryExpression
  | CallExpression
  | ChainExpression
  | Class
  | ConditionalExpression
  | Function
  | ImportExpression
  | LogicalExpression
  | NewExpression
  | ObjectExpression
  | ParenthesizedExpression
  | SequenceExpression
  | TaggedTemplateExpression
  | ThisExpression
  | UnaryExpression
  | UpdateExpression
  | YieldExpression
  | PrivateInExpression
  | JSXElement
  | JSXFragment
  | TSAsExpression
  | TSSatisfiesExpression
  | TSTypeAssertion
  | TSNonNullExpression
  | TSInstantiationExpression
  | ComputedMemberExpression
  | StaticMemberExpression
  | PrivateFieldExpression;

export type JSXEmptyExpression = {
  type: "JSXEmptyExpression";
} & Span;

export type JSXAttributeItem = JSXAttribute | JSXSpreadAttribute;

export type JSXAttribute = {
  type: "JSXAttribute";
  name: JSXAttributeName;
  value: JSXAttributeValue | null;
} & Span;

export type JSXSpreadAttribute = {
  type: "JSXSpreadAttribute";
  argument: Expression;
} & Span;

export type JSXAttributeName = JSXIdentifier | JSXNamespacedName;

export type JSXAttributeValue =
  | StringLiteral
  | JSXExpressionContainer
  | JSXElement
  | JSXFragment;

export type JSXIdentifier = {
  type: "JSXIdentifier";
  name: string;
} & Span;

export type JSXChild =
  | JSXText
  | JSXElement
  | JSXFragment
  | JSXExpressionContainer
  | JSXSpreadChild;

export type JSXSpreadChild = {
  type: "JSXSpreadChild";
  expression: Expression;
} & Span;

export type JSXText = {
  type: "JSXText";
  value: string;
} & Span;

export type AssignmentOperator =
  | "="
  | "+="
  | "-="
  | "*="
  | "/="
  | "%="
  | "<<="
  | ">>="
  | ">>>="
  | "|="
  | "^="
  | "&="
  | "&&="
  | "||="
  | "??="
  | "**=";

export type BinaryOperator =
  | "=="
  | "!="
  | "==="
  | "!=="
  | "<"
  | "<="
  | ">"
  | ">="
  | "<<"
  | ">>"
  | ">>>"
  | "+"
  | "-"
  | "*"
  | "/"
  | "%"
  | "|"
  | "^"
  | "&"
  | "in"
  | "instanceof"
  | "**";

export type LogicalOperator = "||" | "&&" | "??";

export type UnaryOperator =
  | "-"
  | "+"
  | "!"
  | "~"
  | "typeof"
  | "void"
  | "delete";

export type UpdateOperator = "++" | "--";

export type Span = {
  type: "Span";
  start: number;
  end: number;
};

export type SourceType = {
  language: Language;
  moduleKind: ModuleKind;
  variant: LanguageVariant;
};

export type Language = "javascript" | "typescript" | "typescriptDefinition";

export type ModuleKind = "script" | "module" | "unambiguous";

export type LanguageVariant = "standard" | "jsx";

export type Pattern = {
  type: "Pattern";
  body: Disjunction;
} & Span;

export type Disjunction = {
  type: "Disjunction";
  body: Array<Alternative>;
} & Span;

export type Alternative = {
  type: "Alternative";
  body: Array<Term>;
} & Span;

export type Term =
  | BoundaryAssertion
  | LookAroundAssertion
  | Quantifier
  | Character
  | Dot
  | CharacterClassEscape
  | UnicodePropertyEscape
  | CharacterClass
  | CapturingGroup
  | IgnoreGroup
  | IndexedReference
  | NamedReference;

export type BoundaryAssertion = {
  type: "BoundaryAssertion";
  span: Span;
  kind: BoundaryAssertionKind;
};

export type BoundaryAssertionKind =
  | "start"
  | "end"
  | "boundary"
  | "negativeBoundary";

export type LookAroundAssertion = {
  type: "LookAroundAssertion";
  kind: LookAroundAssertionKind;
  body: Disjunction;
} & Span;

export type LookAroundAssertionKind =
  | "lookahead"
  | "negativeLookahead"
  | "lookbehind"
  | "negativeLookbehind";

export type Quantifier = {
  type: "Quantifier";
  min: number;
  max: number | null;
  greedy: boolean;
  body: Term;
} & Span;

export type Character = {
  type: "Character";
  kind: CharacterKind;
  value: number;
} & Span;

export type CharacterKind =
  | "controlLetter"
  | "hexadecimalEscape"
  | "identifier"
  | "null"
  | "octal1"
  | "octal2"
  | "octal3"
  | "singleEscape"
  | "symbol"
  | "unicodeEscape";

export type CharacterClassEscape = {
  type: "CharacterClassEscape";
  kind: CharacterClassEscapeKind;
} & Span;

export type CharacterClassEscapeKind =
  | "d"
  | "negativeD"
  | "s"
  | "negativeS"
  | "w"
  | "negativeW";

export type UnicodePropertyEscape = {
  type: "UnicodePropertyEscape";
  negative: boolean;
  strings: boolean;
  name: string;
  value: string | null;
} & Span;

export type Dot = {
  type: "Dot";
} & Span;

export type CharacterClass = {
  type: "CharacterClass";
  negative: boolean;
  strings: boolean;
  kind: CharacterClassContentsKind;
  body: Array<CharacterClassContents>;
} & Span;

export type CharacterClassContentsKind =
  | "union"
  | "intersection"
  | "subtraction";

export type CharacterClassContents =
  | CharacterClassRange
  | CharacterClassEscape
  | UnicodePropertyEscape
  | Character
  | CharacterClass
  | ClassStringDisjunction;

export type CharacterClassRange = {
  type: "CharacterClassRange";
  min: Character;
  max: Character;
} & Span;

export type ClassStringDisjunction = {
  type: "ClassStringDisjunction";
  strings: boolean;
  body: Array<ClassString>;
} & Span;

export type ClassString = {
  type: "ClassString";
  strings: boolean;
  body: Array<Character>;
} & Span;

export type CapturingGroup = {
  type: "CapturingGroup";
  name: string | null;
  body: Disjunction;
} & Span;

export type IgnoreGroup = {
  type: "IgnoreGroup";
  modifiers: Modifiers | null;
  body: Disjunction;
} & Span;

export type Modifiers = {
  type: "Modifiers";
  enabling: Modifier | null;
  disabling: Modifier | null;
} & Span;

export type Modifier = {
  type: "Modifier";
  ignoreCase: boolean;
  multiline: boolean;
  sticky: boolean;
};

export type IndexedReference = {
  type: "IndexedReference";
  index: number;
} & Span;

export type NamedReference = {
  type: "NamedReference";
  name: string;
} & Span;
