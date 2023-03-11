use oxc_ast::{Atom, Span};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};

#[derive(Debug, Error, Diagnostic)]
#[error("Flow is not supported")]
#[diagnostic()]
pub struct Flow(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Unexpected token")]
#[diagnostic()]
pub struct UnexpectedToken(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Expect token")]
#[diagnostic()]
pub struct ExpectToken(
    pub &'static str,
    pub &'static str,
    #[label("Expect `{0}` here, but found `{1}`")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Invalid escape sequence")]
pub struct InvalidEscapeSequence(#[label("Invalid escape sequence")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Invalid escape sequence")]
pub struct NonOctalDecimalEscapeSequence(
    #[label("\\8 and \\9 are not allowed in strict mode")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Invalid Unicode escape sequence")]
pub struct UnicodeEscapeSequence(#[label("Invalid Unicode escape sequence")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Invalid Character `{0}`")]
pub struct InvalidCharacter(pub char, #[label("Invalid Character `{0}`")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Invalid characters after number")]
pub struct InvalidNumberEnd(#[label("Invalid characters after number")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Unterminated multiline comment")]
pub struct UnterminatedMultiLineComment(#[label("Unterminated multiline comment")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Unterminated string")]
pub struct UnterminatedString(#[label("Unterminated string")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Unexpected flag {0} in regular expression literal")]
pub struct RegExpFlag(
    pub char,
    #[label("Unexpected flag {0} in regular expression literal")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Flag {0} is mentioned twice in regular expression literal")]
pub struct RegExpFlagTwice(
    pub char,
    #[label("Flag {0} is mentioned twice in regular expression literal")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("The 'u' and 'v' regular expression flags cannot be enabled at the same time")]
pub struct RegExpFlagUAndV(
    #[label("The 'u' and 'v' regular expression flags cannot be enabled at the same time")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Unexpected end of file")]
pub struct UnexpectedEnd(#[label("Unexpected end of file")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Unterminated regular expression")]
pub struct UnterminatedRegExp(#[label("Unterminated regular expression")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Invalid Number")]
pub struct InvalidNumber(pub &'static str, #[label("{0}")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Keywords cannot contain escape characters")]
#[diagnostic()]
pub struct EscapedKeyword(#[label("keyword cannot contain escape characters")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Automatic Semicolon Insertion")]
#[diagnostic(help("Try insert a semicolon here"))]
pub struct AutoSemicolonInsertion(
    #[label("Expected a semicolon or an implicit semicolon after a statement, but found none")]
    pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Octal literals are not allowed in strict mode")]
#[diagnostic(help("for octal literals use the '0o' prefix instead"))]
pub struct LegacyOctal(
    #[label("'0'-prefixed octal literals and octal escape sequences are deprecated")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Decimals with leading zeros are not allowed in strict mode")]
#[diagnostic(help("remove the leading zero"))]
pub struct LeadingZeroDecimal(
    #[label("Decimals with leading zeros are not allowed in strict mode")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Line terminator not permitted before arrow")]
#[diagnostic()]
pub struct LineterminatorBeforeArrow(
    #[label("Line terminator not permitted before arrow")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Unexpected new.target expression")]
#[diagnostic(help(
    "new.target is only allowed in constructors and functions invoked using thew `new` operator"
))]
pub struct NewTarget(#[label("new.target expression is not allowed here")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("The only valid meta property for new is new.target")]
#[diagnostic()]
pub struct NewTargetProperty(
    #[label("The only valid meta property for new is new.target")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Unexpected import.meta expression")]
#[diagnostic(help("import.meta is only allowed in module code"))]
pub struct ImportMeta(#[label("import.meta expression is not allowed here")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("The only valid meta property for import is import.meta")]
#[diagnostic()]
pub struct ImportMetaProperty(
    #[label("The only valid meta property for import is import.meta")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Illegal break statement")]
#[diagnostic(help(
    "A `break` statement can only be used within an enclosing iteration or switch statement."
))]
pub struct InvalidBreak(#[label("break statement is not allowed here")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Illegal continue statement: no surrounding iteration statement")]
#[diagnostic(help(
    "A `continue` statement can only be used within an enclosing `for`, `while` or `do while` "
))]
pub struct InvalidContinue(#[label("continue statement is not allowed here")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error(
    "A `{0}` statement can only jump to a label of an enclosing `for`, `while` or `do while` statement."
)]
#[diagnostic()]
pub struct InvalidLabelNonIteration(
    &'static str,
    #[label("This is an non-iteration statement")] pub Span,
    #[label("for this label")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Use of undefined label")]
#[diagnostic()]
pub struct InvalidLabelTarget(#[label("This label is used, but not defined")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Jump target cannot cross function boundary.")]
#[diagnostic()]
pub struct InvalidLabelJumpTarget(#[label("Jump target cannot cross function boundary.")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Unexpected '{0}' strict mode")]
#[diagnostic()]
pub struct UnexpectedIdentifierAssign(
    Atom,
    #[label("Cannot assign to '{0}' in strict mode")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Invalid left-hand side in assignment")]
#[diagnostic()]
pub struct UnexpectedLhsAssign(#[label("Invalid left-hand side in assignment")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("The keyword '{0}' is reserved")]
#[diagnostic()]
pub struct ReservedKeyword(Atom, #[label("{0} is reserved")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("{0} is disallowed as a lexically bound name")]
#[diagnostic()]
pub struct DisallowedLexicalName(
    pub Atom,
    #[label("{0} is disallowed as a lexically bound name")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("`let` cannot be declared as a variable name inside of a `{0}` declaration")]
#[diagnostic()]
pub struct InvalidLetDeclaration(String, #[label("Rename the let identifier here")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Missing initializer in destructuring declaration")]
#[diagnostic()]
pub struct InvalidDestrucuringDeclaration(
    #[label("Missing initializer in destructuring declaration")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Missing initializer in const declaration")]
#[diagnostic()]
pub struct MissinginitializerInConst(#[label("const declaration need an initializer")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Functions cannot be labelled")]
#[diagnostic(help("This is not allowed in strict mode starting with ECMAScript 2015."))]
pub struct FunctionsCannotBeLabelled(#[label("Functions cannot be labelled")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Cannot use {0} outside a method")]
pub struct MethodCode(&'static str, #[label("Cannot use {0} outside a method")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Cannot use {0} outside a module")]
#[diagnostic()]
pub struct ModuleCode(&'static str, #[label("Cannot use {0} outside a module")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Lexical declaration cannot appear in a single-statement context")]
#[diagnostic(help("Wrap this declaration in a block statement"))]
pub struct LexicalDeclarationSingleStatement(
    #[label("Lexical declaration is not allowed here")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Invalid function declaration")]
#[diagnostic(help(
    "In strict mode code, functions can only be declared at top level or inside a block"
))]
pub struct FunctionDeclarationStrict(#[label("function declaration is not allowed here")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Async functions can only be declared at the top level or inside a block")]
#[diagnostic()]
pub struct AsyncFunctionDeclaration(
    #[label("Async functions can only be declared at the top level or inside a block")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Generators can only be declared at the top level or inside a block")]
#[diagnostic()]
pub struct GeneratorFunctionDeclaration(
    #[label("Generators can only be declared at the top level or inside a block")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Invalid function declaration")]
#[diagnostic(help(
    "In non-strict mode code, functions can only be declared at top level, inside a block, or as the body of an if statement"
))]
pub struct FunctionDeclarationNonStrict(
    #[label("function declaration is not allowed here")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("`await` is only allowed within async functions and at the top levels of modules")]
#[diagnostic()]
pub struct AwaitExpression(
    #[label("`await` is only allowed within async functions and at the top levels of modules")]
    pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("A 'yield' expression is only allowed in a generator body.")]
#[diagnostic()]
pub struct YieldExpression(
    #[label("A 'yield' expression is only allowed in a generator body.")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Invalid class declaration")]
#[diagnostic()]
pub struct ClassDeclaration(
    #[label("Classes can only be declared at top level or inside a block")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Rest element must be last element")]
#[diagnostic()]
pub struct RestElement(#[label("Rest element must be last element")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Spread must be last element")]
#[diagnostic()]
pub struct SpreadLastElement(#[label("Spread must be last element")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Unexpected trailing comma after rest element")]
#[diagnostic()]
pub struct RestElementTrailingComma(
    #[label("Unexpected trailing comma after rest element")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Invalid rest argument")]
#[diagnostic(help("Expected identifier in rest argument"))]
pub struct InvalidRestArgument(#[label("Invalid rest argument")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Invalid parenthesized parameter")]
#[diagnostic(help("remove the parentheses"))]
pub struct InvalidParenthesizedParameter(#[label("Invliad parenthesized parameter")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Invalid parenthesized pattern")]
#[diagnostic()]
pub struct InvalidParenthesizedPattern(#[label("Invliad parenthesized pattern")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Invalid assignment")]
#[diagnostic()]
pub struct InvalidAssignment(#[label("Cannot assign to this expression")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Optional chaining cannot appear in the callee of new expressions")]
#[diagnostic()]
pub struct NewOptionalChain(
    #[label("Optional chaining cannot appear in the callee of new expressions")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("The left-hand side of a `for...of` statement may not be `async`")]
#[diagnostic()]
pub struct ForLoopAsyncOf(
    #[label("The left-hand side of a `for...of` statement may not be `async`")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("await can only be used in conjunction with `for...of` statements")]
#[diagnostic()]
pub struct ForAwait(
    #[label("await can only be used in conjunction with `for...of` statements")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Cannot use new with dynamic import")]
#[diagnostic()]
pub struct NewDynamicImport(#[label("Cannot use new with dynamic import")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("'{0}' declaration can only be used at the top level of a module")]
#[diagnostic()]
pub struct TopLevel(
    &'static str,
    #[label("'{0}' declaration can only appear at the top level")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Duplicated export '{0}'")]
#[diagnostic()]
pub struct DuplicateExport(
    Atom,
    #[label("Export has already been declared here")] pub Span,
    #[label("It cannot be redeclared here")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Unexpected private field")]
#[diagnostic(help(
    "Private names are only allowed in property accesses (`obj.#field`) or in `in` expressions (`#field in obj`)."
))]
pub struct UnexpectedPrivateIdentifier(#[label("Unexpected private field")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Classes can't have an element named '#constructor'")]
#[diagnostic()]
pub struct PrivateNameConstructor(
    #[label("Classes can't have an element named '#constructor'")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Private field '{0}' must be declared in an enclosing class")]
#[diagnostic()]
pub struct PrivateFieldUndeclared(
    Atom,
    #[label("Private field '{0}' must be declared in an enclosing class")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Unexpected private identifier")]
#[diagnostic()]
pub struct PrivateNotInClass(
    Atom,
    #[label("Private identifier '#{0}' is not allowed outside class bodies")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Classes may not have a static property named prototype")]
#[diagnostic()]
pub struct StaticPrototype(
    #[label("Classes may not have a static property named prototype")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Constructor can't have get/set modifier")]
#[diagnostic()]
pub struct ConstructorGetterSetter(#[label("Constructor can't have get/set modifier")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Constructor can't be an async method")]
#[diagnostic()]
pub struct ConstructorAsync(#[label("Constructor can't be an async method")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Cannot use `{0}` as an identifier in an async context")]
#[diagnostic()]
pub struct IdentifierAsync(&'static str, #[label("{0} cannot be used here")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Cannot use `{0}` as an identifier in a generator context")]
#[diagnostic()]
pub struct IdentifierGenerator(&'static str, #[label("{0} cannot be used here")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Constructor can't be a generator")]
#[diagnostic()]
pub struct ConstructorGenerator(#[label("Constructor can't be a generator")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Classes can't have a field named 'constructor'")]
#[diagnostic()]
pub struct FieldConstructor(#[label("Classes can't have a field named 'constructor'")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Multiple constructor implementations are not allowed.")]
#[diagnostic()]
pub struct DuplicateConstructor(
    #[label("constructor has already been declared here")] pub Span,
    #[label("it cannot be redeclared here")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("An export name cannot include a unicode lone surrogate")]
#[diagnostic()]
pub struct ExportLoneSurrogate(
    #[label("An export name cannot include a unicode lone surrogate")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("A string literal cannot be used as an exported binding without `from`")]
#[diagnostic(help("Did you mean `export {{ '{0}' as '{1}' }} from 'some-module'`?"))]
pub struct ExportNamedString(
    pub Atom,
    pub Atom,
    #[label("A string literal cannot be used as an exported binding without `from`")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Bad escape sequence in untagged template literal")]
#[diagnostic()]
pub struct TemplateLiteral(#[label("Bad escape sequence in untagged template literal")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Delete of an unqualified identifier in strict mode.")]
#[diagnostic()]
pub struct DeleteOfUnqualified(
    #[label("Delete of an unqualified identifier in strict mode")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("'with' statements are not allowed")]
#[diagnostic()]
pub struct WithStatement(#[label("'with' statements are not allowed")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Private fields can not be deleted")]
#[diagnostic()]
pub struct DeletePrivateField(#[label("Private fields can not be deleted")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Empty parenthesized expression")]
#[diagnostic()]
pub struct EmptyParenthesizedExpression(#[label("Expected an expression here")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Undefined export")]
#[diagnostic()]
pub struct UndefinedExport(Atom, #[label("Export '{0}' is not defined")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Logical expressions and coalesce expressions cannot be mixed")]
#[diagnostic(help("Wrap either expression by parentheses"))]
pub struct MixedCoalesce(
    #[label("Logical expressions and coalesce expressions cannot be mixed")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("'Unexpected `{0}`")]
#[diagnostic()]
pub struct UnexpectedKeyword(
    pub &'static str,
    #[label("'{0}' keyword is unexpected here")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("{0} loop variable declaration may not have an initializer")]
#[diagnostic()]
pub struct UnexpectedInitializerInForLoopHead(
    &'static str,
    #[label("{0} loop variable declaration may not have an initializer")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Only a single declaration is allowed in a `for...{0}` statement")]
#[diagnostic()]
pub struct MultipleDeclarationInForLoopHead(
    &'static str,
    #[label("Only a single declaration is allowed in a `for...{0}` statement")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Illegal newline after {0}")]
#[diagnostic()]
pub struct IllegalNewline(
    pub &'static str,
    #[label("{0} starts here")] pub Span,
    #[label("A newline is not expected here")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Duplicate parameter name not allowed in this context")]
#[diagnostic()]
pub struct DuplicateParameter(
    #[label("Duplicate parameter name not allowed in this context")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Illegal 'use strict' directive in function with non-simple parameter list")]
#[diagnostic()]
pub struct IllegalUseStrict(
    #[label("Illegal 'use strict' directive in function with non-simple parameter list")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("'arguments' is not allowed in {0}")]
#[diagnostic()]
pub struct UnexpectedArguments(
    &'static str,
    #[label("'arguments' is not allowed in {0}")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Unexpected {0} expression")]
#[diagnostic()]
pub struct UnexpectedExpression(&'static str, #[label("Unexpected {0} expression")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Unexpected exponentiation expression")]
#[diagnostic(help("Wrap {0} expression in parentheses to enforce operator precedence"))]
pub struct UnexpectedExponential(
    &'static str,
    #[label("Unexpected exponentiation expression")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Tagged template expressions are not permitted in an optional chain")]
#[diagnostic()]
pub struct OptionalChainTaggedTemplate(
    #[label("Tagged template expressions are not permitted in an optional chain")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("A 'get' accessor must not have any formal parameters.")]
#[diagnostic()]
pub struct GetterParameters(
    #[label("A 'get' accessor must not have any formal parameters.")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("A 'set' accessor must have exactly one parameter.")]
#[diagnostic()]
pub struct SetterParameters(#[label("A 'set' accessor must have exactly one parameter.")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("A 'set' accessor function argument must not be a rest parameter")]
#[diagnostic()]
pub struct SetterParametersRestPattern(
    #[label("A 'set' accessor function argument must not be a rest parameter")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("{0} expression not allowed in formal parameter")]
#[diagnostic()]
pub struct AwaitOrYieldInParameter(
    &'static str,
    #[label("{0} expression not allowed in formal parameter")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Invalid assignment in object literal")]
#[diagnostic(help(
    "Did you mean to use a ':'? An '=' can only follow a property name when the containing object literal is part of a destructuring pattern."
))]
pub struct CoverInitializedNameError(#[label("Assignment is not allowed here")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error(
    "Super calls are not permitted outside constructors or in nested functions inside constructors.
"
)]
#[diagnostic()]
pub struct UnexpectedSuperCall(
    #[label(
        "Super calls are not permitted outside constructors or in nested functions inside constructors."
    )]
    pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error(
    "'super' can only be referenced in members of derived classes or object literal expressions.
"
)]
#[diagnostic()]
pub struct UnexpectedSuperReference(
    #[label("'super' can only be referenced in members of derived classes or object literal expressions.
")]
    pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("'super' can only be used with function calls or in property accesses")]
#[diagnostic(help("replace with `super()` or `super.prop` or `super[prop]`"))]
pub struct UnexpectedSuper(
    #[label("'super' can only be used with function calls or in property accesses ")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("'super' can only be referenced in a derived class.")]
#[diagnostic(help("either remove this super, or extend the class"))]
pub struct SuperWithoutDerivedClass(
    #[label("'super' can only be referenced in a derived class.")] pub Span,
    #[label("class does not have `extends`")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Private fields cannot be accessed on super")]
#[diagnostic()]
pub struct SuperPrivate(#[label("Private fields cannot be accessed on super")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Expected function name")]
#[diagnostic(help("Function name is required in function declaration or named export"))]
pub struct ExpectFunctionName(#[label("Function name is required here")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Missing catch or finally clause")]
#[diagnostic()]
pub struct ExpectCatchFinally(#[label("Expected `catch` or `finally` here")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Cannot assign to '{0}' because it is a {1}")]
#[diagnostic()]
pub struct CannotAssignTo(
    Atom,
    &'static str,
    #[label("Cannot assign to '{0}' because this is a {1}")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("A rest parameter cannot have an initializer")]
#[diagnostic()]
pub struct ARestParameterCannotHaveAnInitializer(
    #[label("A rest parameter cannot have an initializer")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("TS1015: Parameter cannot have question mark and initializer")]
#[diagnostic()]
pub struct ParameterCannotHaveQuestionMarkAndInitializer(
    #[label("Parameter cannot have question mark and initializer")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("TS1047: A rest parameter cannot be optional")]
#[diagnostic()]
pub struct ARestParameterCannotBeOptional(#[label("A rest parameter cannot be optional")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("TS1095: A 'set' accessor cannot have a return type annotation")]
#[diagnostic()]
pub struct ASetAccessorCannotHaveAReturnTypeAnnotation(
    #[label("A 'set' accessor cannot have a return type annotation")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("TS1098: Type parameter list cannot be empty")]
#[diagnostic()]
pub struct TypeParameterListCannotBeEmpty(#[label("Type parameter list cannot be empty")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("TS1099: Type argument list cannot be empty")]
#[diagnostic()]
pub struct TypeArgumentListCannotBeEmpty(#[label("Type argument list cannot be empty")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("TS1108: A 'return' statement can only be used within a function body")]
#[diagnostic()]
pub struct ReturnStatementOnlyInFunctionBody(
    #[label("A 'return' statement can only be used within a function body.")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("TS1164: Computed property names are not allowed in enums")]
#[diagnostic()]
pub struct ComputedPropertyNamesAreNotAllowedInEnums(
    #[label("Computed property names are not allowed in enums")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("TS1313: The body of an 'if' statement cannot be the empty statement")]
#[diagnostic()]
pub struct TheBodyOfAnIfStatementCannotBeTheEmptyStatement(
    #[label("The body of an 'if' statement cannot be the empty statement")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("TS1317: A parameter property cannot be declared using a rest parameter")]
#[diagnostic()]
pub struct AParameterPropertyCannotBeDeclaredUsingARestParameter(
    #[label("A parameter property cannot be declared using a rest parameter")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("TS2452: An enum member cannot have a numeric name")]
#[diagnostic()]
pub struct AnEnumMemberCannotHaveANumericName(
    #[label("An enum member cannot have a numeric name")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("TS18007: JSX expressions may not use the comma operator. Did you mean to write an array?")]
#[diagnostic()]
pub struct JSXExpressionsMayNotUseTheCommaOperator(
    #[label("JSX expressions may not use the comma operator. Did you mean to write an array?")]
    pub Span,
);
