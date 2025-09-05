// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/typescript.rs`.

export interface Program {
  type: 'Program';
  body: Array<Directive | Statement>;
  sourceType: ModuleKind;
  hashbang: Hashbang | null;
  0: number;
}

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
  | V8IntrinsicExpression
  | MemberExpression;

export interface IdentifierName {
  type: 'Identifier';
  decorators?: [];
  name: string;
  optional?: false;
  typeAnnotation?: null;
  0: number;
}

export interface IdentifierReference {
  type: 'Identifier';
  decorators?: [];
  name: string;
  optional?: false;
  typeAnnotation?: null;
  0: number;
}

export interface BindingIdentifier {
  type: 'Identifier';
  decorators?: [];
  name: string;
  optional?: false;
  typeAnnotation?: null;
  0: number;
}

export interface LabelIdentifier {
  type: 'Identifier';
  decorators?: [];
  name: string;
  optional?: false;
  typeAnnotation?: null;
  0: number;
}

export interface ThisExpression {
  type: 'ThisExpression';
  0: number;
}

export interface ArrayExpression {
  type: 'ArrayExpression';
  elements: Array<ArrayExpressionElement>;
  0: number;
}

export type ArrayExpressionElement = SpreadElement | null | Expression;

export interface ObjectExpression {
  type: 'ObjectExpression';
  properties: Array<ObjectPropertyKind>;
  0: number;
}

export type ObjectPropertyKind = ObjectProperty | SpreadElement;

export interface ObjectProperty {
  type: 'Property';
  kind: PropertyKind;
  key: PropertyKey;
  value: Expression;
  method: boolean;
  shorthand: boolean;
  computed: boolean;
  optional?: false;
  0: number;
}

export type PropertyKey = IdentifierName | PrivateIdentifier | Expression;

export type PropertyKind = 'init' | 'get' | 'set';

export interface TemplateLiteral {
  type: 'TemplateLiteral';
  quasis: Array<TemplateElement>;
  expressions: Array<Expression>;
  0: number;
}

export interface TaggedTemplateExpression {
  type: 'TaggedTemplateExpression';
  tag: Expression;
  typeArguments?: TSTypeParameterInstantiation | null;
  quasi: TemplateLiteral;
  0: number;
}

export interface TemplateElement {
  type: 'TemplateElement';
  value: TemplateElementValue;
  tail: boolean;
  0: number;
}

export interface TemplateElementValue {
  raw: string;
  cooked: string | null;
}

export type MemberExpression = ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression;

export interface ComputedMemberExpression {
  type: 'MemberExpression';
  object: Expression;
  property: Expression;
  optional: boolean;
  computed: true;
  0: number;
}

export interface StaticMemberExpression {
  type: 'MemberExpression';
  object: Expression;
  property: IdentifierName;
  optional: boolean;
  computed: false;
  0: number;
}

export interface PrivateFieldExpression {
  type: 'MemberExpression';
  object: Expression;
  property: PrivateIdentifier;
  optional: boolean;
  computed: false;
  0: number;
}

export interface CallExpression {
  type: 'CallExpression';
  callee: Expression;
  typeArguments?: TSTypeParameterInstantiation | null;
  arguments: Array<Argument>;
  optional: boolean;
  0: number;
}

export interface NewExpression {
  type: 'NewExpression';
  callee: Expression;
  typeArguments?: TSTypeParameterInstantiation | null;
  arguments: Array<Argument>;
  0: number;
}

export interface MetaProperty {
  type: 'MetaProperty';
  meta: IdentifierName;
  property: IdentifierName;
  0: number;
}

export interface SpreadElement {
  type: 'SpreadElement';
  argument: Expression;
  0: number;
}

export type Argument = SpreadElement | Expression;

export interface UpdateExpression {
  type: 'UpdateExpression';
  operator: UpdateOperator;
  prefix: boolean;
  argument: SimpleAssignmentTarget;
  0: number;
}

export interface UnaryExpression {
  type: 'UnaryExpression';
  operator: UnaryOperator;
  argument: Expression;
  prefix: true;
  0: number;
}

export interface BinaryExpression {
  type: 'BinaryExpression';
  left: Expression;
  operator: BinaryOperator;
  right: Expression;
  0: number;
}

export interface PrivateInExpression {
  type: 'BinaryExpression';
  left: PrivateIdentifier;
  operator: 'in';
  right: Expression;
  0: number;
}

export interface LogicalExpression {
  type: 'LogicalExpression';
  left: Expression;
  operator: LogicalOperator;
  right: Expression;
  0: number;
}

export interface ConditionalExpression {
  type: 'ConditionalExpression';
  test: Expression;
  consequent: Expression;
  alternate: Expression;
  0: number;
}

export interface AssignmentExpression {
  type: 'AssignmentExpression';
  operator: AssignmentOperator;
  left: AssignmentTarget;
  right: Expression;
  0: number;
}

export type AssignmentTarget = SimpleAssignmentTarget | AssignmentTargetPattern;

export type SimpleAssignmentTarget =
  | IdentifierReference
  | TSAsExpression
  | TSSatisfiesExpression
  | TSNonNullExpression
  | TSTypeAssertion
  | MemberExpression;

export type AssignmentTargetPattern = ArrayAssignmentTarget | ObjectAssignmentTarget;

export interface ArrayAssignmentTarget {
  type: 'ArrayPattern';
  decorators?: [];
  elements: Array<AssignmentTargetMaybeDefault | AssignmentTargetRest | null>;
  optional?: false;
  typeAnnotation?: null;
  0: number;
}

export interface ObjectAssignmentTarget {
  type: 'ObjectPattern';
  decorators?: [];
  properties: Array<AssignmentTargetProperty | AssignmentTargetRest>;
  optional?: false;
  typeAnnotation?: null;
  0: number;
}

export interface AssignmentTargetRest {
  type: 'RestElement';
  decorators?: [];
  argument: AssignmentTarget;
  optional?: false;
  typeAnnotation?: null;
  value?: null;
  0: number;
}

export type AssignmentTargetMaybeDefault = AssignmentTargetWithDefault | AssignmentTarget;

export interface AssignmentTargetWithDefault {
  type: 'AssignmentPattern';
  decorators?: [];
  left: AssignmentTarget;
  right: Expression;
  optional?: false;
  typeAnnotation?: null;
  0: number;
}

export type AssignmentTargetProperty = AssignmentTargetPropertyIdentifier | AssignmentTargetPropertyProperty;

export interface AssignmentTargetPropertyIdentifier {
  type: 'Property';
  kind: 'init';
  key: IdentifierReference;
  value: IdentifierReference | AssignmentTargetWithDefault;
  method: false;
  shorthand: true;
  computed: false;
  optional?: false;
  0: number;
}

export interface AssignmentTargetPropertyProperty {
  type: 'Property';
  kind: 'init';
  key: PropertyKey;
  value: AssignmentTargetMaybeDefault;
  method: false;
  shorthand: false;
  computed: boolean;
  optional?: false;
  0: number;
}

export interface SequenceExpression {
  type: 'SequenceExpression';
  expressions: Array<Expression>;
  0: number;
}

export interface Super {
  type: 'Super';
  0: number;
}

export interface AwaitExpression {
  type: 'AwaitExpression';
  argument: Expression;
  0: number;
}

export interface ChainExpression {
  type: 'ChainExpression';
  expression: ChainElement;
  0: number;
}

export type ChainElement = CallExpression | TSNonNullExpression | MemberExpression;

export interface ParenthesizedExpression {
  type: 'ParenthesizedExpression';
  expression: Expression;
  0: number;
}

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
  | Declaration
  | ModuleDeclaration;

export interface Directive {
  type: 'ExpressionStatement';
  expression: StringLiteral;
  directive: string;
  0: number;
}

export interface Hashbang {
  type: 'Hashbang';
  value: string;
  0: number;
}

export interface BlockStatement {
  type: 'BlockStatement';
  body: Array<Statement>;
  0: number;
}

export type Declaration =
  | VariableDeclaration
  | Function
  | Class
  | TSTypeAliasDeclaration
  | TSInterfaceDeclaration
  | TSEnumDeclaration
  | TSModuleDeclaration
  | TSImportEqualsDeclaration;

export interface VariableDeclaration {
  type: 'VariableDeclaration';
  kind: VariableDeclarationKind;
  declarations: Array<VariableDeclarator>;
  declare?: boolean;
  0: number;
}

export type VariableDeclarationKind = 'var' | 'let' | 'const' | 'using' | 'await using';

export interface VariableDeclarator {
  type: 'VariableDeclarator';
  id: BindingPattern;
  init: Expression | null;
  definite?: boolean;
  0: number;
}

export interface EmptyStatement {
  type: 'EmptyStatement';
  0: number;
}

export interface ExpressionStatement {
  type: 'ExpressionStatement';
  expression: Expression;
  directive?: string | null;
  0: number;
}

export interface IfStatement {
  type: 'IfStatement';
  test: Expression;
  consequent: Statement;
  alternate: Statement | null;
  0: number;
}

export interface DoWhileStatement {
  type: 'DoWhileStatement';
  body: Statement;
  test: Expression;
  0: number;
}

export interface WhileStatement {
  type: 'WhileStatement';
  test: Expression;
  body: Statement;
  0: number;
}

export interface ForStatement {
  type: 'ForStatement';
  init: ForStatementInit | null;
  test: Expression | null;
  update: Expression | null;
  body: Statement;
  0: number;
}

export type ForStatementInit = VariableDeclaration | Expression;

export interface ForInStatement {
  type: 'ForInStatement';
  left: ForStatementLeft;
  right: Expression;
  body: Statement;
  0: number;
}

export type ForStatementLeft = VariableDeclaration | AssignmentTarget;

export interface ForOfStatement {
  type: 'ForOfStatement';
  await: boolean;
  left: ForStatementLeft;
  right: Expression;
  body: Statement;
  0: number;
}

export interface ContinueStatement {
  type: 'ContinueStatement';
  label: LabelIdentifier | null;
  0: number;
}

export interface BreakStatement {
  type: 'BreakStatement';
  label: LabelIdentifier | null;
  0: number;
}

export interface ReturnStatement {
  type: 'ReturnStatement';
  argument: Expression | null;
  0: number;
}

export interface WithStatement {
  type: 'WithStatement';
  object: Expression;
  body: Statement;
  0: number;
}

export interface SwitchStatement {
  type: 'SwitchStatement';
  discriminant: Expression;
  cases: Array<SwitchCase>;
  0: number;
}

export interface SwitchCase {
  type: 'SwitchCase';
  test: Expression | null;
  consequent: Array<Statement>;
  0: number;
}

export interface LabeledStatement {
  type: 'LabeledStatement';
  label: LabelIdentifier;
  body: Statement;
  0: number;
}

export interface ThrowStatement {
  type: 'ThrowStatement';
  argument: Expression;
  0: number;
}

export interface TryStatement {
  type: 'TryStatement';
  block: BlockStatement;
  handler: CatchClause | null;
  finalizer: BlockStatement | null;
  0: number;
}

export interface CatchClause {
  type: 'CatchClause';
  param: BindingPattern | null;
  body: BlockStatement;
  0: number;
}

export interface DebuggerStatement {
  type: 'DebuggerStatement';
  0: number;
}

export type BindingPattern =
  & ({
    optional?: boolean;
    typeAnnotation?: TSTypeAnnotation | null;
  })
  & (BindingIdentifier | ObjectPattern | ArrayPattern | AssignmentPattern);

export type BindingPatternKind = BindingIdentifier | ObjectPattern | ArrayPattern | AssignmentPattern;

export interface AssignmentPattern {
  type: 'AssignmentPattern';
  decorators?: [];
  left: BindingPattern;
  right: Expression;
  optional?: false;
  typeAnnotation?: null;
  0: number;
}

export interface ObjectPattern {
  type: 'ObjectPattern';
  decorators?: [];
  properties: Array<BindingProperty | BindingRestElement>;
  optional?: false;
  typeAnnotation?: null;
  0: number;
}

export interface BindingProperty {
  type: 'Property';
  kind: 'init';
  key: PropertyKey;
  value: BindingPattern;
  method: false;
  shorthand: boolean;
  computed: boolean;
  optional?: false;
  0: number;
}

export interface ArrayPattern {
  type: 'ArrayPattern';
  decorators?: [];
  elements: Array<BindingPattern | BindingRestElement | null>;
  optional?: false;
  typeAnnotation?: null;
  0: number;
}

export interface BindingRestElement {
  type: 'RestElement';
  decorators?: [];
  argument: BindingPattern;
  optional?: false;
  typeAnnotation?: null;
  value?: null;
  0: number;
}

export interface Function {
  type: FunctionType;
  id: BindingIdentifier | null;
  generator: boolean;
  async: boolean;
  declare?: boolean;
  typeParameters?: TSTypeParameterDeclaration | null;
  params: ParamPattern[];
  returnType?: TSTypeAnnotation | null;
  body: FunctionBody | null;
  expression: false;
  0: number;
}

export type ParamPattern = FormalParameter | TSParameterProperty | FormalParameterRest;

export type FunctionType =
  | 'FunctionDeclaration'
  | 'FunctionExpression'
  | 'TSDeclareFunction'
  | 'TSEmptyBodyFunctionExpression';

export interface FormalParameterRest extends Span {
  type: 'RestElement';
  argument: BindingPatternKind;
  decorators?: [];
  optional?: boolean;
  typeAnnotation?: TSTypeAnnotation | null;
  value?: null;
}

export type FormalParameter =
  & ({
    decorators?: Array<Decorator>;
  })
  & BindingPattern;

export interface TSParameterProperty extends Span {
  type: 'TSParameterProperty';
  accessibility: TSAccessibility | null;
  decorators: Array<Decorator>;
  override: boolean;
  parameter: FormalParameter;
  readonly: boolean;
  static: boolean;
}

export interface FunctionBody {
  type: 'BlockStatement';
  body: Array<Directive | Statement>;
  0: number;
}

export interface ArrowFunctionExpression {
  type: 'ArrowFunctionExpression';
  expression: boolean;
  async: boolean;
  typeParameters?: TSTypeParameterDeclaration | null;
  params: ParamPattern[];
  returnType?: TSTypeAnnotation | null;
  body: FunctionBody | Expression;
  id: null;
  generator: false;
  0: number;
}

export interface YieldExpression {
  type: 'YieldExpression';
  delegate: boolean;
  argument: Expression | null;
  0: number;
}

export interface Class {
  type: ClassType;
  decorators: Array<Decorator>;
  id: BindingIdentifier | null;
  typeParameters?: TSTypeParameterDeclaration | null;
  superClass: Expression | null;
  superTypeArguments?: TSTypeParameterInstantiation | null;
  implements?: Array<TSClassImplements>;
  body: ClassBody;
  abstract?: boolean;
  declare?: boolean;
  0: number;
}

export type ClassType = 'ClassDeclaration' | 'ClassExpression';

export interface ClassBody {
  type: 'ClassBody';
  body: Array<ClassElement>;
  0: number;
}

export type ClassElement = StaticBlock | MethodDefinition | PropertyDefinition | AccessorProperty | TSIndexSignature;

export interface MethodDefinition {
  type: MethodDefinitionType;
  decorators: Array<Decorator>;
  key: PropertyKey;
  value: Function;
  kind: MethodDefinitionKind;
  computed: boolean;
  static: boolean;
  override?: boolean;
  optional?: boolean;
  accessibility?: TSAccessibility | null;
  0: number;
}

export type MethodDefinitionType = 'MethodDefinition' | 'TSAbstractMethodDefinition';

export interface PropertyDefinition {
  type: PropertyDefinitionType;
  decorators: Array<Decorator>;
  key: PropertyKey;
  typeAnnotation?: TSTypeAnnotation | null;
  value: Expression | null;
  computed: boolean;
  static: boolean;
  declare?: boolean;
  override?: boolean;
  optional?: boolean;
  definite?: boolean;
  readonly?: boolean;
  accessibility?: TSAccessibility | null;
  0: number;
}

export type PropertyDefinitionType = 'PropertyDefinition' | 'TSAbstractPropertyDefinition';

export type MethodDefinitionKind = 'constructor' | 'method' | 'get' | 'set';

export interface PrivateIdentifier {
  type: 'PrivateIdentifier';
  name: string;
  0: number;
}

export interface StaticBlock {
  type: 'StaticBlock';
  body: Array<Statement>;
  0: number;
}

export type ModuleDeclaration =
  | ImportDeclaration
  | ExportAllDeclaration
  | ExportDefaultDeclaration
  | ExportNamedDeclaration
  | TSExportAssignment
  | TSNamespaceExportDeclaration;

export type AccessorPropertyType = 'AccessorProperty' | 'TSAbstractAccessorProperty';

export interface AccessorProperty {
  type: AccessorPropertyType;
  decorators: Array<Decorator>;
  key: PropertyKey;
  typeAnnotation?: TSTypeAnnotation | null;
  value: Expression | null;
  computed: boolean;
  static: boolean;
  override?: boolean;
  definite?: boolean;
  accessibility?: TSAccessibility | null;
  declare?: false;
  optional?: false;
  readonly?: false;
  0: number;
}

export interface ImportExpression {
  type: 'ImportExpression';
  source: Expression;
  options: Expression | null;
  phase: ImportPhase | null;
  0: number;
}

export interface ImportDeclaration {
  type: 'ImportDeclaration';
  specifiers: Array<ImportDeclarationSpecifier>;
  source: StringLiteral;
  phase: ImportPhase | null;
  attributes: Array<ImportAttribute>;
  importKind?: ImportOrExportKind;
  0: number;
}

export type ImportPhase = 'source' | 'defer';

export type ImportDeclarationSpecifier = ImportSpecifier | ImportDefaultSpecifier | ImportNamespaceSpecifier;

export interface ImportSpecifier {
  type: 'ImportSpecifier';
  imported: ModuleExportName;
  local: BindingIdentifier;
  importKind?: ImportOrExportKind;
  0: number;
}

export interface ImportDefaultSpecifier {
  type: 'ImportDefaultSpecifier';
  local: BindingIdentifier;
  0: number;
}

export interface ImportNamespaceSpecifier {
  type: 'ImportNamespaceSpecifier';
  local: BindingIdentifier;
  0: number;
}

export interface ImportAttribute {
  type: 'ImportAttribute';
  key: ImportAttributeKey;
  value: StringLiteral;
  0: number;
}

export type ImportAttributeKey = IdentifierName | StringLiteral;

export interface ExportNamedDeclaration {
  type: 'ExportNamedDeclaration';
  declaration: Declaration | null;
  specifiers: Array<ExportSpecifier>;
  source: StringLiteral | null;
  exportKind?: ImportOrExportKind;
  attributes: Array<ImportAttribute>;
  0: number;
}

export interface ExportDefaultDeclaration {
  type: 'ExportDefaultDeclaration';
  declaration: ExportDefaultDeclarationKind;
  exportKind?: 'value';
  0: number;
}

export interface ExportAllDeclaration {
  type: 'ExportAllDeclaration';
  exported: ModuleExportName | null;
  source: StringLiteral;
  attributes: Array<ImportAttribute>;
  exportKind?: ImportOrExportKind;
  0: number;
}

export interface ExportSpecifier {
  type: 'ExportSpecifier';
  local: ModuleExportName;
  exported: ModuleExportName;
  exportKind?: ImportOrExportKind;
  0: number;
}

export type ExportDefaultDeclarationKind = Function | Class | TSInterfaceDeclaration | Expression;

export type ModuleExportName = IdentifierName | IdentifierReference | StringLiteral;

export interface V8IntrinsicExpression {
  type: 'V8IntrinsicExpression';
  name: IdentifierName;
  arguments: Array<Argument>;
  0: number;
}

export interface BooleanLiteral {
  type: 'Literal';
  value: boolean;
  raw: string | null;
  0: number;
}

export interface NullLiteral {
  type: 'Literal';
  value: null;
  raw: 'null' | null;
  0: number;
}

export interface NumericLiteral {
  type: 'Literal';
  value: number;
  raw: string | null;
  0: number;
}

export interface StringLiteral {
  type: 'Literal';
  value: string;
  raw: string | null;
  0: number;
}

export interface BigIntLiteral {
  type: 'Literal';
  value: bigint;
  raw: string | null;
  bigint: string;
  0: number;
}

export interface RegExpLiteral {
  type: 'Literal';
  value: RegExp | null;
  raw: string | null;
  regex: { pattern: string; flags: string };
  0: number;
}

export interface JSXElement {
  type: 'JSXElement';
  openingElement: JSXOpeningElement;
  children: Array<JSXChild>;
  closingElement: JSXClosingElement | null;
  0: number;
}

export interface JSXOpeningElement {
  type: 'JSXOpeningElement';
  name: JSXElementName;
  typeArguments?: TSTypeParameterInstantiation | null;
  attributes: Array<JSXAttributeItem>;
  selfClosing: boolean;
  0: number;
}

export interface JSXClosingElement {
  type: 'JSXClosingElement';
  name: JSXElementName;
  0: number;
}

export interface JSXFragment {
  type: 'JSXFragment';
  openingFragment: JSXOpeningFragment;
  children: Array<JSXChild>;
  closingFragment: JSXClosingFragment;
  0: number;
}

export interface JSXOpeningFragment {
  type: 'JSXOpeningFragment';
  attributes?: [];
  selfClosing?: false;
  0: number;
}

export interface JSXClosingFragment {
  type: 'JSXClosingFragment';
  0: number;
}

export type JSXElementName = JSXIdentifier | JSXNamespacedName | JSXMemberExpression;

export interface JSXNamespacedName {
  type: 'JSXNamespacedName';
  namespace: JSXIdentifier;
  name: JSXIdentifier;
  0: number;
}

export interface JSXMemberExpression {
  type: 'JSXMemberExpression';
  object: JSXMemberExpressionObject;
  property: JSXIdentifier;
  0: number;
}

export type JSXMemberExpressionObject = JSXIdentifier | JSXMemberExpression;

export interface JSXExpressionContainer {
  type: 'JSXExpressionContainer';
  expression: JSXExpression;
  0: number;
}

export type JSXExpression = JSXEmptyExpression | Expression;

export interface JSXEmptyExpression {
  type: 'JSXEmptyExpression';
  0: number;
}

export type JSXAttributeItem = JSXAttribute | JSXSpreadAttribute;

export interface JSXAttribute {
  type: 'JSXAttribute';
  name: JSXAttributeName;
  value: JSXAttributeValue | null;
  0: number;
}

export interface JSXSpreadAttribute {
  type: 'JSXSpreadAttribute';
  argument: Expression;
  0: number;
}

export type JSXAttributeName = JSXIdentifier | JSXNamespacedName;

export type JSXAttributeValue = StringLiteral | JSXExpressionContainer | JSXElement | JSXFragment;

export interface JSXIdentifier {
  type: 'JSXIdentifier';
  name: string;
  0: number;
}

export type JSXChild = JSXText | JSXElement | JSXFragment | JSXExpressionContainer | JSXSpreadChild;

export interface JSXSpreadChild {
  type: 'JSXSpreadChild';
  expression: Expression;
  0: number;
}

export interface JSXText {
  type: 'JSXText';
  value: string;
  raw: string | null;
  0: number;
}

export interface TSThisParameter {
  type: 'Identifier';
  decorators: [];
  name: 'this';
  optional: false;
  typeAnnotation: TSTypeAnnotation | null;
  0: number;
}

export interface TSEnumDeclaration {
  type: 'TSEnumDeclaration';
  id: BindingIdentifier;
  body: TSEnumBody;
  const: boolean;
  declare: boolean;
  0: number;
}

export interface TSEnumBody {
  type: 'TSEnumBody';
  members: Array<TSEnumMember>;
  0: number;
}

export interface TSEnumMember {
  type: 'TSEnumMember';
  id: TSEnumMemberName;
  initializer: Expression | null;
  computed: boolean;
  0: number;
}

export type TSEnumMemberName = IdentifierName | StringLiteral | TemplateLiteral;

export interface TSTypeAnnotation {
  type: 'TSTypeAnnotation';
  typeAnnotation: TSType;
  0: number;
}

export interface TSLiteralType {
  type: 'TSLiteralType';
  literal: TSLiteral;
  0: number;
}

export type TSLiteral =
  | BooleanLiteral
  | NumericLiteral
  | BigIntLiteral
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

export interface TSConditionalType {
  type: 'TSConditionalType';
  checkType: TSType;
  extendsType: TSType;
  trueType: TSType;
  falseType: TSType;
  0: number;
}

export interface TSUnionType {
  type: 'TSUnionType';
  types: Array<TSType>;
  0: number;
}

export interface TSIntersectionType {
  type: 'TSIntersectionType';
  types: Array<TSType>;
  0: number;
}

export interface TSParenthesizedType {
  type: 'TSParenthesizedType';
  typeAnnotation: TSType;
  0: number;
}

export interface TSTypeOperator {
  type: 'TSTypeOperator';
  operator: TSTypeOperatorOperator;
  typeAnnotation: TSType;
  0: number;
}

export type TSTypeOperatorOperator = 'keyof' | 'unique' | 'readonly';

export interface TSArrayType {
  type: 'TSArrayType';
  elementType: TSType;
  0: number;
}

export interface TSIndexedAccessType {
  type: 'TSIndexedAccessType';
  objectType: TSType;
  indexType: TSType;
  0: number;
}

export interface TSTupleType {
  type: 'TSTupleType';
  elementTypes: Array<TSTupleElement>;
  0: number;
}

export interface TSNamedTupleMember {
  type: 'TSNamedTupleMember';
  label: IdentifierName;
  elementType: TSTupleElement;
  optional: boolean;
  0: number;
}

export interface TSOptionalType {
  type: 'TSOptionalType';
  typeAnnotation: TSType;
  0: number;
}

export interface TSRestType {
  type: 'TSRestType';
  typeAnnotation: TSType;
  0: number;
}

export type TSTupleElement = TSOptionalType | TSRestType | TSType;

export interface TSAnyKeyword {
  type: 'TSAnyKeyword';
  0: number;
}

export interface TSStringKeyword {
  type: 'TSStringKeyword';
  0: number;
}

export interface TSBooleanKeyword {
  type: 'TSBooleanKeyword';
  0: number;
}

export interface TSNumberKeyword {
  type: 'TSNumberKeyword';
  0: number;
}

export interface TSNeverKeyword {
  type: 'TSNeverKeyword';
  0: number;
}

export interface TSIntrinsicKeyword {
  type: 'TSIntrinsicKeyword';
  0: number;
}

export interface TSUnknownKeyword {
  type: 'TSUnknownKeyword';
  0: number;
}

export interface TSNullKeyword {
  type: 'TSNullKeyword';
  0: number;
}

export interface TSUndefinedKeyword {
  type: 'TSUndefinedKeyword';
  0: number;
}

export interface TSVoidKeyword {
  type: 'TSVoidKeyword';
  0: number;
}

export interface TSSymbolKeyword {
  type: 'TSSymbolKeyword';
  0: number;
}

export interface TSThisType {
  type: 'TSThisType';
  0: number;
}

export interface TSObjectKeyword {
  type: 'TSObjectKeyword';
  0: number;
}

export interface TSBigIntKeyword {
  type: 'TSBigIntKeyword';
  0: number;
}

export interface TSTypeReference {
  type: 'TSTypeReference';
  typeName: TSTypeName;
  typeArguments: TSTypeParameterInstantiation | null;
  0: number;
}

export type TSTypeName = IdentifierReference | TSQualifiedName | ThisExpression;

export interface TSQualifiedName {
  type: 'TSQualifiedName';
  left: TSTypeName;
  right: IdentifierName;
  0: number;
}

export interface TSTypeParameterInstantiation {
  type: 'TSTypeParameterInstantiation';
  params: Array<TSType>;
  0: number;
}

export interface TSTypeParameter {
  type: 'TSTypeParameter';
  name: BindingIdentifier;
  constraint: TSType | null;
  default: TSType | null;
  in: boolean;
  out: boolean;
  const: boolean;
  0: number;
}

export interface TSTypeParameterDeclaration {
  type: 'TSTypeParameterDeclaration';
  params: Array<TSTypeParameter>;
  0: number;
}

export interface TSTypeAliasDeclaration {
  type: 'TSTypeAliasDeclaration';
  id: BindingIdentifier;
  typeParameters: TSTypeParameterDeclaration | null;
  typeAnnotation: TSType;
  declare: boolean;
  0: number;
}

export type TSAccessibility = 'private' | 'protected' | 'public';

export interface TSClassImplements {
  type: 'TSClassImplements';
  expression: IdentifierReference | ThisExpression | MemberExpression;
  typeArguments: TSTypeParameterInstantiation | null;
  0: number;
}

export interface TSInterfaceDeclaration {
  type: 'TSInterfaceDeclaration';
  id: BindingIdentifier;
  typeParameters: TSTypeParameterDeclaration | null;
  extends: Array<TSInterfaceHeritage>;
  body: TSInterfaceBody;
  declare: boolean;
  0: number;
}

export interface TSInterfaceBody {
  type: 'TSInterfaceBody';
  body: Array<TSSignature>;
  0: number;
}

export interface TSPropertySignature {
  type: 'TSPropertySignature';
  computed: boolean;
  optional: boolean;
  readonly: boolean;
  key: PropertyKey;
  typeAnnotation: TSTypeAnnotation | null;
  accessibility: null;
  static: false;
  0: number;
}

export type TSSignature =
  | TSIndexSignature
  | TSPropertySignature
  | TSCallSignatureDeclaration
  | TSConstructSignatureDeclaration
  | TSMethodSignature;

export interface TSIndexSignature {
  type: 'TSIndexSignature';
  parameters: Array<TSIndexSignatureName>;
  typeAnnotation: TSTypeAnnotation;
  readonly: boolean;
  static: boolean;
  accessibility: null;
  0: number;
}

export interface TSCallSignatureDeclaration {
  type: 'TSCallSignatureDeclaration';
  typeParameters: TSTypeParameterDeclaration | null;
  params: ParamPattern[];
  returnType: TSTypeAnnotation | null;
  0: number;
}

export type TSMethodSignatureKind = 'method' | 'get' | 'set';

export interface TSMethodSignature {
  type: 'TSMethodSignature';
  key: PropertyKey;
  computed: boolean;
  optional: boolean;
  kind: TSMethodSignatureKind;
  typeParameters: TSTypeParameterDeclaration | null;
  params: ParamPattern[];
  returnType: TSTypeAnnotation | null;
  accessibility: null;
  readonly: false;
  static: false;
  0: number;
}

export interface TSConstructSignatureDeclaration {
  type: 'TSConstructSignatureDeclaration';
  typeParameters: TSTypeParameterDeclaration | null;
  params: ParamPattern[];
  returnType: TSTypeAnnotation | null;
  0: number;
}

export interface TSIndexSignatureName {
  type: 'Identifier';
  decorators: [];
  name: string;
  optional: false;
  typeAnnotation: TSTypeAnnotation;
  0: number;
}

export interface TSInterfaceHeritage {
  type: 'TSInterfaceHeritage';
  expression: Expression;
  typeArguments: TSTypeParameterInstantiation | null;
  0: number;
}

export interface TSTypePredicate {
  type: 'TSTypePredicate';
  parameterName: TSTypePredicateName;
  asserts: boolean;
  typeAnnotation: TSTypeAnnotation | null;
  0: number;
}

export type TSTypePredicateName = IdentifierName | TSThisType;

export interface TSModuleDeclaration {
  type: 'TSModuleDeclaration';
  id: BindingIdentifier | StringLiteral | TSQualifiedName;
  body: TSModuleBlock | null;
  kind: TSModuleDeclarationKind;
  declare: boolean;
  global: boolean;
  0: number;
}

export type TSModuleDeclarationKind = 'global' | 'module' | 'namespace';

export interface TSModuleBlock {
  type: 'TSModuleBlock';
  body: Array<Directive | Statement>;
  0: number;
}

export interface TSTypeLiteral {
  type: 'TSTypeLiteral';
  members: Array<TSSignature>;
  0: number;
}

export interface TSInferType {
  type: 'TSInferType';
  typeParameter: TSTypeParameter;
  0: number;
}

export interface TSTypeQuery {
  type: 'TSTypeQuery';
  exprName: TSTypeQueryExprName;
  typeArguments: TSTypeParameterInstantiation | null;
  0: number;
}

export type TSTypeQueryExprName = TSImportType | TSTypeName;

export interface TSImportType {
  type: 'TSImportType';
  argument: TSType;
  options: ObjectExpression | null;
  qualifier: TSImportTypeQualifier | null;
  typeArguments: TSTypeParameterInstantiation | null;
  0: number;
}

export type TSImportTypeQualifier = IdentifierName | TSImportTypeQualifiedName;

export interface TSImportTypeQualifiedName {
  type: 'TSQualifiedName';
  left: TSImportTypeQualifier;
  right: IdentifierName;
  0: number;
}

export interface TSFunctionType {
  type: 'TSFunctionType';
  typeParameters: TSTypeParameterDeclaration | null;
  params: ParamPattern[];
  returnType: TSTypeAnnotation;
  0: number;
}

export interface TSConstructorType {
  type: 'TSConstructorType';
  abstract: boolean;
  typeParameters: TSTypeParameterDeclaration | null;
  params: ParamPattern[];
  returnType: TSTypeAnnotation;
  0: number;
}

export interface TSMappedType {
  type: 'TSMappedType';
  key: TSTypeParameter['name'];
  constraint: TSTypeParameter['constraint'];
  nameType: TSType | null;
  typeAnnotation: TSType | null;
  optional: TSMappedTypeModifierOperator | false;
  readonly: TSMappedTypeModifierOperator | null;
  0: number;
}

export type TSMappedTypeModifierOperator = true | '+' | '-';

export interface TSTemplateLiteralType {
  type: 'TSTemplateLiteralType';
  quasis: Array<TemplateElement>;
  types: Array<TSType>;
  0: number;
}

export interface TSAsExpression {
  type: 'TSAsExpression';
  expression: Expression;
  typeAnnotation: TSType;
  0: number;
}

export interface TSSatisfiesExpression {
  type: 'TSSatisfiesExpression';
  expression: Expression;
  typeAnnotation: TSType;
  0: number;
}

export interface TSTypeAssertion {
  type: 'TSTypeAssertion';
  typeAnnotation: TSType;
  expression: Expression;
  0: number;
}

export interface TSImportEqualsDeclaration {
  type: 'TSImportEqualsDeclaration';
  id: BindingIdentifier;
  moduleReference: TSModuleReference;
  importKind: ImportOrExportKind;
  0: number;
}

export type TSModuleReference = TSExternalModuleReference | TSTypeName;

export interface TSExternalModuleReference {
  type: 'TSExternalModuleReference';
  expression: StringLiteral;
  0: number;
}

export interface TSNonNullExpression {
  type: 'TSNonNullExpression';
  expression: Expression;
  0: number;
}

export interface Decorator {
  type: 'Decorator';
  expression: Expression;
  0: number;
}

export interface TSExportAssignment {
  type: 'TSExportAssignment';
  expression: Expression;
  0: number;
}

export interface TSNamespaceExportDeclaration {
  type: 'TSNamespaceExportDeclaration';
  id: IdentifierName;
  0: number;
}

export interface TSInstantiationExpression {
  type: 'TSInstantiationExpression';
  expression: Expression;
  typeArguments: TSTypeParameterInstantiation;
  0: number;
}

export type ImportOrExportKind = 'value' | 'type';

export interface JSDocNullableType {
  type: 'TSJSDocNullableType';
  typeAnnotation: TSType;
  postfix: boolean;
  0: number;
}

export interface JSDocNonNullableType {
  type: 'TSJSDocNonNullableType';
  typeAnnotation: TSType;
  postfix: boolean;
  0: number;
}

export interface JSDocUnknownType {
  type: 'TSJSDocUnknownType';
  0: number;
}

export type AssignmentOperator =
  | '='
  | '+='
  | '-='
  | '*='
  | '/='
  | '%='
  | '**='
  | '<<='
  | '>>='
  | '>>>='
  | '|='
  | '^='
  | '&='
  | '||='
  | '&&='
  | '??=';

export type BinaryOperator =
  | '=='
  | '!='
  | '==='
  | '!=='
  | '<'
  | '<='
  | '>'
  | '>='
  | '+'
  | '-'
  | '*'
  | '/'
  | '%'
  | '**'
  | '<<'
  | '>>'
  | '>>>'
  | '|'
  | '^'
  | '&'
  | 'in'
  | 'instanceof';

export type LogicalOperator = '||' | '&&' | '??';

export type UnaryOperator = '+' | '-' | '!' | '~' | 'typeof' | 'void' | 'delete';

export type UpdateOperator = '++' | '--';

export type ModuleKind = 'script' | 'module';

export type Node =
  | Program
  | IdentifierName
  | IdentifierReference
  | BindingIdentifier
  | LabelIdentifier
  | ThisExpression
  | ArrayExpression
  | ObjectExpression
  | ObjectProperty
  | TemplateLiteral
  | TaggedTemplateExpression
  | TemplateElement
  | ComputedMemberExpression
  | StaticMemberExpression
  | PrivateFieldExpression
  | CallExpression
  | NewExpression
  | MetaProperty
  | SpreadElement
  | UpdateExpression
  | UnaryExpression
  | BinaryExpression
  | PrivateInExpression
  | LogicalExpression
  | ConditionalExpression
  | AssignmentExpression
  | ArrayAssignmentTarget
  | ObjectAssignmentTarget
  | AssignmentTargetRest
  | AssignmentTargetWithDefault
  | AssignmentTargetPropertyIdentifier
  | AssignmentTargetPropertyProperty
  | SequenceExpression
  | Super
  | AwaitExpression
  | ChainExpression
  | ParenthesizedExpression
  | Directive
  | Hashbang
  | BlockStatement
  | VariableDeclaration
  | VariableDeclarator
  | EmptyStatement
  | ExpressionStatement
  | IfStatement
  | DoWhileStatement
  | WhileStatement
  | ForStatement
  | ForInStatement
  | ForOfStatement
  | ContinueStatement
  | BreakStatement
  | ReturnStatement
  | WithStatement
  | SwitchStatement
  | SwitchCase
  | LabeledStatement
  | ThrowStatement
  | TryStatement
  | CatchClause
  | DebuggerStatement
  | AssignmentPattern
  | ObjectPattern
  | BindingProperty
  | ArrayPattern
  | BindingRestElement
  | Function
  | FunctionBody
  | ArrowFunctionExpression
  | YieldExpression
  | Class
  | ClassBody
  | MethodDefinition
  | PropertyDefinition
  | PrivateIdentifier
  | StaticBlock
  | AccessorProperty
  | ImportExpression
  | ImportDeclaration
  | ImportSpecifier
  | ImportDefaultSpecifier
  | ImportNamespaceSpecifier
  | ImportAttribute
  | ExportNamedDeclaration
  | ExportDefaultDeclaration
  | ExportAllDeclaration
  | ExportSpecifier
  | V8IntrinsicExpression
  | BooleanLiteral
  | NullLiteral
  | NumericLiteral
  | StringLiteral
  | BigIntLiteral
  | RegExpLiteral
  | JSXElement
  | JSXOpeningElement
  | JSXClosingElement
  | JSXFragment
  | JSXOpeningFragment
  | JSXClosingFragment
  | JSXNamespacedName
  | JSXMemberExpression
  | JSXExpressionContainer
  | JSXEmptyExpression
  | JSXAttribute
  | JSXSpreadAttribute
  | JSXIdentifier
  | JSXSpreadChild
  | JSXText
  | TSThisParameter
  | TSEnumDeclaration
  | TSEnumBody
  | TSEnumMember
  | TSTypeAnnotation
  | TSLiteralType
  | TSConditionalType
  | TSUnionType
  | TSIntersectionType
  | TSParenthesizedType
  | TSTypeOperator
  | TSArrayType
  | TSIndexedAccessType
  | TSTupleType
  | TSNamedTupleMember
  | TSOptionalType
  | TSRestType
  | TSAnyKeyword
  | TSStringKeyword
  | TSBooleanKeyword
  | TSNumberKeyword
  | TSNeverKeyword
  | TSIntrinsicKeyword
  | TSUnknownKeyword
  | TSNullKeyword
  | TSUndefinedKeyword
  | TSVoidKeyword
  | TSSymbolKeyword
  | TSThisType
  | TSObjectKeyword
  | TSBigIntKeyword
  | TSTypeReference
  | TSQualifiedName
  | TSTypeParameterInstantiation
  | TSTypeParameter
  | TSTypeParameterDeclaration
  | TSTypeAliasDeclaration
  | TSClassImplements
  | TSInterfaceDeclaration
  | TSInterfaceBody
  | TSPropertySignature
  | TSIndexSignature
  | TSCallSignatureDeclaration
  | TSMethodSignature
  | TSConstructSignatureDeclaration
  | TSIndexSignatureName
  | TSInterfaceHeritage
  | TSTypePredicate
  | TSModuleDeclaration
  | TSModuleBlock
  | TSTypeLiteral
  | TSInferType
  | TSTypeQuery
  | TSImportType
  | TSImportTypeQualifiedName
  | TSFunctionType
  | TSConstructorType
  | TSMappedType
  | TSTemplateLiteralType
  | TSAsExpression
  | TSSatisfiesExpression
  | TSTypeAssertion
  | TSImportEqualsDeclaration
  | TSExternalModuleReference
  | TSNonNullExpression
  | Decorator
  | TSExportAssignment
  | TSNamespaceExportDeclaration
  | TSInstantiationExpression
  | JSDocNullableType
  | JSDocNonNullableType
  | JSDocUnknownType
  | ParamPattern;
