// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit `tasks/ast_tools/src/generators/typescript.rs`.

export interface Comment {
  type: "Line" | "Block";
  value: string;
  start: number;
  end: number;
  range?: [number, number];
  /** Original syntax: 0 line, 1 single-line block, 2 multiline block, 3 HTML close, 4 HTML open. */
  kind?: 0 | 1 | 2 | 3 | 4;
  /** Newline flags: bit 0 before the comment, bit 1 after it. */
  newlines?: number;
  /** Original annotation classification from Oxc's CommentContent. */
  content?: number;
  /** Native owner folded into this ESTree node. Positions use UTF-16 offsets. */
  container?: {
    kind: string;
    placement: "leading" | "trailing" | "dangling";
    start: number;
    end: number;
  } | null;
}
export interface NodeComments {
  leading: Comment[] | null;
  trailing: Comment[] | null;
  dangling: Comment[] | null;
}
export interface Program extends Span {
  comments?: NodeComments | null;
  type: "Program";
  body: Array<Directive | Statement>;
  sourceType: ModuleKind;
  hashbang: Hashbang | null;
  parent?: null;
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
  | MetaProperty
  | JSXElement
  | JSXFragment
  | TSAsExpression
  | TSSatisfiesExpression
  | TSTypeAssertion
  | TSNonNullExpression
  | TSInstantiationExpression
  | V8IntrinsicExpression
  | MemberExpression;

export interface IdentifierName extends Span {
  comments?: NodeComments | null;
  type: "Identifier";
  decorators?: [];
  name: string;
  optional?: false;
  typeAnnotation?: null;
  parent?: Node;
}

export interface IdentifierReference extends Span {
  comments?: NodeComments | null;
  type: "Identifier";
  decorators?: [];
  name: string;
  optional?: false;
  typeAnnotation?: null;
  parent?: Node;
}

export interface BindingIdentifier extends Span {
  comments?: NodeComments | null;
  type: "Identifier";
  decorators?: [];
  name: string;
  optional?: false;
  typeAnnotation?: TSTypeAnnotation | null;
  parent?: Node;
}

export interface LabelIdentifier extends Span {
  comments?: NodeComments | null;
  type: "Identifier";
  decorators?: [];
  name: string;
  optional?: false;
  typeAnnotation?: null;
  parent?: Node;
}

export interface ThisExpression extends Span {
  comments?: NodeComments | null;
  type: "ThisExpression";
  parent?: Node;
}

export interface ArrayExpression extends Span {
  comments?: NodeComments | null;
  type: "ArrayExpression";
  elements: Array<ArrayExpressionElement>;
  parent?: Node;
}

export type ArrayExpressionElement = SpreadElement | null | Expression;

export interface ObjectExpression extends Span {
  comments?: NodeComments | null;
  type: "ObjectExpression";
  properties: Array<ObjectPropertyKind>;
  parent?: Node;
}

export type ObjectPropertyKind = ObjectProperty | SpreadElement;

export interface ObjectProperty extends Span {
  comments?: NodeComments | null;
  type: "Property";
  kind: PropertyKind;
  key: PropertyKey;
  value: Expression;
  method: boolean;
  shorthand: boolean;
  computed: boolean;
  optional?: false;
  parent?: Node;
}

export type PropertyKey = IdentifierName | PrivateIdentifier | Expression;

export type PropertyKind = "init" | "get" | "set";

export interface TemplateLiteral extends Span {
  comments?: NodeComments | null;
  type: "TemplateLiteral";
  quasis: Array<TemplateElement>;
  expressions: Array<Expression>;
  parent?: Node;
}

export interface TaggedTemplateExpression extends Span {
  comments?: NodeComments | null;
  type: "TaggedTemplateExpression";
  tag: Expression;
  typeArguments?: TSTypeParameterInstantiation | null;
  quasi: TemplateLiteral;
  parent?: Node;
}

export interface TemplateElement extends Span {
  comments?: NodeComments | null;
  type: "TemplateElement";
  value: TemplateElementValue;
  tail: boolean;
  parent?: Node;
}

export interface TemplateElementValue {
  raw: string;
  cooked: string | null;
}

export type MemberExpression =
  | ComputedMemberExpression
  | StaticMemberExpression
  | PrivateFieldExpression;

export interface ComputedMemberExpression extends Span {
  comments?: NodeComments | null;
  type: "MemberExpression";
  object: Expression;
  property: Expression;
  optional: boolean;
  computed: true;
  parent?: Node;
}

export interface StaticMemberExpression extends Span {
  comments?: NodeComments | null;
  type: "MemberExpression";
  object: Expression;
  property: IdentifierName;
  optional: boolean;
  computed: false;
  parent?: Node;
}

export interface PrivateFieldExpression extends Span {
  comments?: NodeComments | null;
  type: "MemberExpression";
  object: Expression;
  property: PrivateIdentifier;
  optional: boolean;
  computed: false;
  parent?: Node;
}

export interface CallExpression extends Span {
  comments?: NodeComments | null;
  type: "CallExpression";
  callee: Expression;
  typeArguments?: TSTypeParameterInstantiation | null;
  arguments: Array<Argument>;
  optional: boolean;
  parent?: Node;
}

export interface NewExpression extends Span {
  comments?: NodeComments | null;
  type: "NewExpression";
  callee: Expression;
  typeArguments?: TSTypeParameterInstantiation | null;
  arguments: Array<Argument>;
  parent?: Node;
}

export interface MetaProperty extends Span {
  comments?: NodeComments | null;
  type: "MetaProperty";
  meta: IdentifierName;
  property: IdentifierName;
  parent?: Node;
}

export interface SpreadElement extends Span {
  comments?: NodeComments | null;
  type: "SpreadElement";
  argument: Expression;
  parent?: Node;
}

export type Argument = SpreadElement | Expression;

export interface UpdateExpression extends Span {
  comments?: NodeComments | null;
  type: "UpdateExpression";
  operator: UpdateOperator;
  prefix: boolean;
  argument: SimpleAssignmentTarget;
  parent?: Node;
}

export interface UnaryExpression extends Span {
  comments?: NodeComments | null;
  type: "UnaryExpression";
  operator: UnaryOperator;
  argument: Expression;
  prefix: true;
  parent?: Node;
}

export interface BinaryExpression extends Span {
  comments?: NodeComments | null;
  type: "BinaryExpression";
  left: Expression;
  operator: BinaryOperator;
  right: Expression;
  parent?: Node;
}

export interface PrivateInExpression extends Span {
  comments?: NodeComments | null;
  type: "BinaryExpression";
  left: PrivateIdentifier;
  operator: "in";
  right: Expression;
  parent?: Node;
}

export interface LogicalExpression extends Span {
  comments?: NodeComments | null;
  type: "LogicalExpression";
  left: Expression;
  operator: LogicalOperator;
  right: Expression;
  parent?: Node;
}

export interface ConditionalExpression extends Span {
  comments?: NodeComments | null;
  type: "ConditionalExpression";
  test: Expression;
  consequent: Expression;
  alternate: Expression;
  parent?: Node;
}

export interface AssignmentExpression extends Span {
  comments?: NodeComments | null;
  type: "AssignmentExpression";
  operator: AssignmentOperator;
  left: AssignmentTarget;
  right: Expression;
  parent?: Node;
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

export interface ArrayAssignmentTarget extends Span {
  comments?: NodeComments | null;
  type: "ArrayPattern";
  decorators?: [];
  elements: Array<AssignmentTargetMaybeDefault | AssignmentTargetRest | null>;
  optional?: false;
  typeAnnotation?: null;
  parent?: Node;
}

export interface ObjectAssignmentTarget extends Span {
  comments?: NodeComments | null;
  type: "ObjectPattern";
  decorators?: [];
  properties: Array<AssignmentTargetProperty | AssignmentTargetRest>;
  optional?: false;
  typeAnnotation?: null;
  parent?: Node;
}

export interface AssignmentTargetRest extends Span {
  comments?: NodeComments | null;
  type: "RestElement";
  decorators?: [];
  argument: AssignmentTarget;
  optional?: false;
  typeAnnotation?: null;
  value?: null;
  parent?: Node;
}

export type AssignmentTargetMaybeDefault = AssignmentTargetWithDefault | AssignmentTarget;

export interface AssignmentTargetWithDefault extends Span {
  comments?: NodeComments | null;
  type: "AssignmentPattern";
  decorators?: [];
  left: AssignmentTarget;
  right: Expression;
  optional?: false;
  typeAnnotation?: null;
  parent?: Node;
}

export type AssignmentTargetProperty =
  | AssignmentTargetPropertyIdentifier
  | AssignmentTargetPropertyProperty;

export interface AssignmentTargetPropertyIdentifier extends Span {
  comments?: NodeComments | null;
  type: "Property";
  kind: "init";
  key: IdentifierReference;
  value: IdentifierReference | AssignmentTargetWithDefault;
  method: false;
  shorthand: true;
  computed: false;
  optional?: false;
  parent?: Node;
}

export interface AssignmentTargetPropertyProperty extends Span {
  comments?: NodeComments | null;
  type: "Property";
  kind: "init";
  key: PropertyKey;
  value: AssignmentTargetMaybeDefault;
  method: false;
  shorthand: false;
  computed: boolean;
  optional?: false;
  parent?: Node;
}

export interface SequenceExpression extends Span {
  comments?: NodeComments | null;
  type: "SequenceExpression";
  expressions: Array<Expression>;
  parent?: Node;
}

export interface Super extends Span {
  comments?: NodeComments | null;
  type: "Super";
  parent?: Node;
}

export interface AwaitExpression extends Span {
  comments?: NodeComments | null;
  type: "AwaitExpression";
  argument: Expression;
  parent?: Node;
}

export interface ChainExpression extends Span {
  comments?: NodeComments | null;
  type: "ChainExpression";
  expression: ChainElement;
  parent?: Node;
}

export type ChainElement = CallExpression | TSNonNullExpression | MemberExpression;

export interface ParenthesizedExpression extends Span {
  comments?: NodeComments | null;
  type: "ParenthesizedExpression";
  expression: Expression;
  parent?: Node;
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

export interface Directive extends Span {
  comments?: NodeComments | null;
  type: "ExpressionStatement";
  expression: StringLiteral;
  directive: string;
  parent?: Node;
}

export interface Hashbang extends Span {
  comments?: NodeComments | null;
  type: "Hashbang";
  value: string;
  parent?: Node;
}

export interface BlockStatement extends Span {
  comments?: NodeComments | null;
  type: "BlockStatement";
  body: Array<Statement>;
  parent?: Node;
}

export type Declaration =
  | VariableDeclaration
  | Function
  | Class
  | TSTypeAliasDeclaration
  | TSInterfaceDeclaration
  | TSEnumDeclaration
  | TSModuleDeclaration
  | TSGlobalDeclaration
  | TSImportEqualsDeclaration;

export interface VariableDeclaration extends Span {
  comments?: NodeComments | null;
  type: "VariableDeclaration";
  kind: VariableDeclarationKind;
  declarations: Array<VariableDeclarator>;
  declare?: boolean;
  parent?: Node;
}

export type VariableDeclarationKind = "var" | "let" | "const" | "using" | "await using";

export interface VariableDeclarator extends Span {
  comments?: NodeComments | null;
  type: "VariableDeclarator";
  id: BindingPattern;
  init: Expression | null;
  definite?: boolean;
  parent?: Node;
}

export interface EmptyStatement extends Span {
  comments?: NodeComments | null;
  type: "EmptyStatement";
  parent?: Node;
}

export interface ExpressionStatement extends Span {
  comments?: NodeComments | null;
  type: "ExpressionStatement";
  expression: Expression;
  directive?: string | null;
  parent?: Node;
}

export interface IfStatement extends Span {
  comments?: NodeComments | null;
  type: "IfStatement";
  test: Expression;
  consequent: Statement;
  alternate: Statement | null;
  parent?: Node;
}

export interface DoWhileStatement extends Span {
  comments?: NodeComments | null;
  type: "DoWhileStatement";
  body: Statement;
  test: Expression;
  parent?: Node;
}

export interface WhileStatement extends Span {
  comments?: NodeComments | null;
  type: "WhileStatement";
  test: Expression;
  body: Statement;
  parent?: Node;
}

export interface ForStatement extends Span {
  comments?: NodeComments | null;
  type: "ForStatement";
  init: ForStatementInit | null;
  test: Expression | null;
  update: Expression | null;
  body: Statement;
  parent?: Node;
}

export type ForStatementInit = VariableDeclaration | Expression;

export interface ForInStatement extends Span {
  comments?: NodeComments | null;
  type: "ForInStatement";
  left: ForStatementLeft;
  right: Expression;
  body: Statement;
  parent?: Node;
}

export type ForStatementLeft = VariableDeclaration | AssignmentTarget;

export interface ForOfStatement extends Span {
  comments?: NodeComments | null;
  type: "ForOfStatement";
  await: boolean;
  left: ForStatementLeft;
  right: Expression;
  body: Statement;
  parent?: Node;
}

export interface ContinueStatement extends Span {
  comments?: NodeComments | null;
  type: "ContinueStatement";
  label: LabelIdentifier | null;
  parent?: Node;
}

export interface BreakStatement extends Span {
  comments?: NodeComments | null;
  type: "BreakStatement";
  label: LabelIdentifier | null;
  parent?: Node;
}

export interface ReturnStatement extends Span {
  comments?: NodeComments | null;
  type: "ReturnStatement";
  argument: Expression | null;
  parent?: Node;
}

export interface WithStatement extends Span {
  comments?: NodeComments | null;
  type: "WithStatement";
  object: Expression;
  body: Statement;
  parent?: Node;
}

export interface SwitchStatement extends Span {
  comments?: NodeComments | null;
  type: "SwitchStatement";
  discriminant: Expression;
  cases: Array<SwitchCase>;
  parent?: Node;
}

export interface SwitchCase extends Span {
  comments?: NodeComments | null;
  type: "SwitchCase";
  test: Expression | null;
  consequent: Array<Statement>;
  parent?: Node;
}

export interface LabeledStatement extends Span {
  comments?: NodeComments | null;
  type: "LabeledStatement";
  label: LabelIdentifier;
  body: Statement;
  parent?: Node;
}

export interface ThrowStatement extends Span {
  comments?: NodeComments | null;
  type: "ThrowStatement";
  argument: Expression;
  parent?: Node;
}

export interface TryStatement extends Span {
  comments?: NodeComments | null;
  type: "TryStatement";
  block: BlockStatement;
  handler: CatchClause | null;
  finalizer: BlockStatement | null;
  parent?: Node;
}

export interface CatchClause extends Span {
  comments?: NodeComments | null;
  type: "CatchClause";
  param: BindingPattern | null;
  body: BlockStatement;
  parent?: Node;
}

export interface DebuggerStatement extends Span {
  comments?: NodeComments | null;
  type: "DebuggerStatement";
  parent?: Node;
}

export type BindingPattern = BindingIdentifier | ObjectPattern | ArrayPattern | AssignmentPattern;

export interface AssignmentPattern extends Span {
  comments?: NodeComments | null;
  type: "AssignmentPattern";
  decorators?: [];
  left: BindingPattern;
  right: Expression;
  optional?: false;
  typeAnnotation?: null;
  parent?: Node;
}

export interface ObjectPattern extends Span {
  comments?: NodeComments | null;
  type: "ObjectPattern";
  decorators?: [];
  properties: Array<BindingProperty | BindingRestElement>;
  optional?: false;
  typeAnnotation?: TSTypeAnnotation | null;
  parent?: Node;
}

export interface BindingProperty extends Span {
  comments?: NodeComments | null;
  type: "Property";
  kind: "init";
  key: PropertyKey;
  value: BindingPattern;
  method: false;
  shorthand: boolean;
  computed: boolean;
  optional?: false;
  parent?: Node;
}

export interface ArrayPattern extends Span {
  comments?: NodeComments | null;
  type: "ArrayPattern";
  decorators?: [];
  elements: Array<BindingPattern | BindingRestElement | null>;
  optional?: false;
  typeAnnotation?: TSTypeAnnotation | null;
  parent?: Node;
}

export interface BindingRestElement extends Span {
  comments?: NodeComments | null;
  type: "RestElement";
  decorators?: [];
  argument: BindingPattern;
  optional?: false;
  typeAnnotation?: TSTypeAnnotation | null;
  value?: null;
  parent?: Node;
}

export interface Function extends Span {
  comments?: NodeComments | null;
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
  parent?: Node;
}

export type ParamPattern = FormalParameter | TSParameterProperty | FormalParameterRest;

export type FunctionType =
  | "FunctionDeclaration"
  | "FunctionExpression"
  | "TSDeclareFunction"
  | "TSEmptyBodyFunctionExpression";

export interface FormalParameterRest extends Span {
  comments?: NodeComments | null;
  type: "RestElement";
  argument: BindingPattern;
  decorators?: Array<Decorator>;
  optional?: boolean;
  typeAnnotation?: TSTypeAnnotation | null;
  value?: null;
  parent?: Node;
}

export type FormalParameter = {
  decorators?: Array<Decorator>;
} & BindingPattern;

export interface TSParameterProperty extends Span {
  comments?: NodeComments | null;
  type: "TSParameterProperty";
  accessibility: TSAccessibility | null;
  decorators: Array<Decorator>;
  override: boolean;
  parameter: FormalParameter;
  readonly: boolean;
  static: boolean;
  parent?: Node;
}

export interface FunctionBody extends Span {
  comments?: NodeComments | null;
  type: "BlockStatement";
  body: Array<Directive | Statement>;
  parent?: Node;
}

export interface ArrowFunctionExpression extends Span {
  comments?: NodeComments | null;
  type: "ArrowFunctionExpression";
  expression: boolean;
  async: boolean;
  typeParameters?: TSTypeParameterDeclaration | null;
  params: ParamPattern[];
  returnType?: TSTypeAnnotation | null;
  body: FunctionBody | Expression;
  id: null;
  generator: false;
  parent?: Node;
}

export interface YieldExpression extends Span {
  comments?: NodeComments | null;
  type: "YieldExpression";
  delegate: boolean;
  argument: Expression | null;
  parent?: Node;
}

export interface Class extends Span {
  comments?: NodeComments | null;
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
  parent?: Node;
}

export type ClassType = "ClassDeclaration" | "ClassExpression";

export interface ClassBody extends Span {
  comments?: NodeComments | null;
  type: "ClassBody";
  body: Array<ClassElement>;
  parent?: Node;
}

export type ClassElement =
  | StaticBlock
  | MethodDefinition
  | PropertyDefinition
  | AccessorProperty
  | TSIndexSignature;

export interface MethodDefinition extends Span {
  comments?: NodeComments | null;
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
  parent?: Node;
}

export type MethodDefinitionType = "MethodDefinition" | "TSAbstractMethodDefinition";

export interface PropertyDefinition extends Span {
  comments?: NodeComments | null;
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
  parent?: Node;
}

export type PropertyDefinitionType = "PropertyDefinition" | "TSAbstractPropertyDefinition";

export type MethodDefinitionKind = "constructor" | "method" | "get" | "set";

export interface PrivateIdentifier extends Span {
  comments?: NodeComments | null;
  type: "PrivateIdentifier";
  name: string;
  parent?: Node;
}

export interface StaticBlock extends Span {
  comments?: NodeComments | null;
  type: "StaticBlock";
  body: Array<Statement>;
  parent?: Node;
}

export type ModuleDeclaration =
  | ImportDeclaration
  | ExportAllDeclaration
  | ExportDefaultDeclaration
  | ExportNamedDeclaration
  | TSExportAssignment
  | TSNamespaceExportDeclaration;

export type AccessorPropertyType = "AccessorProperty" | "TSAbstractAccessorProperty";

export interface AccessorProperty extends Span {
  comments?: NodeComments | null;
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
  parent?: Node;
}

export interface ImportExpression extends Span {
  comments?: NodeComments | null;
  type: "ImportExpression";
  source: Expression;
  options: Expression | null;
  phase: ImportPhase | null;
  parent?: Node;
}

export interface ImportDeclaration extends Span {
  comments?: NodeComments | null;
  type: "ImportDeclaration";
  specifiers: Array<ImportDeclarationSpecifier>;
  source: StringLiteral;
  phase: ImportPhase | null;
  attributes: Array<ImportAttribute>;
  importKind?: ImportOrExportKind;
  parent?: Node;
}

export type ImportPhase = "source" | "defer";

export type ImportDeclarationSpecifier =
  | ImportSpecifier
  | ImportDefaultSpecifier
  | ImportNamespaceSpecifier;

export interface ImportSpecifier extends Span {
  comments?: NodeComments | null;
  type: "ImportSpecifier";
  imported: ModuleExportName;
  local: BindingIdentifier;
  importKind?: ImportOrExportKind;
  parent?: Node;
}

export interface ImportDefaultSpecifier extends Span {
  comments?: NodeComments | null;
  type: "ImportDefaultSpecifier";
  local: BindingIdentifier;
  parent?: Node;
}

export interface ImportNamespaceSpecifier extends Span {
  comments?: NodeComments | null;
  type: "ImportNamespaceSpecifier";
  local: BindingIdentifier;
  parent?: Node;
}

export interface ImportAttribute extends Span {
  comments?: NodeComments | null;
  type: "ImportAttribute";
  key: ImportAttributeKey;
  value: StringLiteral;
  parent?: Node;
}

export type ImportAttributeKey = IdentifierName | StringLiteral;

export interface ExportNamedDeclaration extends Span {
  comments?: NodeComments | null;
  type: "ExportNamedDeclaration";
  declaration: Declaration | null;
  specifiers: Array<ExportSpecifier>;
  source: StringLiteral | null;
  exportKind?: ImportOrExportKind;
  attributes: Array<ImportAttribute>;
  parent?: Node;
}

export interface ExportDefaultDeclaration extends Span {
  comments?: NodeComments | null;
  type: "ExportDefaultDeclaration";
  declaration: ExportDefaultDeclarationKind;
  exportKind?: "value";
  parent?: Node;
}

export interface ExportAllDeclaration extends Span {
  comments?: NodeComments | null;
  type: "ExportAllDeclaration";
  exported: ModuleExportName | null;
  source: StringLiteral;
  attributes: Array<ImportAttribute>;
  exportKind?: ImportOrExportKind;
  parent?: Node;
}

export interface ExportSpecifier extends Span {
  comments?: NodeComments | null;
  type: "ExportSpecifier";
  local: ModuleExportName;
  exported: ModuleExportName;
  exportKind?: ImportOrExportKind;
  parent?: Node;
}

export type ExportDefaultDeclarationKind = Function | Class | TSInterfaceDeclaration | Expression;

export type ModuleExportName = IdentifierName | IdentifierReference | StringLiteral;

export interface V8IntrinsicExpression extends Span {
  comments?: NodeComments | null;
  type: "V8IntrinsicExpression";
  name: IdentifierName;
  arguments: Array<Argument>;
  parent?: Node;
}

export interface BooleanLiteral extends Span {
  comments?: NodeComments | null;
  type: "Literal";
  value: boolean;
  raw: string | null;
  parent?: Node;
}

export interface NullLiteral extends Span {
  comments?: NodeComments | null;
  type: "Literal";
  value: null;
  raw: "null" | null;
  parent?: Node;
}

export interface NumericLiteral extends Span {
  comments?: NodeComments | null;
  type: "Literal";
  value: number;
  raw: string | null;
  parent?: Node;
}

export interface StringLiteral extends Span {
  comments?: NodeComments | null;
  type: "Literal";
  value: string;
  raw: string | null;
  parent?: Node;
}

export interface BigIntLiteral extends Span {
  comments?: NodeComments | null;
  type: "Literal";
  value: bigint;
  raw: string | null;
  bigint: string;
  parent?: Node;
}

export interface RegExpLiteral extends Span {
  comments?: NodeComments | null;
  type: "Literal";
  value: RegExp | null;
  raw: string | null;
  regex: { pattern: string; flags: string };
  parent?: Node;
}

export interface JSXElement extends Span {
  comments?: NodeComments | null;
  type: "JSXElement";
  openingElement: JSXOpeningElement;
  children: Array<JSXChild>;
  closingElement: JSXClosingElement | null;
  parent?: Node;
}

export interface JSXOpeningElement extends Span {
  comments?: NodeComments | null;
  type: "JSXOpeningElement";
  name: JSXElementName;
  typeArguments?: TSTypeParameterInstantiation | null;
  attributes: Array<JSXAttributeItem>;
  selfClosing: boolean;
  parent?: Node;
}

export interface JSXClosingElement extends Span {
  comments?: NodeComments | null;
  type: "JSXClosingElement";
  name: JSXElementName;
  parent?: Node;
}

export interface JSXFragment extends Span {
  comments?: NodeComments | null;
  type: "JSXFragment";
  openingFragment: JSXOpeningFragment;
  children: Array<JSXChild>;
  closingFragment: JSXClosingFragment;
  parent?: Node;
}

export interface JSXOpeningFragment extends Span {
  comments?: NodeComments | null;
  type: "JSXOpeningFragment";
  attributes?: [];
  selfClosing?: false;
  parent?: Node;
}

export interface JSXClosingFragment extends Span {
  comments?: NodeComments | null;
  type: "JSXClosingFragment";
  parent?: Node;
}

export type JSXElementName = JSXIdentifier | JSXNamespacedName | JSXMemberExpression;

export interface JSXNamespacedName extends Span {
  comments?: NodeComments | null;
  type: "JSXNamespacedName";
  namespace: JSXIdentifier;
  name: JSXIdentifier;
  parent?: Node;
}

export interface JSXMemberExpression extends Span {
  comments?: NodeComments | null;
  type: "JSXMemberExpression";
  object: JSXMemberExpressionObject;
  property: JSXIdentifier;
  parent?: Node;
}

export type JSXMemberExpressionObject = JSXIdentifier | JSXMemberExpression;

export interface JSXExpressionContainer extends Span {
  comments?: NodeComments | null;
  type: "JSXExpressionContainer";
  expression: JSXExpression;
  parent?: Node;
}

export type JSXExpression = JSXEmptyExpression | Expression;

export interface JSXEmptyExpression extends Span {
  comments?: NodeComments | null;
  type: "JSXEmptyExpression";
  parent?: Node;
}

export type JSXAttributeItem = JSXAttribute | JSXSpreadAttribute;

export interface JSXAttribute extends Span {
  comments?: NodeComments | null;
  type: "JSXAttribute";
  name: JSXAttributeName;
  value: JSXAttributeValue | null;
  parent?: Node;
}

export interface JSXSpreadAttribute extends Span {
  comments?: NodeComments | null;
  type: "JSXSpreadAttribute";
  argument: Expression;
  parent?: Node;
}

export type JSXAttributeName = JSXIdentifier | JSXNamespacedName;

export type JSXAttributeValue = StringLiteral | JSXExpressionContainer | JSXElement | JSXFragment;

export interface JSXIdentifier extends Span {
  comments?: NodeComments | null;
  type: "JSXIdentifier";
  name: string;
  parent?: Node;
}

export type JSXChild = JSXText | JSXElement | JSXFragment | JSXExpressionContainer | JSXSpreadChild;

export interface JSXSpreadChild extends Span {
  comments?: NodeComments | null;
  type: "JSXSpreadChild";
  expression: Expression;
  parent?: Node;
}

export interface JSXText extends Span {
  comments?: NodeComments | null;
  type: "JSXText";
  value: string;
  raw: string | null;
  parent?: Node;
}

export interface TSThisParameter extends Span {
  comments?: NodeComments | null;
  type: "Identifier";
  decorators: [];
  name: "this";
  optional: false;
  typeAnnotation: TSTypeAnnotation | null;
  parent?: Node;
}

export interface TSEnumDeclaration extends Span {
  comments?: NodeComments | null;
  type: "TSEnumDeclaration";
  id: BindingIdentifier;
  body: TSEnumBody;
  const: boolean;
  declare: boolean;
  parent?: Node;
}

export interface TSEnumBody extends Span {
  comments?: NodeComments | null;
  type: "TSEnumBody";
  members: Array<TSEnumMember>;
  parent?: Node;
}

export interface TSEnumMember extends Span {
  comments?: NodeComments | null;
  type: "TSEnumMember";
  id: TSEnumMemberName;
  initializer: Expression | null;
  computed: boolean;
  parent?: Node;
}

export type TSEnumMemberName = IdentifierName | StringLiteral | TemplateLiteral;

export interface TSTypeAnnotation extends Span {
  comments?: NodeComments | null;
  type: "TSTypeAnnotation";
  typeAnnotation: TSType;
  parent?: Node;
}

export interface TSLiteralType extends Span {
  comments?: NodeComments | null;
  type: "TSLiteralType";
  literal: TSLiteral;
  parent?: Node;
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

export interface TSConditionalType extends Span {
  comments?: NodeComments | null;
  type: "TSConditionalType";
  checkType: TSType;
  extendsType: TSType;
  trueType: TSType;
  falseType: TSType;
  parent?: Node;
}

export interface TSUnionType extends Span {
  comments?: NodeComments | null;
  type: "TSUnionType";
  types: Array<TSType>;
  parent?: Node;
}

export interface TSIntersectionType extends Span {
  comments?: NodeComments | null;
  type: "TSIntersectionType";
  types: Array<TSType>;
  parent?: Node;
}

export interface TSParenthesizedType extends Span {
  comments?: NodeComments | null;
  type: "TSParenthesizedType";
  typeAnnotation: TSType;
  parent?: Node;
}

export interface TSTypeOperator extends Span {
  comments?: NodeComments | null;
  type: "TSTypeOperator";
  operator: TSTypeOperatorOperator;
  typeAnnotation: TSType;
  parent?: Node;
}

export type TSTypeOperatorOperator = "keyof" | "unique" | "readonly";

export interface TSArrayType extends Span {
  comments?: NodeComments | null;
  type: "TSArrayType";
  elementType: TSType;
  parent?: Node;
}

export interface TSIndexedAccessType extends Span {
  comments?: NodeComments | null;
  type: "TSIndexedAccessType";
  objectType: TSType;
  indexType: TSType;
  parent?: Node;
}

export interface TSTupleType extends Span {
  comments?: NodeComments | null;
  type: "TSTupleType";
  elementTypes: Array<TSTupleElement>;
  parent?: Node;
}

export interface TSNamedTupleMember extends Span {
  comments?: NodeComments | null;
  type: "TSNamedTupleMember";
  label: IdentifierName;
  elementType: TSTupleElement;
  optional: boolean;
  parent?: Node;
}

export interface TSOptionalType extends Span {
  comments?: NodeComments | null;
  type: "TSOptionalType";
  typeAnnotation: TSType;
  parent?: Node;
}

export interface TSRestType extends Span {
  comments?: NodeComments | null;
  type: "TSRestType";
  typeAnnotation: TSType;
  parent?: Node;
}

export type TSTupleElement = TSOptionalType | TSRestType | TSType;

export interface TSAnyKeyword extends Span {
  comments?: NodeComments | null;
  type: "TSAnyKeyword";
  parent?: Node;
}

export interface TSStringKeyword extends Span {
  comments?: NodeComments | null;
  type: "TSStringKeyword";
  parent?: Node;
}

export interface TSBooleanKeyword extends Span {
  comments?: NodeComments | null;
  type: "TSBooleanKeyword";
  parent?: Node;
}

export interface TSNumberKeyword extends Span {
  comments?: NodeComments | null;
  type: "TSNumberKeyword";
  parent?: Node;
}

export interface TSNeverKeyword extends Span {
  comments?: NodeComments | null;
  type: "TSNeverKeyword";
  parent?: Node;
}

export interface TSIntrinsicKeyword extends Span {
  comments?: NodeComments | null;
  type: "TSIntrinsicKeyword";
  parent?: Node;
}

export interface TSUnknownKeyword extends Span {
  comments?: NodeComments | null;
  type: "TSUnknownKeyword";
  parent?: Node;
}

export interface TSNullKeyword extends Span {
  comments?: NodeComments | null;
  type: "TSNullKeyword";
  parent?: Node;
}

export interface TSUndefinedKeyword extends Span {
  comments?: NodeComments | null;
  type: "TSUndefinedKeyword";
  parent?: Node;
}

export interface TSVoidKeyword extends Span {
  comments?: NodeComments | null;
  type: "TSVoidKeyword";
  parent?: Node;
}

export interface TSSymbolKeyword extends Span {
  comments?: NodeComments | null;
  type: "TSSymbolKeyword";
  parent?: Node;
}

export interface TSThisType extends Span {
  comments?: NodeComments | null;
  type: "TSThisType";
  parent?: Node;
}

export interface TSObjectKeyword extends Span {
  comments?: NodeComments | null;
  type: "TSObjectKeyword";
  parent?: Node;
}

export interface TSBigIntKeyword extends Span {
  comments?: NodeComments | null;
  type: "TSBigIntKeyword";
  parent?: Node;
}

export interface TSTypeReference extends Span {
  comments?: NodeComments | null;
  type: "TSTypeReference";
  typeName: TSTypeName;
  typeArguments: TSTypeParameterInstantiation | null;
  parent?: Node;
}

export type TSTypeName = IdentifierReference | TSQualifiedName | ThisExpression;

export interface TSQualifiedName extends Span {
  comments?: NodeComments | null;
  type: "TSQualifiedName";
  left: TSTypeName;
  right: IdentifierName;
  parent?: Node;
}

export interface TSTypeParameterInstantiation extends Span {
  comments?: NodeComments | null;
  type: "TSTypeParameterInstantiation";
  params: Array<TSType>;
  parent?: Node;
}

export interface TSTypeParameter extends Span {
  comments?: NodeComments | null;
  type: "TSTypeParameter";
  name: BindingIdentifier;
  constraint: TSType | null;
  default: TSType | null;
  in: boolean;
  out: boolean;
  const: boolean;
  parent?: Node;
}

export interface TSTypeParameterDeclaration extends Span {
  comments?: NodeComments | null;
  type: "TSTypeParameterDeclaration";
  params: Array<TSTypeParameter>;
  parent?: Node;
}

export interface TSTypeAliasDeclaration extends Span {
  comments?: NodeComments | null;
  type: "TSTypeAliasDeclaration";
  id: BindingIdentifier;
  typeParameters: TSTypeParameterDeclaration | null;
  typeAnnotation: TSType;
  declare: boolean;
  parent?: Node;
}

export type TSAccessibility = "private" | "protected" | "public";

export interface TSClassImplements extends Span {
  comments?: NodeComments | null;
  type: "TSClassImplements";
  expression: IdentifierReference | ThisExpression | MemberExpression;
  typeArguments: TSTypeParameterInstantiation | null;
  parent?: Node;
}

export interface TSInterfaceDeclaration extends Span {
  comments?: NodeComments | null;
  type: "TSInterfaceDeclaration";
  id: BindingIdentifier;
  typeParameters: TSTypeParameterDeclaration | null;
  extends: Array<TSInterfaceHeritage>;
  body: TSInterfaceBody;
  declare: boolean;
  parent?: Node;
}

export interface TSInterfaceBody extends Span {
  comments?: NodeComments | null;
  type: "TSInterfaceBody";
  body: Array<TSSignature>;
  parent?: Node;
}

export interface TSPropertySignature extends Span {
  comments?: NodeComments | null;
  type: "TSPropertySignature";
  computed: boolean;
  optional: boolean;
  readonly: boolean;
  key: PropertyKey;
  typeAnnotation: TSTypeAnnotation | null;
  accessibility: null;
  static: false;
  parent?: Node;
}

export type TSSignature =
  | TSIndexSignature
  | TSPropertySignature
  | TSCallSignatureDeclaration
  | TSConstructSignatureDeclaration
  | TSMethodSignature;

export interface TSIndexSignature extends Span {
  comments?: NodeComments | null;
  type: "TSIndexSignature";
  parameters: Array<TSIndexSignatureName>;
  typeAnnotation: TSTypeAnnotation;
  readonly: boolean;
  static: boolean;
  accessibility: null;
  parent?: Node;
}

export interface TSCallSignatureDeclaration extends Span {
  comments?: NodeComments | null;
  type: "TSCallSignatureDeclaration";
  typeParameters: TSTypeParameterDeclaration | null;
  params: ParamPattern[];
  returnType: TSTypeAnnotation | null;
  parent?: Node;
}

export type TSMethodSignatureKind = "method" | "get" | "set";

export interface TSMethodSignature extends Span {
  comments?: NodeComments | null;
  type: "TSMethodSignature";
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
  parent?: Node;
}

export interface TSConstructSignatureDeclaration extends Span {
  comments?: NodeComments | null;
  type: "TSConstructSignatureDeclaration";
  typeParameters: TSTypeParameterDeclaration | null;
  params: ParamPattern[];
  returnType: TSTypeAnnotation | null;
  parent?: Node;
}

export interface TSIndexSignatureName extends Span {
  comments?: NodeComments | null;
  type: "Identifier";
  decorators: [];
  name: string;
  optional: false;
  typeAnnotation: TSTypeAnnotation;
  parent?: Node;
}

export interface TSInterfaceHeritage extends Span {
  comments?: NodeComments | null;
  type: "TSInterfaceHeritage";
  expression: Expression;
  typeArguments: TSTypeParameterInstantiation | null;
  parent?: Node;
}

export interface TSTypePredicate extends Span {
  comments?: NodeComments | null;
  type: "TSTypePredicate";
  parameterName: TSTypePredicateName;
  asserts: boolean;
  typeAnnotation: TSTypeAnnotation | null;
  parent?: Node;
}

export type TSTypePredicateName = IdentifierName | TSThisType;

export type TSModuleDeclarationKind = "module" | "namespace";

export interface TSModuleDeclaration extends Span {
  comments?: NodeComments | null;
  type: "TSModuleDeclaration";
  id: BindingIdentifier | StringLiteral | TSQualifiedName;
  body: TSModuleBlock | null;
  kind: TSModuleDeclarationKind;
  declare: boolean;
  global: false;
  parent?: Node;
}

export interface TSGlobalDeclaration extends Span {
  comments?: NodeComments | null;
  type: "TSModuleDeclaration";
  id: IdentifierName;
  body: TSModuleBlock;
  kind: "global";
  declare: boolean;
  global: true;
  parent?: Node;
}

export interface TSModuleBlock extends Span {
  comments?: NodeComments | null;
  type: "TSModuleBlock";
  body: Array<Directive | Statement>;
  parent?: Node;
}

export interface TSTypeLiteral extends Span {
  comments?: NodeComments | null;
  type: "TSTypeLiteral";
  members: Array<TSSignature>;
  parent?: Node;
}

export interface TSInferType extends Span {
  comments?: NodeComments | null;
  type: "TSInferType";
  typeParameter: TSTypeParameter;
  parent?: Node;
}

export interface TSTypeQuery extends Span {
  comments?: NodeComments | null;
  type: "TSTypeQuery";
  exprName: TSTypeQueryExprName;
  typeArguments: TSTypeParameterInstantiation | null;
  parent?: Node;
}

export type TSTypeQueryExprName = TSImportType | TSTypeName;

export interface TSImportType extends Span {
  comments?: NodeComments | null;
  type: "TSImportType";
  source: StringLiteral;
  options: ObjectExpression | null;
  qualifier: TSImportTypeQualifier | null;
  typeArguments: TSTypeParameterInstantiation | null;
  parent?: Node;
}

export type TSImportTypeQualifier = IdentifierName | TSImportTypeQualifiedName;

export interface TSImportTypeQualifiedName extends Span {
  comments?: NodeComments | null;
  type: "TSQualifiedName";
  left: TSImportTypeQualifier;
  right: IdentifierName;
  parent?: Node;
}

export interface TSFunctionType extends Span {
  comments?: NodeComments | null;
  type: "TSFunctionType";
  typeParameters: TSTypeParameterDeclaration | null;
  params: ParamPattern[];
  returnType: TSTypeAnnotation;
  parent?: Node;
}

export interface TSConstructorType extends Span {
  comments?: NodeComments | null;
  type: "TSConstructorType";
  abstract: boolean;
  typeParameters: TSTypeParameterDeclaration | null;
  params: ParamPattern[];
  returnType: TSTypeAnnotation;
  parent?: Node;
}

export interface TSMappedType extends Span {
  comments?: NodeComments | null;
  type: "TSMappedType";
  key: BindingIdentifier;
  constraint: TSType;
  nameType: TSType | null;
  typeAnnotation: TSType | null;
  optional: TSMappedTypeModifierOperator | false;
  readonly: TSMappedTypeModifierOperator | null;
  parent?: Node;
}

export type TSMappedTypeModifierOperator = true | "+" | "-";

export interface TSTemplateLiteralType extends Span {
  comments?: NodeComments | null;
  type: "TSTemplateLiteralType";
  quasis: Array<TemplateElement>;
  types: Array<TSType>;
  parent?: Node;
}

export interface TSAsExpression extends Span {
  comments?: NodeComments | null;
  type: "TSAsExpression";
  expression: Expression;
  typeAnnotation: TSType;
  parent?: Node;
}

export interface TSSatisfiesExpression extends Span {
  comments?: NodeComments | null;
  type: "TSSatisfiesExpression";
  expression: Expression;
  typeAnnotation: TSType;
  parent?: Node;
}

export interface TSTypeAssertion extends Span {
  comments?: NodeComments | null;
  type: "TSTypeAssertion";
  typeAnnotation: TSType;
  expression: Expression;
  parent?: Node;
}

export interface TSImportEqualsDeclaration extends Span {
  comments?: NodeComments | null;
  type: "TSImportEqualsDeclaration";
  id: BindingIdentifier;
  moduleReference: TSModuleReference;
  importKind: ImportOrExportKind;
  parent?: Node;
}

export type TSModuleReference = TSExternalModuleReference | IdentifierReference | TSQualifiedName;

export interface TSExternalModuleReference extends Span {
  comments?: NodeComments | null;
  type: "TSExternalModuleReference";
  expression: StringLiteral;
  parent?: Node;
}

export interface TSNonNullExpression extends Span {
  comments?: NodeComments | null;
  type: "TSNonNullExpression";
  expression: Expression;
  parent?: Node;
}

export interface Decorator extends Span {
  comments?: NodeComments | null;
  type: "Decorator";
  expression: Expression;
  parent?: Node;
}

export interface TSExportAssignment extends Span {
  comments?: NodeComments | null;
  type: "TSExportAssignment";
  expression: Expression;
  parent?: Node;
}

export interface TSNamespaceExportDeclaration extends Span {
  comments?: NodeComments | null;
  type: "TSNamespaceExportDeclaration";
  id: IdentifierName;
  parent?: Node;
}

export interface TSInstantiationExpression extends Span {
  comments?: NodeComments | null;
  type: "TSInstantiationExpression";
  expression: Expression;
  typeArguments: TSTypeParameterInstantiation;
  parent?: Node;
}

export type ImportOrExportKind = "value" | "type";

export interface JSDocNullableType extends Span {
  comments?: NodeComments | null;
  type: "TSJSDocNullableType";
  typeAnnotation: TSType;
  postfix: boolean;
  parent?: Node;
}

export interface JSDocNonNullableType extends Span {
  comments?: NodeComments | null;
  type: "TSJSDocNonNullableType";
  typeAnnotation: TSType;
  postfix: boolean;
  parent?: Node;
}

export interface JSDocUnknownType extends Span {
  comments?: NodeComments | null;
  type: "TSJSDocUnknownType";
  parent?: Node;
}

export type ModuleKind = "script" | "module" | "commonjs";

export interface Span {
  start: number;
  end: number;
  range?: [number, number];
}

export type AssignmentOperator =
  | "="
  | "+="
  | "-="
  | "*="
  | "/="
  | "%="
  | "**="
  | "<<="
  | ">>="
  | ">>>="
  | "|="
  | "^="
  | "&="
  | "||="
  | "&&="
  | "??=";

export type BinaryOperator =
  | "=="
  | "!="
  | "==="
  | "!=="
  | "<"
  | "<="
  | ">"
  | ">="
  | "+"
  | "-"
  | "*"
  | "/"
  | "%"
  | "**"
  | "<<"
  | ">>"
  | ">>>"
  | "|"
  | "^"
  | "&"
  | "in"
  | "instanceof";

export type LogicalOperator = "||" | "&&" | "??";

export type UnaryOperator = "+" | "-" | "!" | "~" | "typeof" | "void" | "delete";

export type UpdateOperator = "++" | "--";

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
  | TSGlobalDeclaration
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
