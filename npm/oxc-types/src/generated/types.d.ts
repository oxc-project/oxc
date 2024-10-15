// To edit this generated file you have to edit `tasks/ast_tools/src/generators/typescript.rs`
// Auto-generated code, DO NOT EDIT DIRECTLY!

export type BooleanLiteral = {type: 'BooleanLiteral';span: Span;value: boolean;}

export type NullLiteral = {type: 'NullLiteral';span: Span;}

export type NumericLiteral = {type: 'NumericLiteral';span: Span;value: number;raw: string;}

export type BigIntLiteral = {type: 'BigIntLiteral';span: Span;raw: string;}

export type RegExpLiteral = {type: 'RegExpLiteral';span: Span;value: EmptyObject;regex: RegExp;}

export type RegExp = {pattern: RegExpPattern;flags: RegExpFlags;}

export type RegExpPattern = Raw | Invalid | Pattern

export type EmptyObject = {}

export type StringLiteral = {type: 'StringLiteral';span: Span;value: string;}

export type Program = {type: 'Program';span: Span;source_type: SourceType;hashbang: (Hashbang) | null;directives: Array<Directive>;body: Array<Statement>;}

export type Expression = BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | Identifier | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | ClassExpression | ConditionalExpression | FunctionExpression | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression

export type IdentifierName = {type: 'Identifier';span: Span;name: string;}

export type IdentifierReference = {type: 'Identifier';span: Span;name: string;}

export type BindingIdentifier = {type: 'Identifier';span: Span;name: string;}

export type LabelIdentifier = {type: 'Identifier';span: Span;name: string;}

export type ThisExpression = {type: 'ThisExpression';span: Span;}

export type ArrayExpression = {type: 'ArrayExpression';span: Span;elements: Array<ArrayExpressionElement>;}

export type ArrayExpressionElement = SpreadElement | Elision | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | Identifier | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | ClassExpression | ConditionalExpression | FunctionExpression | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression

export type Elision = {type: 'Elision';span: Span;}

export type ObjectExpression = {type: 'ObjectExpression';span: Span;properties: Array<ObjectPropertyKind>;}

export type ObjectPropertyKind = ObjectProperty | SpreadProperty

export type ObjectProperty = {type: 'ObjectProperty';span: Span;kind: PropertyKind;key: PropertyKey;value: Expression;init: (Expression) | null;method: boolean;shorthand: boolean;computed: boolean;}

export type PropertyKey = StaticIdentifier | PrivateIdentifier | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | Identifier | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | ClassExpression | ConditionalExpression | FunctionExpression | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression

export type PropertyKind = 'init' | 'get' | 'set'

export type TemplateLiteral = {type: 'TemplateLiteral';span: Span;quasis: Array<TemplateElement>;expressions: Array<Expression>;}

export type TaggedTemplateExpression = {type: 'TaggedTemplateExpression';span: Span;tag: Expression;quasi: TemplateLiteral;type_parameters: (TSTypeParameterInstantiation) | null;}

export type TemplateElement = {type: 'TemplateElement';span: Span;tail: boolean;value: TemplateElementValue;}

export type TemplateElementValue = {raw: string;cooked: (string) | null;}

export type MemberExpression = ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression

export type ComputedMemberExpression = {type: 'ComputedMemberExpression';span: Span;object: Expression;expression: Expression;optional: boolean;}

export type StaticMemberExpression = {type: 'StaticMemberExpression';span: Span;object: Expression;property: Identifier;optional: boolean;}

export type PrivateFieldExpression = {type: 'PrivateFieldExpression';span: Span;object: Expression;field: PrivateIdentifier;optional: boolean;}

export type CallExpression = {type: 'CallExpression';span: Span;callee: Expression;type_parameters: (TSTypeParameterInstantiation) | null;arguments: Array<Argument>;optional: boolean;}

export type NewExpression = {type: 'NewExpression';span: Span;callee: Expression;arguments: Array<Argument>;type_parameters: (TSTypeParameterInstantiation) | null;}

export type MetaProperty = {type: 'MetaProperty';span: Span;meta: Identifier;property: Identifier;}

export type SpreadElement = {type: 'SpreadElement';span: Span;argument: Expression;}

export type Argument = SpreadElement | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | Identifier | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | ClassExpression | ConditionalExpression | FunctionExpression | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression

export type UpdateExpression = {type: 'UpdateExpression';span: Span;operator: UpdateOperator;prefix: boolean;argument: SimpleAssignmentTarget;}

export type UnaryExpression = {type: 'UnaryExpression';span: Span;operator: UnaryOperator;argument: Expression;}

export type BinaryExpression = {type: 'BinaryExpression';span: Span;left: Expression;operator: BinaryOperator;right: Expression;}

export type PrivateInExpression = {type: 'PrivateInExpression';span: Span;left: PrivateIdentifier;operator: BinaryOperator;right: Expression;}

export type LogicalExpression = {type: 'LogicalExpression';span: Span;left: Expression;operator: LogicalOperator;right: Expression;}

export type ConditionalExpression = {type: 'ConditionalExpression';span: Span;test: Expression;consequent: Expression;alternate: Expression;}

export type AssignmentExpression = {type: 'AssignmentExpression';span: Span;operator: AssignmentOperator;left: AssignmentTarget;right: Expression;}

export type AssignmentTarget = AssignmentTargetIdentifier | TSAsExpression | TSSatisfiesExpression | TSNonNullExpression | TSTypeAssertion | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression | ArrayAssignmentTarget | ObjectAssignmentTarget

export type SimpleAssignmentTarget = AssignmentTargetIdentifier | TSAsExpression | TSSatisfiesExpression | TSNonNullExpression | TSTypeAssertion | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression

export type AssignmentTargetPattern = ArrayAssignmentTarget | ObjectAssignmentTarget

export type ArrayAssignmentTarget = {type: 'ArrayAssignmentTarget';span: Span;elements: Array<(AssignmentTargetMaybeDefault) | null>;}

export type ObjectAssignmentTarget = {type: 'ObjectAssignmentTarget';span: Span;properties: Array<AssignmentTargetProperty>;}

export type AssignmentTargetRest = {type: 'RestElement';span: Span;target: AssignmentTarget;}

export type AssignmentTargetMaybeDefault = AssignmentTargetWithDefault | AssignmentTargetIdentifier | TSAsExpression | TSSatisfiesExpression | TSNonNullExpression | TSTypeAssertion | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression | ArrayAssignmentTarget | ObjectAssignmentTarget

export type AssignmentTargetWithDefault = {type: 'AssignmentTargetWithDefault';span: Span;binding: AssignmentTarget;init: Expression;}

export type AssignmentTargetProperty = AssignmentTargetPropertyIdentifier | AssignmentTargetPropertyProperty

export type AssignmentTargetPropertyIdentifier = {type: 'AssignmentTargetPropertyIdentifier';span: Span;binding: Identifier;init: (Expression) | null;}

export type AssignmentTargetPropertyProperty = {type: 'AssignmentTargetPropertyProperty';span: Span;name: PropertyKey;binding: AssignmentTargetMaybeDefault;}

export type SequenceExpression = {type: 'SequenceExpression';span: Span;expressions: Array<Expression>;}

export type Super = {type: 'Super';span: Span;}

export type AwaitExpression = {type: 'AwaitExpression';span: Span;argument: Expression;}

export type ChainExpression = {type: 'ChainExpression';span: Span;expression: ChainElement;}

export type ChainElement = CallExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression

export type ParenthesizedExpression = {type: 'ParenthesizedExpression';span: Span;expression: Expression;}

export type Statement = BlockStatement | BreakStatement | ContinueStatement | DebuggerStatement | DoWhileStatement | EmptyStatement | ExpressionStatement | ForInStatement | ForOfStatement | ForStatement | IfStatement | LabeledStatement | ReturnStatement | SwitchStatement | ThrowStatement | TryStatement | WhileStatement | WithStatement | VariableDeclaration | FunctionDeclaration | ClassDeclaration | TSTypeAliasDeclaration | TSInterfaceDeclaration | TSEnumDeclaration | TSModuleDeclaration | TSImportEqualsDeclaration | ImportDeclaration | ExportAllDeclaration | ExportDefaultDeclaration | ExportNamedDeclaration | TSExportAssignment | TSNamespaceExportDeclaration

export type Directive = {type: 'Directive';span: Span;expression: StringLiteral;directive: string;}

export type Hashbang = {type: 'Hashbang';span: Span;value: string;}

export type BlockStatement = {type: 'BlockStatement';span: Span;body: Array<Statement>;}

export type Declaration = VariableDeclaration | FunctionDeclaration | ClassDeclaration | TSTypeAliasDeclaration | TSInterfaceDeclaration | TSEnumDeclaration | TSModuleDeclaration | TSImportEqualsDeclaration

export type VariableDeclaration = {type: 'VariableDeclaration';span: Span;kind: VariableDeclarationKind;declarations: Array<VariableDeclarator>;declare: boolean;}

export type VariableDeclarationKind = 'var' | 'const' | 'let' | 'using' | 'await using'

export type VariableDeclarator = {type: 'VariableDeclarator';span: Span;id: BindingPattern;init: (Expression) | null;definite: boolean;}

export type EmptyStatement = {type: 'EmptyStatement';span: Span;}

export type ExpressionStatement = {type: 'ExpressionStatement';span: Span;expression: Expression;}

export type IfStatement = {type: 'IfStatement';span: Span;test: Expression;consequent: Statement;alternate: (Statement) | null;}

export type DoWhileStatement = {type: 'DoWhileStatement';span: Span;body: Statement;test: Expression;}

export type WhileStatement = {type: 'WhileStatement';span: Span;test: Expression;body: Statement;}

export type ForStatement = {type: 'ForStatement';span: Span;init: (ForStatementInit) | null;test: (Expression) | null;update: (Expression) | null;body: Statement;}

export type ForStatementInit = VariableDeclaration | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | Identifier | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | ClassExpression | ConditionalExpression | FunctionExpression | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression

export type ForInStatement = {type: 'ForInStatement';span: Span;left: ForStatementLeft;right: Expression;body: Statement;}

export type ForStatementLeft = VariableDeclaration | AssignmentTargetIdentifier | TSAsExpression | TSSatisfiesExpression | TSNonNullExpression | TSTypeAssertion | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression | ArrayAssignmentTarget | ObjectAssignmentTarget

export type ForOfStatement = {type: 'ForOfStatement';span: Span;await: boolean;left: ForStatementLeft;right: Expression;body: Statement;}

export type ContinueStatement = {type: 'ContinueStatement';span: Span;label: (Identifier) | null;}

export type BreakStatement = {type: 'BreakStatement';span: Span;label: (Identifier) | null;}

export type ReturnStatement = {type: 'ReturnStatement';span: Span;argument: (Expression) | null;}

export type WithStatement = {type: 'WithStatement';span: Span;object: Expression;body: Statement;}

export type SwitchStatement = {type: 'SwitchStatement';span: Span;discriminant: Expression;cases: Array<SwitchCase>;}

export type SwitchCase = {type: 'SwitchCase';span: Span;test: (Expression) | null;consequent: Array<Statement>;}

export type LabeledStatement = {type: 'LabeledStatement';span: Span;label: Identifier;body: Statement;}

export type ThrowStatement = {type: 'ThrowStatement';span: Span;argument: Expression;}

export type TryStatement = {type: 'TryStatement';span: Span;block: BlockStatement;handler: (CatchClause) | null;finalizer: (BlockStatement) | null;}

export type CatchClause = {type: 'CatchClause';span: Span;param: (CatchParameter) | null;body: BlockStatement;}

export type CatchParameter = {type: 'CatchParameter';span: Span;pattern: BindingPattern;}

export type DebuggerStatement = {type: 'DebuggerStatement';span: Span;}

export type BindingPattern = {kind: BindingPatternKind;type_annotation: (TSTypeAnnotation) | null;optional: boolean;}

export type BindingPatternKind = Identifier | ObjectPattern | ArrayPattern | AssignmentPattern

export type AssignmentPattern = {type: 'AssignmentPattern';span: Span;left: BindingPattern;right: Expression;}

export type ObjectPattern = {type: 'ObjectPattern';span: Span;properties: Array<BindingProperty>;}

export type BindingProperty = {type: 'BindingProperty';span: Span;key: PropertyKey;value: BindingPattern;shorthand: boolean;computed: boolean;}

export type ArrayPattern = {type: 'ArrayPattern';span: Span;elements: Array<(BindingPattern) | null>;}

export type BindingRestElement = {type: 'RestElement';span: Span;argument: BindingPattern;}

export type Function = {type: FunctionType;span: Span;id: (Identifier) | null;generator: boolean;async: boolean;declare: boolean;type_parameters: (TSTypeParameterDeclaration) | null;this_param: (TSThisParameter) | null;params: FormalParameters;return_type: (TSTypeAnnotation) | null;body: (FunctionBody) | null;}

export type FunctionType = 'FunctionDeclaration' | 'FunctionExpression' | 'TSDeclareFunction' | 'TSEmptyBodyFunctionExpression'

export type FormalParameters = {type: 'FormalParameters';span: Span;kind: FormalParameterKind;items: Array<FormalParameter>;}

export type FormalParameter = {type: 'FormalParameter';span: Span;decorators: Array<Decorator>;pattern: BindingPattern;accessibility: (TSAccessibility) | null;readonly: boolean;override: boolean;}

export type FormalParameterKind = 'FormalParameter' | 'UniqueFormalParameters' | 'ArrowFormalParameters' | 'Signature'

export type FunctionBody = {type: 'FunctionBody';span: Span;directives: Array<Directive>;statements: Array<Statement>;}

export type ArrowFunctionExpression = {type: 'ArrowFunctionExpression';span: Span;expression: boolean;async: boolean;type_parameters: (TSTypeParameterDeclaration) | null;params: FormalParameters;return_type: (TSTypeAnnotation) | null;body: FunctionBody;}

export type YieldExpression = {type: 'YieldExpression';span: Span;delegate: boolean;argument: (Expression) | null;}

export type Class = {type: ClassType;span: Span;decorators: Array<Decorator>;id: (Identifier) | null;type_parameters: (TSTypeParameterDeclaration) | null;super_class: (Expression) | null;super_type_parameters: (TSTypeParameterInstantiation) | null;implements: (Array<TSClassImplements>) | null;body: ClassBody;abstract: boolean;declare: boolean;}

export type ClassType = 'ClassDeclaration' | 'ClassExpression'

export type ClassBody = {type: 'ClassBody';span: Span;body: Array<ClassElement>;}

export type ClassElement = StaticBlock | MethodDefinition | PropertyDefinition | AccessorProperty | TSIndexSignature

export type MethodDefinition = {type: MethodDefinitionType;span: Span;decorators: Array<Decorator>;key: PropertyKey;value: Function;kind: MethodDefinitionKind;computed: boolean;static: boolean;override: boolean;optional: boolean;accessibility: (TSAccessibility) | null;}

export type MethodDefinitionType = 'MethodDefinition' | 'TSAbstractMethodDefinition'

export type PropertyDefinition = {type: PropertyDefinitionType;span: Span;decorators: Array<Decorator>;key: PropertyKey;value: (Expression) | null;computed: boolean;static: boolean;declare: boolean;override: boolean;optional: boolean;definite: boolean;readonly: boolean;type_annotation: (TSTypeAnnotation) | null;accessibility: (TSAccessibility) | null;}

export type PropertyDefinitionType = 'PropertyDefinition' | 'TSAbstractPropertyDefinition'

export type MethodDefinitionKind = 'constructor' | 'method' | 'get' | 'set'

export type PrivateIdentifier = {type: 'PrivateIdentifier';span: Span;name: string;}

export type StaticBlock = {type: 'StaticBlock';span: Span;body: Array<Statement>;}

export type ModuleDeclaration = ImportDeclaration | ExportAllDeclaration | ExportDefaultDeclaration | ExportNamedDeclaration | TSExportAssignment | TSNamespaceExportDeclaration

export type AccessorPropertyType = 'AccessorProperty' | 'TSAbstractAccessorProperty'

export type AccessorProperty = {type: AccessorPropertyType;span: Span;decorators: Array<Decorator>;key: PropertyKey;value: (Expression) | null;computed: boolean;static: boolean;definite: boolean;type_annotation: (TSTypeAnnotation) | null;accessibility: (TSAccessibility) | null;}

export type ImportExpression = {type: 'ImportExpression';span: Span;source: Expression;arguments: Array<Expression>;}

export type ImportDeclaration = {type: 'ImportDeclaration';span: Span;specifiers: (Array<ImportDeclarationSpecifier>) | null;source: StringLiteral;with_clause: (WithClause) | null;import_kind: ImportOrExportKind;}

export type ImportDeclarationSpecifier = ImportSpecifier | ImportDefaultSpecifier | ImportNamespaceSpecifier

export type ImportSpecifier = {type: 'ImportSpecifier';span: Span;imported: ModuleExportName;local: Identifier;import_kind: ImportOrExportKind;}

export type ImportDefaultSpecifier = {type: 'ImportDefaultSpecifier';span: Span;local: Identifier;}

export type ImportNamespaceSpecifier = {type: 'ImportNamespaceSpecifier';span: Span;local: Identifier;}

export type WithClause = {type: 'WithClause';span: Span;attributes_keyword: Identifier;with_entries: Array<ImportAttribute>;}

export type ImportAttribute = {type: 'ImportAttribute';span: Span;key: ImportAttributeKey;value: StringLiteral;}

export type ImportAttributeKey = Identifier | StringLiteral

export type ExportNamedDeclaration = {type: 'ExportNamedDeclaration';span: Span;declaration: (Declaration) | null;specifiers: Array<ExportSpecifier>;source: (StringLiteral) | null;export_kind: ImportOrExportKind;with_clause: (WithClause) | null;}

export type ExportDefaultDeclaration = {type: 'ExportDefaultDeclaration';span: Span;declaration: ExportDefaultDeclarationKind;exported: ModuleExportName;}

export type ExportAllDeclaration = {type: 'ExportAllDeclaration';span: Span;exported: (ModuleExportName) | null;source: StringLiteral;with_clause: (WithClause) | null;export_kind: ImportOrExportKind;}

export type ExportSpecifier = {type: 'ExportSpecifier';span: Span;local: ModuleExportName;exported: ModuleExportName;export_kind: ImportOrExportKind;}

export type ExportDefaultDeclarationKind = FunctionDeclaration | ClassDeclaration | TSInterfaceDeclaration | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | Identifier | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | ClassExpression | ConditionalExpression | FunctionExpression | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression

export type ModuleExportName = Identifier | Identifier | StringLiteral

export type TSThisParameter = {type: 'TSThisParameter';span: Span;this_span: Span;type_annotation: (TSTypeAnnotation) | null;}

export type TSEnumDeclaration = {type: 'TSEnumDeclaration';span: Span;id: Identifier;members: Array<TSEnumMember>;const: boolean;declare: boolean;}

export type TSEnumMember = {type: 'TSEnumMember';span: Span;id: TSEnumMemberName;initializer: (Expression) | null;}

export type TSEnumMemberName = StaticIdentifier | StaticStringLiteral | StaticTemplateLiteral | StaticNumericLiteral | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | Identifier | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | ClassExpression | ConditionalExpression | FunctionExpression | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression

export type TSTypeAnnotation = {type: 'TSTypeAnnotation';span: Span;type_annotation: TSType;}

export type TSLiteralType = {type: 'TSLiteralType';span: Span;literal: TSLiteral;}

export type TSLiteral = BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | UnaryExpression

export type TSType = TSAnyKeyword | TSBigIntKeyword | TSBooleanKeyword | TSIntrinsicKeyword | TSNeverKeyword | TSNullKeyword | TSNumberKeyword | TSObjectKeyword | TSStringKeyword | TSSymbolKeyword | TSUndefinedKeyword | TSUnknownKeyword | TSVoidKeyword | TSArrayType | TSConditionalType | TSConstructorType | TSFunctionType | TSImportType | TSIndexedAccessType | TSInferType | TSIntersectionType | TSLiteralType | TSMappedType | TSNamedTupleMember | TSQualifiedName | TSTemplateLiteralType | TSThisType | TSTupleType | TSTypeLiteral | TSTypeOperatorType | TSTypePredicate | TSTypeQuery | TSTypeReference | TSUnionType | TSParenthesizedType | JSDocNullableType | JSDocNonNullableType | JSDocUnknownType

export type TSConditionalType = {type: 'TSConditionalType';span: Span;check_type: TSType;extends_type: TSType;true_type: TSType;false_type: TSType;}

export type TSUnionType = {type: 'TSUnionType';span: Span;types: Array<TSType>;}

export type TSIntersectionType = {type: 'TSIntersectionType';span: Span;types: Array<TSType>;}

export type TSParenthesizedType = {type: 'TSParenthesizedType';span: Span;type_annotation: TSType;}

export type TSTypeOperator = {type: 'TSTypeOperator';span: Span;operator: TSTypeOperatorOperator;type_annotation: TSType;}

export type TSTypeOperatorOperator = 'keyof' | 'unique' | 'readonly'

export type TSArrayType = {type: 'TSArrayType';span: Span;element_type: TSType;}

export type TSIndexedAccessType = {type: 'TSIndexedAccessType';span: Span;object_type: TSType;index_type: TSType;}

export type TSTupleType = {type: 'TSTupleType';span: Span;element_types: Array<TSTupleElement>;}

export type TSNamedTupleMember = {type: 'TSNamedTupleMember';span: Span;element_type: TSTupleElement;label: Identifier;optional: boolean;}

export type TSOptionalType = {type: 'TSOptionalType';span: Span;type_annotation: TSType;}

export type TSRestType = {type: 'TSRestType';span: Span;type_annotation: TSType;}

export type TSTupleElement = TSOptionalType | TSRestType | TSAnyKeyword | TSBigIntKeyword | TSBooleanKeyword | TSIntrinsicKeyword | TSNeverKeyword | TSNullKeyword | TSNumberKeyword | TSObjectKeyword | TSStringKeyword | TSSymbolKeyword | TSUndefinedKeyword | TSUnknownKeyword | TSVoidKeyword | TSArrayType | TSConditionalType | TSConstructorType | TSFunctionType | TSImportType | TSIndexedAccessType | TSInferType | TSIntersectionType | TSLiteralType | TSMappedType | TSNamedTupleMember | TSQualifiedName | TSTemplateLiteralType | TSThisType | TSTupleType | TSTypeLiteral | TSTypeOperatorType | TSTypePredicate | TSTypeQuery | TSTypeReference | TSUnionType | TSParenthesizedType | JSDocNullableType | JSDocNonNullableType | JSDocUnknownType

export type TSAnyKeyword = {type: 'TSAnyKeyword';span: Span;}

export type TSStringKeyword = {type: 'TSStringKeyword';span: Span;}

export type TSBooleanKeyword = {type: 'TSBooleanKeyword';span: Span;}

export type TSNumberKeyword = {type: 'TSNumberKeyword';span: Span;}

export type TSNeverKeyword = {type: 'TSNeverKeyword';span: Span;}

export type TSIntrinsicKeyword = {type: 'TSIntrinsicKeyword';span: Span;}

export type TSUnknownKeyword = {type: 'TSUnknownKeyword';span: Span;}

export type TSNullKeyword = {type: 'TSNullKeyword';span: Span;}

export type TSUndefinedKeyword = {type: 'TSUndefinedKeyword';span: Span;}

export type TSVoidKeyword = {type: 'TSVoidKeyword';span: Span;}

export type TSSymbolKeyword = {type: 'TSSymbolKeyword';span: Span;}

export type TSThisType = {type: 'TSThisType';span: Span;}

export type TSObjectKeyword = {type: 'TSObjectKeyword';span: Span;}

export type TSBigIntKeyword = {type: 'TSBigIntKeyword';span: Span;}

export type TSTypeReference = {type: 'TSTypeReference';span: Span;type_name: TSTypeName;type_parameters: (TSTypeParameterInstantiation) | null;}

export type TSTypeName = Identifier | QualifiedName

export type TSQualifiedName = {type: 'TSQualifiedName';span: Span;left: TSTypeName;right: Identifier;}

export type TSTypeParameterInstantiation = {type: 'TSTypeParameterInstantiation';span: Span;params: Array<TSType>;}

export type TSTypeParameter = {type: 'TSTypeParameter';span: Span;name: Identifier;constraint: (TSType) | null;default: (TSType) | null;in: boolean;out: boolean;const: boolean;}

export type TSTypeParameterDeclaration = {type: 'TSTypeParameterDeclaration';span: Span;params: Array<TSTypeParameter>;}

export type TSTypeAliasDeclaration = {type: 'TSTypeAliasDeclaration';span: Span;id: Identifier;type_parameters: (TSTypeParameterDeclaration) | null;type_annotation: TSType;declare: boolean;}

export type TSAccessibility = 'private' | 'protected' | 'public'

export type TSClassImplements = {type: 'TSClassImplements';span: Span;expression: TSTypeName;type_parameters: (TSTypeParameterInstantiation) | null;}

export type TSInterfaceDeclaration = {type: 'TSInterfaceDeclaration';span: Span;id: Identifier;extends: (Array<TSInterfaceHeritage>) | null;type_parameters: (TSTypeParameterDeclaration) | null;body: TSInterfaceBody;declare: boolean;}

export type TSInterfaceBody = {type: 'TSInterfaceBody';span: Span;body: Array<TSSignature>;}

export type TSPropertySignature = {type: 'TSPropertySignature';span: Span;computed: boolean;optional: boolean;readonly: boolean;key: PropertyKey;type_annotation: (TSTypeAnnotation) | null;}

export type TSSignature = TSIndexSignature | TSPropertySignature | TSCallSignatureDeclaration | TSConstructSignatureDeclaration | TSMethodSignature

export type TSIndexSignature = {type: 'TSIndexSignature';span: Span;parameters: Array<Identifier>;type_annotation: TSTypeAnnotation;readonly: boolean;}

export type TSCallSignatureDeclaration = {type: 'TSCallSignatureDeclaration';span: Span;this_param: (TSThisParameter) | null;params: FormalParameters;return_type: (TSTypeAnnotation) | null;type_parameters: (TSTypeParameterDeclaration) | null;}

export type TSMethodSignatureKind = 'method' | 'get' | 'set'

export type TSMethodSignature = {type: 'TSMethodSignature';span: Span;key: PropertyKey;computed: boolean;optional: boolean;kind: TSMethodSignatureKind;this_param: (TSThisParameter) | null;params: FormalParameters;return_type: (TSTypeAnnotation) | null;type_parameters: (TSTypeParameterDeclaration) | null;}

export type TSConstructSignatureDeclaration = {type: 'TSConstructSignatureDeclaration';span: Span;params: FormalParameters;return_type: (TSTypeAnnotation) | null;type_parameters: (TSTypeParameterDeclaration) | null;}

export type TSIndexSignatureName = {type: 'Identifier';span: Span;name: string;type_annotation: TSTypeAnnotation;}

export type TSInterfaceHeritage = {type: 'TSInterfaceHeritage';span: Span;expression: Expression;type_parameters: (TSTypeParameterInstantiation) | null;}

export type TSTypePredicate = {type: 'TSTypePredicate';span: Span;parameter_name: TSTypePredicateName;asserts: boolean;type_annotation: (TSTypeAnnotation) | null;}

export type TSTypePredicateName = Identifier | This

export type TSModuleDeclaration = {type: 'TSModuleDeclaration';span: Span;id: TSModuleDeclarationName;body: (TSModuleDeclarationBody) | null;kind: TSModuleDeclarationKind;declare: boolean;}

export type TSModuleDeclarationKind = 'global' | 'module' | 'namespace'

export type TSModuleDeclarationName = Identifier | StringLiteral

export type TSModuleDeclarationBody = TSModuleDeclaration | TSModuleBlock

export type TSModuleBlock = {type: 'TSModuleBlock';span: Span;body: Array<Statement>;}

export type TSTypeLiteral = {type: 'TSTypeLiteral';span: Span;members: Array<TSSignature>;}

export type TSInferType = {type: 'TSInferType';span: Span;type_parameter: TSTypeParameter;}

export type TSTypeQuery = {type: 'TSTypeQuery';span: Span;expr_name: TSTypeQueryExprName;type_parameters: (TSTypeParameterInstantiation) | null;}

export type TSTypeQueryExprName = TSImportType | Identifier | QualifiedName

export type TSImportType = {type: 'TSImportType';span: Span;is_type_of: boolean;parameter: TSType;qualifier: (TSTypeName) | null;attributes: (TSImportAttributes) | null;type_parameters: (TSTypeParameterInstantiation) | null;}

export type TSImportAttributes = {type: 'TSImportAttributes';span: Span;attributes_keyword: Identifier;elements: Array<TSImportAttribute>;}

export type TSImportAttribute = {type: 'TSImportAttribute';span: Span;name: TSImportAttributeName;value: Expression;}

export type TSImportAttributeName = Identifier | StringLiteral

export type TSFunctionType = {type: 'TSFunctionType';span: Span;this_param: (TSThisParameter) | null;params: FormalParameters;return_type: TSTypeAnnotation;type_parameters: (TSTypeParameterDeclaration) | null;}

export type TSConstructorType = {type: 'TSConstructorType';span: Span;abstract: boolean;params: FormalParameters;return_type: TSTypeAnnotation;type_parameters: (TSTypeParameterDeclaration) | null;}

export type TSMappedType = {type: 'TSMappedType';span: Span;type_parameter: TSTypeParameter;name_type: (TSType) | null;type_annotation: (TSType) | null;optional: TSMappedTypeModifierOperator;readonly: TSMappedTypeModifierOperator;}

export type TSMappedTypeModifierOperator = 'true' | '+' | '-' | 'none'

export type TSTemplateLiteralType = {type: 'TSTemplateLiteralType';span: Span;quasis: Array<TemplateElement>;types: Array<TSType>;}

export type TSAsExpression = {type: 'TSAsExpression';span: Span;expression: Expression;type_annotation: TSType;}

export type TSSatisfiesExpression = {type: 'TSSatisfiesExpression';span: Span;expression: Expression;type_annotation: TSType;}

export type TSTypeAssertion = {type: 'TSTypeAssertion';span: Span;expression: Expression;type_annotation: TSType;}

export type TSImportEqualsDeclaration = {type: 'TSImportEqualsDeclaration';span: Span;id: Identifier;module_reference: TSModuleReference;import_kind: ImportOrExportKind;}

export type TSModuleReference = ExternalModuleReference | Identifier | QualifiedName

export type TSExternalModuleReference = {type: 'TSExternalModuleReference';span: Span;expression: StringLiteral;}

export type TSNonNullExpression = {type: 'TSNonNullExpression';span: Span;expression: Expression;}

export type Decorator = {type: 'Decorator';span: Span;expression: Expression;}

export type TSExportAssignment = {type: 'TSExportAssignment';span: Span;expression: Expression;}

export type TSNamespaceExportDeclaration = {type: 'TSNamespaceExportDeclaration';span: Span;id: Identifier;}

export type TSInstantiationExpression = {type: 'TSInstantiationExpression';span: Span;expression: Expression;type_parameters: TSTypeParameterInstantiation;}

export type ImportOrExportKind = 'value' | 'type'

export type JSDocNullableType = {type: 'JSDocNullableType';span: Span;type_annotation: TSType;postfix: boolean;}

export type JSDocNonNullableType = {type: 'JSDocNonNullableType';span: Span;type_annotation: TSType;postfix: boolean;}

export type JSDocUnknownType = {type: 'JSDocUnknownType';span: Span;}

export type JSXElement = {type: 'JSXElement';span: Span;opening_element: JSXOpeningElement;closing_element: (JSXClosingElement) | null;children: Array<JSXChild>;}

export type JSXOpeningElement = {type: 'JSXOpeningElement';span: Span;self_closing: boolean;name: JSXElementName;attributes: Array<JSXAttributeItem>;type_parameters: (TSTypeParameterInstantiation) | null;}

export type JSXClosingElement = {type: 'JSXClosingElement';span: Span;name: JSXElementName;}

export type JSXFragment = {type: 'JSXFragment';span: Span;opening_fragment: JSXOpeningFragment;closing_fragment: JSXClosingFragment;children: Array<JSXChild>;}

export type JSXOpeningFragment = {type: 'JSXOpeningFragment';span: Span;}

export type JSXClosingFragment = {type: 'JSXClosingFragment';span: Span;}

export type JSXElementName = Identifier | Identifier | NamespacedName | MemberExpression | ThisExpression

export type JSXNamespacedName = {type: 'JSXNamespacedName';span: Span;namespace: JSXIdentifier;property: JSXIdentifier;}

export type JSXMemberExpression = {type: 'JSXMemberExpression';span: Span;object: JSXMemberExpressionObject;property: JSXIdentifier;}

export type JSXMemberExpressionObject = Identifier | MemberExpression | ThisExpression

export type JSXExpressionContainer = {type: 'JSXExpressionContainer';span: Span;expression: JSXExpression;}

export type JSXExpression = EmptyExpression | BooleanLiteral | NullLiteral | NumericLiteral | BigIntLiteral | RegExpLiteral | StringLiteral | TemplateLiteral | Identifier | MetaProperty | Super | ArrayExpression | ArrowFunctionExpression | AssignmentExpression | AwaitExpression | BinaryExpression | CallExpression | ChainExpression | ClassExpression | ConditionalExpression | FunctionExpression | ImportExpression | LogicalExpression | NewExpression | ObjectExpression | ParenthesizedExpression | SequenceExpression | TaggedTemplateExpression | ThisExpression | UnaryExpression | UpdateExpression | YieldExpression | PrivateInExpression | JSXElement | JSXFragment | TSAsExpression | TSSatisfiesExpression | TSTypeAssertion | TSNonNullExpression | TSInstantiationExpression | ComputedMemberExpression | StaticMemberExpression | PrivateFieldExpression

export type JSXEmptyExpression = {type: 'JSXEmptyExpression';span: Span;}

export type JSXAttributeItem = Attribute | SpreadAttribute

export type JSXAttribute = {type: 'JSXAttribute';span: Span;name: JSXAttributeName;value: (JSXAttributeValue) | null;}

export type JSXSpreadAttribute = {type: 'JSXSpreadAttribute';span: Span;argument: Expression;}

export type JSXAttributeName = Identifier | NamespacedName

export type JSXAttributeValue = StringLiteral | ExpressionContainer | Element | Fragment

export type JSXIdentifier = {type: 'JSXIdentifier';span: Span;name: string;}

export type JSXChild = Text | Element | Fragment | ExpressionContainer | Spread

export type JSXSpreadChild = {type: 'JSXSpreadChild';span: Span;expression: Expression;}

export type JSXText = {type: 'JSXText';span: Span;value: string;}

export type NumberBase = 'Float' | 'Decimal' | 'Binary' | 'Octal' | 'Hex'

export type BigintBase = 'Decimal' | 'Binary' | 'Octal' | 'Hex'

export type AssignmentOperator = '=' | '+=' | '-=' | '*=' | '/=' | '%=' | '<<=' | '>>=' | '>>>=' | '|=' | '^=' | '&=' | '&&=' | '||=' | '??=' | '**='

export type BinaryOperator = '==' | '!=' | '===' | '!==' | '<' | '<=' | '>' | '>=' | '<<' | '>>' | '>>>' | '+' | '-' | '*' | '/' | '%' | '|' | '^' | '&' | 'in' | 'instanceof' | '**'

export type LogicalOperator = '||' | '&&' | '??'

export type UnaryOperator = '-' | '+' | '!' | '~' | 'typeof' | 'void' | 'delete'

export type UpdateOperator = '++' | '--'

export type Span = {type: 'Span';start: number;end: number;}

export type SourceType = {language: Language;module_kind: ModuleKind;variant: LanguageVariant;}

export type Language = 'javascript' | 'typescript' | 'typescriptDefinition'

export type ModuleKind = 'script' | 'module' | 'unambiguous'

export type LanguageVariant = 'standard' | 'jsx'

export type Pattern = {type: 'Pattern';span: Span;body: Disjunction;}

export type Disjunction = {type: 'Disjunction';span: Span;body: Array<Alternative>;}

export type Alternative = {type: 'Alternative';span: Span;body: Array<Term>;}

export type Term = BoundaryAssertion | LookAroundAssertion | Quantifier | Character | Dot | CharacterClassEscape | UnicodePropertyEscape | CharacterClass | CapturingGroup | IgnoreGroup | IndexedReference | NamedReference

export type BoundaryAssertion = {type: 'BoundaryAssertion';span: Span;kind: BoundaryAssertionKind;}

export type BoundaryAssertionKind = 'start' | 'end' | 'boundary' | 'negativeBoundary'

export type LookAroundAssertion = {type: 'LookAroundAssertion';span: Span;kind: LookAroundAssertionKind;body: Disjunction;}

export type LookAroundAssertionKind = 'lookahead' | 'negativeLookahead' | 'lookbehind' | 'negativeLookbehind'

export type Quantifier = {type: 'Quantifier';span: Span;min: number;max: (number) | null;greedy: boolean;body: Term;}

export type Character = {type: 'Character';span: Span;kind: CharacterKind;value: number;}

export type CharacterKind = 'controlLetter' | 'hexadecimalEscape' | 'identifier' | 'null' | 'octal1' | 'octal2' | 'octal3' | 'singleEscape' | 'symbol' | 'unicodeEscape'

export type CharacterClassEscape = {type: 'CharacterClassEscape';span: Span;kind: CharacterClassEscapeKind;}

export type CharacterClassEscapeKind = 'd' | 'negativeD' | 's' | 'negativeS' | 'w' | 'negativeW'

export type UnicodePropertyEscape = {type: 'UnicodePropertyEscape';span: Span;negative: boolean;strings: boolean;name: string;value: (string) | null;}

export type Dot = {type: 'Dot';span: Span;}

export type CharacterClass = {type: 'CharacterClass';span: Span;negative: boolean;strings: boolean;kind: CharacterClassContentsKind;body: Array<CharacterClassContents>;}

export type CharacterClassContentsKind = 'union' | 'intersection' | 'subtraction'

export type CharacterClassContents = CharacterClassRange | CharacterClassEscape | UnicodePropertyEscape | Character | NestedCharacterClass | ClassStringDisjunction

export type CharacterClassRange = {type: 'CharacterClassRange';span: Span;min: Character;max: Character;}

export type ClassStringDisjunction = {type: 'ClassStringDisjunction';span: Span;strings: boolean;body: Array<ClassString>;}

export type ClassString = {type: 'ClassString';span: Span;strings: boolean;body: Array<Character>;}

export type CapturingGroup = {type: 'CapturingGroup';span: Span;name: (string) | null;body: Disjunction;}

export type IgnoreGroup = {type: 'IgnoreGroup';span: Span;modifiers: (Modifiers) | null;body: Disjunction;}

export type Modifiers = {type: 'Modifiers';span: Span;enabling: (Modifier) | null;disabling: (Modifier) | null;}

export type Modifier = {type: 'Modifier';ignore_case: boolean;multiline: boolean;sticky: boolean;}

export type IndexedReference = {type: 'IndexedReference';span: Span;index: number;}

export type NamedReference = {type: 'NamedReference';span: Span;name: string;}

