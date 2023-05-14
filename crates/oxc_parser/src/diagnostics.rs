use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_span::{Atom, Span};

#[derive(Debug, Error, Diagnostic)]
#[error("Flow is not supported")]
#[diagnostic()]
pub struct Flow(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Unexpected token")]
#[diagnostic()]
pub struct UnexpectedToken(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Expected `{0}` but found `{1}`")]
#[diagnostic()]
pub struct ExpectToken(pub &'static str, pub &'static str, #[label("`{0}` expected")] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Invalid escape sequence")]
pub struct InvalidEscapeSequence(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Invalid Unicode escape sequence")]
pub struct UnicodeEscapeSequence(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Invalid Character `{0}`")]
pub struct InvalidCharacter(pub char, #[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Invalid characters after number")]
pub struct InvalidNumberEnd(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Unterminated multiline comment")]
pub struct UnterminatedMultiLineComment(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Unterminated string")]
pub struct UnterminatedString(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Unexpected flag {0} in regular expression literal")]
pub struct RegExpFlag(pub char, #[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Flag {0} is mentioned twice in regular expression literal")]
pub struct RegExpFlagTwice(pub char, #[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Unexpected end of file")]
pub struct UnexpectedEnd(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Unterminated regular expression")]
pub struct UnterminatedRegExp(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Invalid Number {0}")]
pub struct InvalidNumber(pub &'static str, #[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Keywords cannot contain escape characters")]
#[diagnostic()]
pub struct EscapedKeyword(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Expected a semicolon or an implicit semicolon after a statement, but found none")]
#[diagnostic(help("Try insert a semicolon here"))]
pub struct AutoSemicolonInsertion(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Line terminator not permitted before arrow")]
#[diagnostic()]
pub struct LineterminatorBeforeArrow(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Missing initializer in destructuring declaration")]
#[diagnostic()]
pub struct InvalidDestrucuringDeclaration(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Missing initializer in const declaration")]
#[diagnostic()]
pub struct MissinginitializerInConst(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Lexical declaration cannot appear in a single-statement context")]
#[diagnostic(help("Wrap this declaration in a block statement"))]
pub struct LexicalDeclarationSingleStatement(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Async functions can only be declared at the top level or inside a block")]
#[diagnostic()]
pub struct AsyncFunctionDeclaration(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Generators can only be declared at the top level or inside a block")]
#[diagnostic()]
pub struct GeneratorFunctionDeclaration(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("`await` is only allowed within async functions and at the top levels of modules")]
#[diagnostic()]
pub struct AwaitExpression(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("A 'yield' expression is only allowed in a generator body.")]
#[diagnostic()]
pub struct YieldExpression(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Invalid class declaration")]
#[diagnostic(help("Classes can only be declared at top level or inside a block"))]
pub struct ClassDeclaration(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Rest element must be last element")]
#[diagnostic()]
pub struct RestElementLast(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Spread must be last element")]
#[diagnostic()]
pub struct SpreadLastElement(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Unexpected trailing comma after rest element")]
#[diagnostic()]
pub struct RestElementTrailingComma(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Invalid rest element")]
#[diagnostic(help("Expected identifier in rest element"))]
pub struct InvalidRestElement(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Cannot assign to this expression")]
#[diagnostic()]
pub struct InvalidAssignment(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Optional chaining cannot appear in the callee of new expressions")]
#[diagnostic()]
pub struct NewOptionalChain(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("The left-hand side of a `for...of` statement may not be `async`")]
#[diagnostic()]
pub struct ForLoopAsyncOf(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("await can only be used in conjunction with `for...of` statements")]
#[diagnostic()]
pub struct ForAwait(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Cannot use new with dynamic import")]
#[diagnostic()]
pub struct NewDynamicImport(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Classes can't have an element named '#constructor'")]
#[diagnostic()]
pub struct PrivateNameConstructor(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Classes may not have a static property named prototype")]
#[diagnostic()]
pub struct StaticPrototype(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Constructor can't have get/set modifier")]
#[diagnostic()]
pub struct ConstructorGetterSetter(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Constructor can't be an async method")]
#[diagnostic()]
pub struct ConstructorAsync(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Cannot use `{0}` as an identifier in an async context")]
#[diagnostic()]
pub struct IdentifierAsync(pub &'static str, #[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Cannot use `{0}` as an identifier in a generator context")]
#[diagnostic()]
pub struct IdentifierGenerator(pub &'static str, #[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Constructor can't be a generator")]
#[diagnostic()]
pub struct ConstructorGenerator(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Classes can't have a field named 'constructor'")]
#[diagnostic()]
pub struct FieldConstructor(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("An export name cannot include a unicode lone surrogate")]
#[diagnostic()]
pub struct ExportLoneSurrogate(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("A string literal cannot be used as an exported binding without `from`")]
#[diagnostic(help("Did you mean `export {{ '{0}' as '{1}' }} from 'some-module'`?"))]
pub struct ExportNamedString(pub Atom, pub Atom, #[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Bad escape sequence in untagged template literal")]
#[diagnostic()]
pub struct TemplateLiteral(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Empty parenthesized expression")]
#[diagnostic()]
pub struct EmptyParenthesizedExpression(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Illegal newline after {0}")]
#[diagnostic()]
pub struct IllegalNewline(
    pub &'static str,
    #[label("{0} starts here")] pub Span,
    #[label("A newline is not expected here")] pub Span,
);

#[derive(Debug, Error, Diagnostic)]
#[error("Tagged template expressions are not permitted in an optional chain")]
#[diagnostic()]
pub struct OptionalChainTaggedTemplate(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("A 'get' accessor must not have any formal parameters.")]
#[diagnostic()]
pub struct GetterParameters(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("A 'set' accessor must have exactly one parameter.")]
#[diagnostic()]
pub struct SetterParameters(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("A 'set' accessor function argument must not be a rest parameter")]
#[diagnostic()]
pub struct SetterParametersRestPattern(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("'super' can only be used with function calls or in property accesses")]
#[diagnostic(help("replace with `super()` or `super.prop` or `super[prop]`"))]
pub struct UnexpectedSuper(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Expected function name")]
#[diagnostic(help("Function name is required in function declaration or named export"))]
pub struct ExpectFunctionName(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Missing catch or finally clause")]
#[diagnostic()]
pub struct ExpectCatchFinally(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("TS1095: A 'set' accessor cannot have a return type annotation")]
#[diagnostic()]
pub struct ASetAccessorCannotHaveAReturnTypeAnnotation(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("TS1108: A 'return' statement can only be used within a function body")]
#[diagnostic()]
pub struct ReturnStatementOnlyInFunctionBody(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("TS18007: JSX expressions may not use the comma operator.")]
#[diagnostic(help("Did you mean to write an array?"))]
pub struct JSXExpressionsMayNotUseTheCommaOperator(#[label] pub Span);
