// To edit this generated file you have to edit `tasks/ast_tools/src/generators/typescript.rs`
// Auto-generated code, DO NOT EDIT DIRECTLY!

type bool = boolean;type f64 = number;type str = string;type Atom = string;type u32 = number;type u64 = number;

export type BooleanLiteral = {span: Span;value: bool;}

export type NullLiteral = {span: Span;}

export type NumericLiteral = {span: Span;value: f64;raw: str;base: NumberBase;}

export type BigIntLiteral = {span: Span;raw: Atom;base: BigintBase;}

export type RegExpLiteral = {span: Span;value: EmptyObject;regex: RegExp;}

export type RegExp = {pattern: RegExpPattern;flags: RegExpFlags;}

export type RegExpPattern = Raw | Invalid | Pattern

export type EmptyObject = {}

export type StringLiteral = {span: Span;value: Atom;}

export type Program = {span: Span;source_type: SourceType;hashbang: (Hashbang) | null;directives: Array<Directive>;body: Array<Statement>;scope_id: (ScopeId) | null;}

export type Expression = BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | Identifier | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | ClassExpression | ConditionalExpression | FunctionExpression | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression

export type IdentifierName = {span: Span;name: Atom;}

export type IdentifierReference = {span: Span;name: Atom;reference_id: (ReferenceId) | null;}

export type BindingIdentifier = {span: Span;name: Atom;symbol_id: (SymbolId) | null;}

export type LabelIdentifier = {span: Span;name: Atom;}

export type ThisExpression = {span: Span;}

export type ArrayExpression = {span: Span;elements: Array<ArrayExpressionElement>;trailing_comma: (Span) | null;}

export type ArrayExpressionElement = SpreadElement | Elision | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | Identifier | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | ClassExpression | ConditionalExpression | FunctionExpression | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression

export type Elision = {span: Span;}

export type ObjectExpression = {span: Span;properties: Array<ObjectPropertyKind>;trailing_comma: (Span) | null;}

export type ObjectPropertyKind = ObjectProperty | SpreadProperty

export type ObjectProperty = {span: Span;kind: PropertyKind;key: PropertyKey;value: Expression;init: (Expression) | null;method: bool;shorthand: bool;computed: bool;}

export type PropertyKey = StaticIdentifier | PrivateIdentifier | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | Identifier | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | ClassExpression | ConditionalExpression | FunctionExpression | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression

export type PropertyKind = 'Init' | 'Get' | 'Set'

export type TemplateLiteral = {span: Span;quasis: Array<TemplateElement>;expressions: Array<Expression>;}

export type TaggedTemplateExpression = {span: Span;tag: Expression;quasi: TemplateLiteral;type_parameters: (TSTypeParameterInstantiation) | null;}

export type TemplateElement = {span: Span;tail: bool;value: TemplateElementValue;}

export type TemplateElementValue = {raw: Atom;cooked: (Atom) | null;}

export type MemberExpression = ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression

export type ComputedMemberExpression = {span: Span;object: Expression;expression: Expression;optional: bool;}

export type StaticMemberExpression = {span: Span;object: Expression;property: IdentifierName;optional: bool;}

export type PrivateFieldExpression = {span: Span;object: Expression;field: PrivateIdentifier;optional: bool;}

export type CallExpression = {span: Span;callee: Expression;type_parameters: (TSTypeParameterInstantiation) | null;arguments: Array<Argument>;optional: bool;}

export type NewExpression = {span: Span;callee: Expression;arguments: Array<Argument>;type_parameters: (TSTypeParameterInstantiation) | null;}

export type MetaProperty = {span: Span;meta: IdentifierName;property: IdentifierName;}

export type SpreadElement = {span: Span;argument: Expression;}

export type Argument = SpreadElement | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | Identifier | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | ClassExpression | ConditionalExpression | FunctionExpression | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression

export type UpdateExpression = {span: Span;operator: UpdateOperator;prefix: bool;argument: SimpleAssignmentTarget;}

export type UnaryExpression = {span: Span;operator: UnaryOperator;argument: Expression;}

export type BinaryExpression = {span: Span;left: Expression;operator: BinaryOperator;right: Expression;}

export type PrivateInExpression = {span: Span;left: PrivateIdentifier;operator: BinaryOperator;right: Expression;}

export type LogicalExpression = {span: Span;left: Expression;operator: LogicalOperator;right: Expression;}

export type ConditionalExpression = {span: Span;test: Expression;consequent: Expression;alternate: Expression;}

export type AssignmentExpression = {span: Span;operator: AssignmentOperator;left: AssignmentTarget;right: Expression;}

export type AssignmentTarget = AssignmentTargetIdentifier | TSAsExpression | TSSatisfiesExpression | TSNonNullExpression | TSTypeAssertion | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression | ArrayAssignmentTarget | ObjectAssignmentTarget

export type SimpleAssignmentTarget = AssignmentTargetIdentifier | TSAsExpression | TSSatisfiesExpression | TSNonNullExpression | TSTypeAssertion | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression

export type AssignmentTargetPattern = ArrayAssignmentTarget | ObjectAssignmentTarget

export type ArrayAssignmentTarget = {span: Span;elements: Array<(AssignmentTargetMaybeDefault) | null>;rest: (AssignmentTargetRest) | null;trailing_comma: (Span) | null;}

export type ObjectAssignmentTarget = {span: Span;properties: Array<AssignmentTargetProperty>;rest: (AssignmentTargetRest) | null;}

export type AssignmentTargetRest = {span: Span;target: AssignmentTarget;}

export type AssignmentTargetMaybeDefault = AssignmentTargetWithDefault | AssignmentTargetIdentifier | TSAsExpression | TSSatisfiesExpression | TSNonNullExpression | TSTypeAssertion | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression | ArrayAssignmentTarget | ObjectAssignmentTarget

export type AssignmentTargetWithDefault = {span: Span;binding: AssignmentTarget;init: Expression;}

export type AssignmentTargetProperty = AssignmentTargetPropertyIdentifier | AssignmentTargetPropertyProperty

export type AssignmentTargetPropertyIdentifier = {span: Span;binding: IdentifierReference;init: (Expression) | null;}

export type AssignmentTargetPropertyProperty = {span: Span;name: PropertyKey;binding: AssignmentTargetMaybeDefault;}

export type SequenceExpression = {span: Span;expressions: Array<Expression>;}

export type Super = {span: Span;}

export type AwaitExpression = {span: Span;argument: Expression;}

export type ChainExpression = {span: Span;expression: ChainElement;}

export type ChainElement = CallExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression

export type ParenthesizedExpression = {span: Span;expression: Expression;}

export type Statement = BlockStatement | BreakStatement | ContinueStatement | DebuggerStatement | DoWhileStatement | EmptyStatement | ExpressionStatement | ForInStatement | ForOfStatement | ForStatement | IfStatement | LabeledStatement | ReturnStatement | SwitchStatement | ThrowStatement | TryStatement | WhileStatement | WithStatement | VariableDeclaration | FunctionDeclaration | ClassDeclaration | TSTypeAliasDeclaration | TSInterfaceDeclaration | TSEnumDeclaration | TSModuleDeclaration | TSImportEqualsDeclaration | ImportDeclaration | ExportAllDeclaration | ExportDefaultDeclaration | ExportNamedDeclaration | TSExportAssignment | TSNamespaceExportDeclaration

export type Directive = {span: Span;expression: StringLiteral;directive: Atom;}

export type Hashbang = {span: Span;value: Atom;}

export type BlockStatement = {span: Span;body: Array<Statement>;scope_id: (ScopeId) | null;}

export type Declaration = VariableDeclaration | FunctionDeclaration | ClassDeclaration | TSTypeAliasDeclaration | TSInterfaceDeclaration | TSEnumDeclaration | TSModuleDeclaration | TSImportEqualsDeclaration

export type VariableDeclaration = {span: Span;kind: VariableDeclarationKind;declarations: Array<VariableDeclarator>;declare: bool;}

export type VariableDeclarationKind = 'Var' | 'Const' | 'Let' | 'Using' | 'AwaitUsing'

export type VariableDeclarator = {span: Span;kind: VariableDeclarationKind;id: BindingPattern;init: (Expression) | null;definite: bool;}

export type EmptyStatement = {span: Span;}

export type ExpressionStatement = {span: Span;expression: Expression;}

export type IfStatement = {span: Span;test: Expression;consequent: Statement;alternate: (Statement) | null;}

export type DoWhileStatement = {span: Span;body: Statement;test: Expression;}

export type WhileStatement = {span: Span;test: Expression;body: Statement;}

export type ForStatement = {span: Span;init: (ForStatementInit) | null;test: (Expression) | null;update: (Expression) | null;body: Statement;scope_id: (ScopeId) | null;}

export type ForStatementInit = VariableDeclaration | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | Identifier | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | ClassExpression | ConditionalExpression | FunctionExpression | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression

export type ForInStatement = {span: Span;left: ForStatementLeft;right: Expression;body: Statement;scope_id: (ScopeId) | null;}

export type ForStatementLeft = VariableDeclaration | AssignmentTargetIdentifier | TSAsExpression | TSSatisfiesExpression | TSNonNullExpression | TSTypeAssertion | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression | ArrayAssignmentTarget | ObjectAssignmentTarget

export type ForOfStatement = {span: Span;await: bool;left: ForStatementLeft;right: Expression;body: Statement;scope_id: (ScopeId) | null;}

export type ContinueStatement = {span: Span;label: (LabelIdentifier) | null;}

export type BreakStatement = {span: Span;label: (LabelIdentifier) | null;}

export type ReturnStatement = {span: Span;argument: (Expression) | null;}

export type WithStatement = {span: Span;object: Expression;body: Statement;}

export type SwitchStatement = {span: Span;discriminant: Expression;cases: Array<SwitchCase>;scope_id: (ScopeId) | null;}

export type SwitchCase = {span: Span;test: (Expression) | null;consequent: Array<Statement>;}

export type LabeledStatement = {span: Span;label: LabelIdentifier;body: Statement;}

export type ThrowStatement = {span: Span;argument: Expression;}

export type TryStatement = {span: Span;block: BlockStatement;handler: (CatchClause) | null;finalizer: (BlockStatement) | null;}

export type CatchClause = {span: Span;param: (CatchParameter) | null;body: BlockStatement;scope_id: (ScopeId) | null;}

export type CatchParameter = {span: Span;pattern: BindingPattern;}

export type DebuggerStatement = {span: Span;}

export type BindingPattern = {kind: BindingPatternKind;type_annotation: (TSTypeAnnotation) | null;optional: bool;}

export type BindingPatternKind = BindingIdentifier | ObjectPattern | ArrayPattern | AssignmentPattern

export type AssignmentPattern = {span: Span;left: BindingPattern;right: Expression;}

export type ObjectPattern = {span: Span;properties: Array<BindingProperty>;rest: (BindingRestElement) | null;}

export type BindingProperty = {span: Span;key: PropertyKey;value: BindingPattern;shorthand: bool;computed: bool;}

export type ArrayPattern = {span: Span;elements: Array<(BindingPattern) | null>;rest: (BindingRestElement) | null;}

export type BindingRestElement = {span: Span;argument: BindingPattern;}

export type Function = {type: FunctionType;span: Span;id: (BindingIdentifier) | null;generator: bool;async: bool;declare: bool;type_parameters: (TSTypeParameterDeclaration) | null;this_param: (TSThisParameter) | null;params: FormalParameters;return_type: (TSTypeAnnotation) | null;body: (FunctionBody) | null;scope_id: (ScopeId) | null;}

export type FunctionType = 'FunctionDeclaration' | 'FunctionExpression' | 'TSDeclareFunction' | 'TSEmptyBodyFunctionExpression'

export type FormalParameters = {span: Span;kind: FormalParameterKind;items: Array<FormalParameter>;rest: (BindingRestElement) | null;}

export type FormalParameter = {span: Span;decorators: Array<Decorator>;pattern: BindingPattern;accessibility: (TSAccessibility) | null;readonly: bool;override: bool;}

export type FormalParameterKind = 'FormalParameter' | 'UniqueFormalParameters' | 'ArrowFormalParameters' | 'Signature'

export type FunctionBody = {span: Span;directives: Array<Directive>;statements: Array<Statement>;}

export type ArrowFunctionExpression = {span: Span;expression: bool;async: bool;type_parameters: (TSTypeParameterDeclaration) | null;params: FormalParameters;return_type: (TSTypeAnnotation) | null;body: FunctionBody;scope_id: (ScopeId) | null;}

export type YieldExpression = {span: Span;delegate: bool;argument: (Expression) | null;}

export type Class = {type: ClassType;span: Span;decorators: Array<Decorator>;id: (BindingIdentifier) | null;type_parameters: (TSTypeParameterDeclaration) | null;super_class: (Expression) | null;super_type_parameters: (TSTypeParameterInstantiation) | null;implements: (Array<TSClassImplements>) | null;body: ClassBody;abstract: bool;declare: bool;scope_id: (ScopeId) | null;}

export type ClassType = 'ClassDeclaration' | 'ClassExpression'

export type ClassBody = {span: Span;body: Array<ClassElement>;}

export type ClassElement = StaticBlock | MethodDefinition | PropertyDefinition | AccessorProperty | TSIndexSignature

export type MethodDefinition = {type: MethodDefinitionType;span: Span;decorators: Array<Decorator>;key: PropertyKey;value: Function;kind: MethodDefinitionKind;computed: bool;static: bool;override: bool;optional: bool;accessibility: (TSAccessibility) | null;}

export type MethodDefinitionType = 'MethodDefinition' | 'TSAbstractMethodDefinition'

export type PropertyDefinition = {type: PropertyDefinitionType;span: Span;decorators: Array<Decorator>;key: PropertyKey;value: (Expression) | null;computed: bool;static: bool;declare: bool;override: bool;optional: bool;definite: bool;readonly: bool;type_annotation: (TSTypeAnnotation) | null;accessibility: (TSAccessibility) | null;}

export type PropertyDefinitionType = 'PropertyDefinition' | 'TSAbstractPropertyDefinition'

export type MethodDefinitionKind = 'Constructor' | 'Method' | 'Get' | 'Set'

export type PrivateIdentifier = {span: Span;name: Atom;}

export type StaticBlock = {span: Span;body: Array<Statement>;scope_id: (ScopeId) | null;}

export type ModuleDeclaration = ImportDeclaration | ExportAllDeclaration | ExportDefaultDeclaration | ExportNamedDeclaration | TSExportAssignment | TSNamespaceExportDeclaration

export type AccessorPropertyType = 'AccessorProperty' | 'TSAbstractAccessorProperty'

export type AccessorProperty = {type: AccessorPropertyType;span: Span;decorators: Array<Decorator>;key: PropertyKey;value: (Expression) | null;computed: bool;static: bool;definite: bool;type_annotation: (TSTypeAnnotation) | null;accessibility: (TSAccessibility) | null;}

export type ImportExpression = {span: Span;source: Expression;arguments: Array<Expression>;}

export type ImportDeclaration = {span: Span;specifiers: (Array<ImportDeclarationSpecifier>) | null;source: StringLiteral;with_clause: (WithClause) | null;import_kind: ImportOrExportKind;}

export type ImportDeclarationSpecifier = ImportSpecifier | ImportDefaultSpecifier | ImportNamespaceSpecifier

export type ImportSpecifier = {span: Span;imported: ModuleExportName;local: BindingIdentifier;import_kind: ImportOrExportKind;}

export type ImportDefaultSpecifier = {span: Span;local: BindingIdentifier;}

export type ImportNamespaceSpecifier = {span: Span;local: BindingIdentifier;}

export type WithClause = {span: Span;attributes_keyword: IdentifierName;with_entries: Array<ImportAttribute>;}

export type ImportAttribute = {span: Span;key: ImportAttributeKey;value: StringLiteral;}

export type ImportAttributeKey = Identifier | StringLiteral

export type ExportNamedDeclaration = {span: Span;declaration: (Declaration) | null;specifiers: Array<ExportSpecifier>;source: (StringLiteral) | null;export_kind: ImportOrExportKind;with_clause: (WithClause) | null;}

export type ExportDefaultDeclaration = {span: Span;declaration: ExportDefaultDeclarationKind;exported: ModuleExportName;}

export type ExportAllDeclaration = {span: Span;exported: (ModuleExportName) | null;source: StringLiteral;with_clause: (WithClause) | null;export_kind: ImportOrExportKind;}

export type ExportSpecifier = {span: Span;local: ModuleExportName;exported: ModuleExportName;export_kind: ImportOrExportKind;}

export type ExportDefaultDeclarationKind = FunctionDeclaration | ClassDeclaration | TSInterfaceDeclaration | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | Identifier | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | ClassExpression | ConditionalExpression | FunctionExpression | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression

export type ModuleExportName = IdentifierName | IdentifierReference | StringLiteral

export type TSThisParameter = {span: Span;this_span: Span;type_annotation: (TSTypeAnnotation) | null;}

export type TSEnumDeclaration = {span: Span;id: BindingIdentifier;members: Array<TSEnumMember>;const: bool;declare: bool;scope_id: (ScopeId) | null;}

export type TSEnumMember = {span: Span;id: TSEnumMemberName;initializer: (Expression) | null;}

export type TSEnumMemberName = StaticIdentifier | StaticStringLiteral | StaticTemplateLiteral | StaticNumericLiteral | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | Identifier | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | ClassExpression | ConditionalExpression | FunctionExpression | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression

export type TSTypeAnnotation = {span: Span;type_annotation: TSType;}

export type TSLiteralType = {span: Span;literal: TSLiteral;}

export type TSLiteral = BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | UnaryExpression

export type TSType = TSAnyKeyword | TSBigIntKeyword | TSBooleanKeyword | TSIntrinsicKeyword | TSNeverKeyword | TSNullKeyword | TSNumberKeyword | TSObjectKeyword | TSStringKeyword | TSSymbolKeyword | TSUndefinedKeyword | TSUnknownKeyword | TSVoidKeyword | TSArrayType | TSConditionalType | TSConstructorType | TSFunctionType | TSImportType | TSIndexedAccessType | TSInferType | TSIntersectionType | TSLiteralType | TSMappedType | TSNamedTupleMember | TSQualifiedName | TSTemplateLiteralType | TSThisType | TSTupleType | TSTypeLiteral | TSTypeOperatorType | TSTypePredicate | TSTypeQuery | TSTypeReference | TSUnionType | TSParenthesizedType | JSDocNullableType | JSDocNonNullableType | JSDocUnknownType

export type TSConditionalType = {span: Span;check_type: TSType;extends_type: TSType;true_type: TSType;false_type: TSType;scope_id: (ScopeId) | null;}

export type TSUnionType = {span: Span;types: Array<TSType>;}

export type TSIntersectionType = {span: Span;types: Array<TSType>;}

export type TSParenthesizedType = {span: Span;type_annotation: TSType;}

export type TSTypeOperator = {span: Span;operator: TSTypeOperatorOperator;type_annotation: TSType;}

export type TSTypeOperatorOperator = 'Keyof' | 'Unique' | 'Readonly'

export type TSArrayType = {span: Span;element_type: TSType;}

export type TSIndexedAccessType = {span: Span;object_type: TSType;index_type: TSType;}

export type TSTupleType = {span: Span;element_types: Array<TSTupleElement>;}

export type TSNamedTupleMember = {span: Span;element_type: TSTupleElement;label: IdentifierName;optional: bool;}

export type TSOptionalType = {span: Span;type_annotation: TSType;}

export type TSRestType = {span: Span;type_annotation: TSType;}

export type TSTupleElement = TSOptionalType | TSRestType | TSAnyKeyword | TSBigIntKeyword | TSBooleanKeyword | TSIntrinsicKeyword | TSNeverKeyword | TSNullKeyword | TSNumberKeyword | TSObjectKeyword | TSStringKeyword | TSSymbolKeyword | TSUndefinedKeyword | TSUnknownKeyword | TSVoidKeyword | TSArrayType | TSConditionalType | TSConstructorType | TSFunctionType | TSImportType | TSIndexedAccessType | TSInferType | TSIntersectionType | TSLiteralType | TSMappedType | TSNamedTupleMember | TSQualifiedName | TSTemplateLiteralType | TSThisType | TSTupleType | TSTypeLiteral | TSTypeOperatorType | TSTypePredicate | TSTypeQuery | TSTypeReference | TSUnionType | TSParenthesizedType | JSDocNullableType | JSDocNonNullableType | JSDocUnknownType

export type TSAnyKeyword = {span: Span;}

export type TSStringKeyword = {span: Span;}

export type TSBooleanKeyword = {span: Span;}

export type TSNumberKeyword = {span: Span;}

export type TSNeverKeyword = {span: Span;}

export type TSIntrinsicKeyword = {span: Span;}

export type TSUnknownKeyword = {span: Span;}

export type TSNullKeyword = {span: Span;}

export type TSUndefinedKeyword = {span: Span;}

export type TSVoidKeyword = {span: Span;}

export type TSSymbolKeyword = {span: Span;}

export type TSThisType = {span: Span;}

export type TSObjectKeyword = {span: Span;}

export type TSBigIntKeyword = {span: Span;}

export type TSTypeReference = {span: Span;type_name: TSTypeName;type_parameters: (TSTypeParameterInstantiation) | null;}

export type TSTypeName = IdentifierReference | QualifiedName

export type TSQualifiedName = {span: Span;left: TSTypeName;right: IdentifierName;}

export type TSTypeParameterInstantiation = {span: Span;params: Array<TSType>;}

export type TSTypeParameter = {span: Span;name: BindingIdentifier;constraint: (TSType) | null;default: (TSType) | null;in: bool;out: bool;const: bool;}

export type TSTypeParameterDeclaration = {span: Span;params: Array<TSTypeParameter>;}

export type TSTypeAliasDeclaration = {span: Span;id: BindingIdentifier;type_parameters: (TSTypeParameterDeclaration) | null;type_annotation: TSType;declare: bool;scope_id: (ScopeId) | null;}

export type TSAccessibility = 'Private' | 'Protected' | 'Public'

export type TSClassImplements = {span: Span;expression: TSTypeName;type_parameters: (TSTypeParameterInstantiation) | null;}

export type TSInterfaceDeclaration = {span: Span;id: BindingIdentifier;extends: (Array<TSInterfaceHeritage>) | null;type_parameters: (TSTypeParameterDeclaration) | null;body: TSInterfaceBody;declare: bool;scope_id: (ScopeId) | null;}

export type TSInterfaceBody = {span: Span;body: Array<TSSignature>;}

export type TSPropertySignature = {span: Span;computed: bool;optional: bool;readonly: bool;key: PropertyKey;type_annotation: (TSTypeAnnotation) | null;}

export type TSSignature = TSIndexSignature | TSPropertySignature | TSCallSignatureDeclaration | TSConstructSignatureDeclaration | TSMethodSignature

export type TSIndexSignature = {span: Span;parameters: Array<TSIndexSignatureName>;type_annotation: TSTypeAnnotation;readonly: bool;}

export type TSCallSignatureDeclaration = {span: Span;this_param: (TSThisParameter) | null;params: FormalParameters;return_type: (TSTypeAnnotation) | null;type_parameters: (TSTypeParameterDeclaration) | null;}

export type TSMethodSignatureKind = 'Method' | 'Get' | 'Set'

export type TSMethodSignature = {span: Span;key: PropertyKey;computed: bool;optional: bool;kind: TSMethodSignatureKind;this_param: (TSThisParameter) | null;params: FormalParameters;return_type: (TSTypeAnnotation) | null;type_parameters: (TSTypeParameterDeclaration) | null;scope_id: (ScopeId) | null;}

export type TSConstructSignatureDeclaration = {span: Span;params: FormalParameters;return_type: (TSTypeAnnotation) | null;type_parameters: (TSTypeParameterDeclaration) | null;scope_id: (ScopeId) | null;}

export type TSIndexSignatureName = {span: Span;name: Atom;type_annotation: TSTypeAnnotation;}

export type TSInterfaceHeritage = {span: Span;expression: Expression;type_parameters: (TSTypeParameterInstantiation) | null;}

export type TSTypePredicate = {span: Span;parameter_name: TSTypePredicateName;asserts: bool;type_annotation: (TSTypeAnnotation) | null;}

export type TSTypePredicateName = Identifier | This

export type TSModuleDeclaration = {span: Span;id: TSModuleDeclarationName;body: (TSModuleDeclarationBody) | null;kind: TSModuleDeclarationKind;declare: bool;scope_id: (ScopeId) | null;}

export type TSModuleDeclarationKind = 'Global' | 'Module' | 'Namespace'

export type TSModuleDeclarationName = Identifier | StringLiteral

export type TSModuleDeclarationBody = TSModuleDeclaration | TSModuleBlock

export type TSModuleBlock = {span: Span;directives: Array<Directive>;body: Array<Statement>;}

export type TSTypeLiteral = {span: Span;members: Array<TSSignature>;}

export type TSInferType = {span: Span;type_parameter: TSTypeParameter;}

export type TSTypeQuery = {span: Span;expr_name: TSTypeQueryExprName;type_parameters: (TSTypeParameterInstantiation) | null;}

export type TSTypeQueryExprName = TSImportType | IdentifierReference | QualifiedName

export type TSImportType = {span: Span;is_type_of: bool;parameter: TSType;qualifier: (TSTypeName) | null;attributes: (TSImportAttributes) | null;type_parameters: (TSTypeParameterInstantiation) | null;}

export type TSImportAttributes = {span: Span;attributes_keyword: IdentifierName;elements: Array<TSImportAttribute>;}

export type TSImportAttribute = {span: Span;name: TSImportAttributeName;value: Expression;}

export type TSImportAttributeName = Identifier | StringLiteral

export type TSFunctionType = {span: Span;this_param: (TSThisParameter) | null;params: FormalParameters;return_type: TSTypeAnnotation;type_parameters: (TSTypeParameterDeclaration) | null;}

export type TSConstructorType = {span: Span;abstract: bool;params: FormalParameters;return_type: TSTypeAnnotation;type_parameters: (TSTypeParameterDeclaration) | null;}

export type TSMappedType = {span: Span;type_parameter: TSTypeParameter;name_type: (TSType) | null;type_annotation: (TSType) | null;optional: TSMappedTypeModifierOperator;readonly: TSMappedTypeModifierOperator;scope_id: (ScopeId) | null;}

export type TSMappedTypeModifierOperator = 'True' | 'Plus' | 'Minus' | 'None'

export type TSTemplateLiteralType = {span: Span;quasis: Array<TemplateElement>;types: Array<TSType>;}

export type TSAsExpression = {span: Span;expression: Expression;type_annotation: TSType;}

export type TSSatisfiesExpression = {span: Span;expression: Expression;type_annotation: TSType;}

export type TSTypeAssertion = {span: Span;expression: Expression;type_annotation: TSType;}

export type TSImportEqualsDeclaration = {span: Span;id: BindingIdentifier;module_reference: TSModuleReference;import_kind: ImportOrExportKind;}

export type TSModuleReference = ExternalModuleReference | IdentifierReference | QualifiedName

export type TSExternalModuleReference = {span: Span;expression: StringLiteral;}

export type TSNonNullExpression = {span: Span;expression: Expression;}

export type Decorator = {span: Span;expression: Expression;}

export type TSExportAssignment = {span: Span;expression: Expression;}

export type TSNamespaceExportDeclaration = {span: Span;id: IdentifierName;}

export type TSInstantiationExpression = {span: Span;expression: Expression;type_parameters: TSTypeParameterInstantiation;}

export type ImportOrExportKind = 'Value' | 'Type'

export type JSDocNullableType = {span: Span;type_annotation: TSType;postfix: bool;}

export type JSDocNonNullableType = {span: Span;type_annotation: TSType;postfix: bool;}

export type JSDocUnknownType = {span: Span;}

export type JSXElement = {span: Span;opening_element: JSXOpeningElement;closing_element: (JSXClosingElement) | null;children: Array<JSXChild>;}

export type JSXOpeningElement = {span: Span;self_closing: bool;name: JSXElementName;attributes: Array<JSXAttributeItem>;type_parameters: (TSTypeParameterInstantiation) | null;}

export type JSXClosingElement = {span: Span;name: JSXElementName;}

export type JSXFragment = {span: Span;opening_fragment: JSXOpeningFragment;closing_fragment: JSXClosingFragment;children: Array<JSXChild>;}

export type JSXOpeningFragment = {span: Span;}

export type JSXClosingFragment = {span: Span;}

export type JSXElementName = Identifier | IdentifierReference | NamespacedName | MemberExpression | ThisExpression

export type JSXNamespacedName = {span: Span;namespace: JSXIdentifier;property: JSXIdentifier;}

export type JSXMemberExpression = {span: Span;object: JSXMemberExpressionObject;property: JSXIdentifier;}

export type JSXMemberExpressionObject = IdentifierReference | MemberExpression | ThisExpression

export type JSXExpressionContainer = {span: Span;expression: JSXExpression;}

export type JSXExpression = EmptyExpression | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | Identifier | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | ClassExpression | ConditionalExpression | FunctionExpression | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression

export type JSXEmptyExpression = {span: Span;}

export type JSXAttributeItem = Attribute | SpreadAttribute

export type JSXAttribute = {span: Span;name: JSXAttributeName;value: (JSXAttributeValue) | null;}

export type JSXSpreadAttribute = {span: Span;argument: Expression;}

export type JSXAttributeName = Identifier | NamespacedName

export type JSXAttributeValue = StringLiteral | ExpressionContainer | Element | Fragment

export type JSXIdentifier = {span: Span;name: Atom;}

export type JSXChild = Text | Element | Fragment | ExpressionContainer | Spread

export type JSXSpreadChild = {span: Span;expression: Expression;}

export type JSXText = {span: Span;value: Atom;}

export type NumberBase = 'Float' | 'Decimal' | 'Binary' | 'Octal' | 'Hex'

export type BigintBase = 'Decimal' | 'Binary' | 'Octal' | 'Hex'

export type AssignmentOperator = 'Assign' | 'Addition' | 'Subtraction' | 'Multiplication' | 'Division' | 'Remainder' | 'ShiftLeft' | 'ShiftRight' | 'ShiftRightZeroFill' | 'BitwiseOR' | 'BitwiseXOR' | 'BitwiseAnd' | 'LogicalAnd' | 'LogicalOr' | 'LogicalNullish' | 'Exponential'

export type BinaryOperator = 'Equality' | 'Inequality' | 'StrictEquality' | 'StrictInequality' | 'LessThan' | 'LessEqualThan' | 'GreaterThan' | 'GreaterEqualThan' | 'ShiftLeft' | 'ShiftRight' | 'ShiftRightZeroFill' | 'Addition' | 'Subtraction' | 'Multiplication' | 'Division' | 'Remainder' | 'BitwiseOR' | 'BitwiseXOR' | 'BitwiseAnd' | 'In' | 'Instanceof' | 'Exponential'

export type LogicalOperator = 'Or' | 'And' | 'Coalesce'

export type UnaryOperator = 'UnaryNegation' | 'UnaryPlus' | 'LogicalNot' | 'BitwiseNot' | 'Typeof' | 'Void' | 'Delete'

export type UpdateOperator = 'Increment' | 'Decrement'

export type Span = {start: u32;end: u32;}

export type SourceType = {language: Language;module_kind: ModuleKind;variant: LanguageVariant;}

export type Language = 'JavaScript' | 'TypeScript' | 'TypeScriptDefinition'

export type ModuleKind = 'Script' | 'Module' | 'Unambiguous'

export type LanguageVariant = 'Standard' | 'Jsx'

export type Pattern = {span: Span;body: Disjunction;}

export type Disjunction = {span: Span;body: Array<Alternative>;}

export type Alternative = {span: Span;body: Array<Term>;}

export type Term = BoundaryAssertion | LookAroundAssertion | Quantifier | Character | Dot | CharacterClassEscape | UnicodePropertyEscape | CharacterClass | CapturingGroup | IgnoreGroup | IndexedReference | NamedReference

export type BoundaryAssertion = {span: Span;kind: BoundaryAssertionKind;}

export type BoundaryAssertionKind = 'Start' | 'End' | 'Boundary' | 'NegativeBoundary'

export type LookAroundAssertion = {span: Span;kind: LookAroundAssertionKind;body: Disjunction;}

export type LookAroundAssertionKind = 'Lookahead' | 'NegativeLookahead' | 'Lookbehind' | 'NegativeLookbehind'

export type Quantifier = {span: Span;min: u64;max: (u64) | null;greedy: bool;body: Term;}

export type Character = {span: Span;kind: CharacterKind;value: u32;}

export type CharacterKind = 'ControlLetter' | 'HexadecimalEscape' | 'Identifier' | 'Null' | 'Octal1' | 'Octal2' | 'Octal3' | 'SingleEscape' | 'Symbol' | 'UnicodeEscape'

export type CharacterClassEscape = {span: Span;kind: CharacterClassEscapeKind;}

export type CharacterClassEscapeKind = 'D' | 'NegativeD' | 'S' | 'NegativeS' | 'W' | 'NegativeW'

export type UnicodePropertyEscape = {span: Span;negative: bool;strings: bool;name: Atom;value: (Atom) | null;}

export type Dot = {span: Span;}

export type CharacterClass = {span: Span;negative: bool;strings: bool;kind: CharacterClassContentsKind;body: Array<CharacterClassContents>;}

export type CharacterClassContentsKind = 'Union' | 'Intersection' | 'Subtraction'

export type CharacterClassContents = CharacterClassRange | CharacterClassEscape | UnicodePropertyEscape | Character | NestedCharacterClass | ClassStringDisjunction

export type CharacterClassRange = {span: Span;min: Character;max: Character;}

export type ClassStringDisjunction = {span: Span;strings: bool;body: Array<ClassString>;}

export type ClassString = {span: Span;strings: bool;body: Array<Character>;}

export type CapturingGroup = {span: Span;name: (Atom) | null;body: Disjunction;}

export type IgnoreGroup = {span: Span;modifiers: (Modifiers) | null;body: Disjunction;}

export type Modifiers = {span: Span;enabling: (Modifier) | null;disabling: (Modifier) | null;}

export type Modifier = {ignore_case: bool;multiline: bool;sticky: bool;}

export type IndexedReference = {span: Span;index: u32;}

export type NamedReference = {span: Span;name: Atom;}

