//! All Parser / Linter Diagnostics

use std::{cell::RefCell, ops::Deref, rc::Rc};

use oxc_ast::{Atom, Span};
use thiserror::Error;

pub type PError = miette::Error;

pub type Result<T> = std::result::Result<T, PError>;

#[derive(Debug, Default, Clone)]
pub struct Diagnostics(Rc<RefCell<Vec<PError>>>);

impl Deref for Diagnostics {
    type Target = Rc<RefCell<Vec<PError>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Diagnostics {
    /// # Panics
    #[must_use]
    pub fn into_inner(self) -> Vec<PError> {
        Rc::try_unwrap(self.0).unwrap().into_inner()
    }
}

#[derive(Debug, Clone, Error, miette::Diagnostic)]
pub enum Diagnostic {
    #[error("This file panicked")]
    #[diagnostic()]
    Panic(#[label("")] Span),

    #[error("Flow is not supported")]
    #[diagnostic()]
    Flow(#[label("")] Span),

    /* Lexer */
    #[error("Syntax Error")]
    #[diagnostic()]
    UnexpectedToken(#[label("Unexpected Token")] Span),

    #[error("Syntax Error")]
    #[diagnostic()]
    ExpectToken(&'static str, &'static str, #[label("Expect `{0}` here, but found `{1}`")] Span),

    #[error("Invalid escape sequence")]
    InvalidEscapeSequence(#[label("Invalid escape sequence")] Span),

    #[error("Invalid escape sequence")]
    NonOctalDecimalEscapeSequence(#[label("\\8 and \\9 are not allowed in strict mode")] Span),

    #[error("Invalid Unicode escape sequence")]
    UnicodeEscapeSequence(#[label("Invalid Unicode escape sequence")] Span),

    #[error("Invalid Character `{0}`")]
    InvalidCharacter(char, #[label("Invalid Character `{0}`")] Span),

    #[error("Invalid characters after number")]
    InvalidNumberEnd(#[label("Invalid characters after number")] Span),

    #[error("Unterminated multiLine comment")]
    UnterminatedMultiLineComment(#[label("Unterminated multiLine comment")] Span),

    #[error("Unterminated string")]
    UnterminatedString(#[label("Unterminated string")] Span),

    #[error("Unexpected flag {0} in regular expression literal")]
    RegExpFlag(char, #[label("Unexpected flag {0} in regular expression literal")] Span),

    #[error("Flag {0} is mentioned twice in regular expression literal")]
    RegExpFlagTwice(
        char,
        #[label("Flag {0} is mentioned twice in regular expression literal")] Span,
    ),

    #[error("The 'u' and 'v' regular expression flags cannot be enabled at the same time")]
    RegExpFlagUAndV(
        #[label("The 'u' and 'v' regular expression flags cannot be enabled at the same time")]
        Span,
    ),

    #[error("Unexpected end of file")]
    UnexpectedEnd(#[label("Unexpected end of file")] Span),

    #[error("Unterminated regular expression")]
    UnterminatedRegExp(#[label("Unterminated regular expression")] Span),

    #[error("Invalid Number")]
    InvalidNumber(&'static str, #[label("{0}")] Span),

    #[error("Keywords cannot contain escape characters")]
    #[diagnostic()]
    EscapedKeyword(#[label("keyword cannot contain escape characters")] Span),
    /* Syntax Errors */
    #[error("Automatic Semicolon Insertion")]
    #[diagnostic(help("Try insert a semicolon here"))]
    AutoSemicolonInsertion(
        #[label("Expected a semicolon or an implicit semicolon after a statement, but found none")]
        Span,
    ),

    #[error("Octal literals are not allowed in strict mode")]
    #[diagnostic(help("for octal literals use the '0o' prefix instead"))]
    LegacyOctal(
        #[label("'0'-prefixed octal literals and octal escape sequences are deprecated")] Span,
    ),

    #[error("Decimals with leading zeros are not allowed in strict mode")]
    #[diagnostic(help("remove the leading zero"))]
    LeadingZeroDecimal(#[label("Decimals with leading zeros are not allowed in strict mode")] Span),

    #[error("Line terminator not permitted before arrow")]
    #[diagnostic()]
    LineterminatorBeforeArrow(#[label("Line terminator not permitted before arrow")] Span),

    #[error("Unexpected new.target expression")]
    #[diagnostic(help(
        "new.target is only allowed in constructors and functions invoked using thew `new` operator"
    ))]
    NewTarget(#[label("new.target expression is not allowed here")] Span),

    #[error("The only valid meta property for new is new.target")]
    #[diagnostic()]
    NewTargetProperty(#[label("The only valid meta property for new is new.target")] Span),

    #[error("Unexpected import.meta expression")]
    #[diagnostic(help("import.meta is only allowed in module code"))]
    ImportMeta(#[label("import.meta expression is not allowed here")] Span),

    #[error("The only valid meta property for import is import.meta")]
    #[diagnostic()]
    ImportMetaProperty(#[label("The only valid meta property for import is import.meta")] Span),

    #[error("Illegal break statement")]
    #[diagnostic(help(
        "A `break` statement can only be used within an enclosing iteration or switch statement."
    ))]
    InvalidBreak(#[label("break statement is not allowed here")] Span),

    #[error("Illegal continue statement: no surrounding iteration statement")]
    #[diagnostic(help(
        "A `continue` statement can only be used within an enclosing `for`, `while` or `do while` "
    ))]
    InvalidContinue(#[label("continue statement is not allowed here")] Span),

    #[error(
        "A `{0}` statement can only jump to a label of an enclosing `for`, `while` or `do while` statement."
    )]
    #[diagnostic()]
    InvalidLabelNonIteration(
        &'static str,
        #[label("This is an non-iteration statement")] Span,
        #[label("for this label")] Span,
    ),

    #[error("Use of undefined label")]
    #[diagnostic()]
    InvalidLabelTarget(#[label("This label is used, but not defined")] Span),

    #[error("Jump target cannot cross function boundary.")]
    #[diagnostic()]
    InvalidLabelJumpTarget(#[label("Jump target cannot cross function boundary.")] Span),

    #[error("Unexpected '{0}' strict mode")]
    #[diagnostic()]
    UnexpectedIdentifierAssign(Atom, #[label("Cannot assign to '{0}' in strict mode")] Span),

    #[error("Invalid left-hand side in assignment")]
    #[diagnostic()]
    UnexpectedLhsAssign(#[label("Invalid left-hand side in assignment")] Span),

    #[error("The keyword '{0}' is reserved")]
    #[diagnostic()]
    ReservedKeyword(Atom, #[label("{0} is reserved")] Span),

    #[error("Identifier `{0}` has already been declared")]
    #[diagnostic()]
    Redeclaration(
        Atom,
        #[label("`{0}` has already been declared here")] Span,
        #[label("It can not be redeclared here")] Span,
    ),

    #[error("{0} is disallowed as a lexically bound name")]
    #[diagnostic()]
    DisallowedLexicalName(Atom, #[label("{0} is disallowed as a lexically bound name")] Span),

    #[error("`let` cannot be declared as a variable name inside of a `{0}` declaration")]
    #[diagnostic()]
    InvalidLetDeclaration(String, #[label("Rename the let identifier here")] Span),

    #[error("Missing initializer in destructuring declaration")]
    #[diagnostic()]
    InvalidDestrucuringDeclaration(
        #[label("Missing initializer in destructuring declaration")] Span,
    ),

    #[error("Missing initializer in const declaration")]
    #[diagnostic()]
    MissinginitializerInConst(#[label("const declaration need an initializer")] Span),

    #[error("Functions cannot be labelled")]
    #[diagnostic(help("This is not allowed in strict mode starting with ECMAScript 2015."))]
    FunctionsCannotBeLabelled(#[label("Functions cannot be labelled")] Span),

    #[error("Cannot use {0} outside a method")]
    MethodCode(&'static str, #[label("Cannot use {0} outside a method")] Span),

    #[error("Cannot use {0} outside a module")]
    #[diagnostic()]
    ModuleCode(&'static str, #[label("Cannot use {0} outside a module")] Span),

    #[error("Lexical declaration cannot appear in a single-statement context")]
    #[diagnostic(help("Wrap this declaration in a block statement"))]
    LexicalDeclarationSingleStatement(#[label("Lexical declaration is not allowed here")] Span),

    #[error("Invalid function declaration")]
    #[diagnostic(help(
        "In strict mode code, functions can only be declared at top level or inside a block"
    ))]
    FunctionDeclarationStrict(#[label("function declaration is not allowed here")] Span),

    #[error("Async functions can only be declared at the top level or inside a block")]
    #[diagnostic()]
    AsyncFunctionDeclaration(
        #[label("Async functions can only be declared at the top level or inside a block")] Span,
    ),

    #[error("Generators can only be declared at the top level or inside a block")]
    #[diagnostic()]
    GeneratorFunctionDeclaration(
        #[label("Generators can only be declared at the top level or inside a block")] Span,
    ),

    #[error("Invalid function declaration")]
    #[diagnostic(help(
        "In non-strict mode code, functions can only be declared at top level, inside a block, or as the body of an if statement"
    ))]
    FunctionDeclarationNonStrict(#[label("function declaration is not allowed here")] Span),

    #[error("`await` is only allowed within async functions and at the top levels of modules")]
    #[diagnostic()]
    AwaitExpression(
        #[label("`await` is only allowed within async functions and at the top levels of modules")]
        Span,
    ),

    #[error("A 'yield' expression is only allowed in a generator body.")]
    #[diagnostic()]
    YieldExpression(#[label("A 'yield' expression is only allowed in a generator body.")] Span),

    #[error("Invalid class declaration")]
    #[diagnostic()]
    ClassDeclaration(#[label("Classes can only be declared at top level or inside a block")] Span),

    #[error("Rest element must be last element")]
    #[diagnostic()]
    RestElement(#[label("Rest element must be last element")] Span),

    #[error("Spread must be last element")]
    #[diagnostic()]
    SpreadLastElement(#[label("Spread must be last element")] Span),

    #[error("Unexpected trailing comma after rest element")]
    #[diagnostic()]
    RestElementTraillingComma(#[label("Unexpected trailing comma after rest element")] Span),

    #[error("Invalid rest argument")]
    #[diagnostic(help("Expected identifier in rest argument"))]
    InvalidRestArgument(#[label("Invalid rest argument")] Span),

    #[error("Invalid parenthesized parameter")]
    #[diagnostic(help("remove the parentheses"))]
    InvalidParenthesizedParameter(#[label("Invliad parenthesized parameter")] Span),

    #[error("Invalid parenthesized pattern")]
    #[diagnostic()]
    InvalidParenthesizedPattern(#[label("Invliad parenthesized pattern")] Span),

    #[error("Invalid assignment")]
    #[diagnostic()]
    InvalidAssignment(#[label("Cannot assign to this expression")] Span),

    #[error("Optional chaining cannot appear in the callee of new expressions")]
    #[diagnostic()]
    NewOptionalChain(
        #[label("Optional chaining cannot appear in the callee of new expressions")] Span,
    ),

    #[error("The left-hand side of a `for...of` statement may not be `async`")]
    #[diagnostic()]
    ForLoopAsyncOf(
        #[label("The left-hand side of a `for...of` statement may not be `async`")] Span,
    ),

    #[error("await can only be used in conjunction with `for...of` statements")]
    #[diagnostic()]
    ForAwait(#[label("await can only be used in conjunction with `for...of` statements")] Span),

    #[error("Cannot use new with dynamic import")]
    #[diagnostic()]
    NewDynamicImport(#[label("Cannot use new with dynamic import")] Span),

    #[error("'{0}' declaration can only be used at the top level of a module")]
    #[diagnostic()]
    TopLevel(&'static str, #[label("'{0}' declaration can only appear at the top level")] Span),

    #[error("Duplicated export '{0}'")]
    #[diagnostic()]
    DuplicateExport(
        Atom,
        #[label("Export has already been declared here")] Span,
        #[label("It cannot be redeclared here")] Span,
    ),

    #[error("Unexpected private field")]
    #[diagnostic(help(
        "Private names are only allowed in property accesses (`obj.#field`) or in `in` expressions (`#field in obj`)."
    ))]
    UnexpectedPrivateIdentifier(#[label("Unexpected private field")] Span),

    #[error("Classes can't have an element named '#constructor'")]
    #[diagnostic()]
    PrivateNameConstructor(#[label("Classes can't have an element named '#constructor'")] Span),

    #[error("Private field '{0}' must be declared in an enclosing class")]
    #[diagnostic()]
    PrivateFieldUndeclared(
        Atom,
        #[label("Private field '{0}' must be declared in an enclosing class")] Span,
    ),

    #[error("Unexpected private identifier")]
    #[diagnostic()]
    PrivateNotInClass(
        Atom,
        #[label("Private identifier '#{0}' is not allowed outside class bodies")] Span,
    ),

    #[error("Classes may not have a static property named prototype")]
    #[diagnostic()]
    StaticPrototype(#[label("Classes may not have a static property named prototype")] Span),

    #[error("Constructor can't have get/set modifier")]
    #[diagnostic()]
    ConstructorGetterSetter(#[label("Constructor can't have get/set modifier")] Span),

    #[error("Constructor can't be an async method")]
    #[diagnostic()]
    ConstructorAsync(#[label("Constructor can't be an async method")] Span),

    #[error("Cannot use `{0}` as an identifier in an async context")]
    #[diagnostic()]
    IdentifierAsync(&'static str, #[label("{0} cannot be used here")] Span),

    #[error("Cannot use `{0}` as an identifier in a generator context")]
    #[diagnostic()]
    IdentifierGenerator(&'static str, #[label("{0} cannot be used here")] Span),

    #[error("Constructor can't be a generator")]
    #[diagnostic()]
    ConstructorGenerator(#[label("Constructor can't be a generator")] Span),

    #[error("Classes can't have a field named 'constructor'")]
    #[diagnostic()]
    FieldConstructor(#[label("Classes can't have a field named 'constructor'")] Span),

    #[error("Multiple constructor implementations are not allowed.")]
    #[diagnostic()]
    DuplicateConstructor(
        #[label("constructor has already been declared here")] Span,
        #[label("it cannot be redeclared here")] Span,
    ),

    #[error("An export name cannot include a unicode lone surrogate")]
    #[diagnostic()]
    ExportLoneSurrogate(#[label("An export name cannot include a unicode lone surrogate")] Span),

    #[error("A string literal cannot be used as an exported binding without `from`")]
    #[diagnostic(help("Did you mean `export {{ '{0}' as '{1}' }} from 'some-module'`?"))]
    ExportNamedString(
        Atom,
        Atom,
        #[label("A string literal cannot be used as an exported binding without `from`")] Span,
    ),

    #[error("Bad escape sequence in untagged template literal")]
    #[diagnostic()]
    TemplateLiteral(#[label("Bad escape sequence in untagged template literal")] Span),

    #[error("Delete of an unqualified identifier in strict mode.")]
    #[diagnostic()]
    DeleteOfUnqualified(#[label("Delete of an unqualified identifier in strict mode")] Span),

    #[error("'with' statements are not allowed")]
    #[diagnostic()]
    WithStatement(#[label("'with' statements are not allowed")] Span),

    #[error("Private fields can not be deleted")]
    #[diagnostic()]
    DeletePrivateField(#[label("Private fields can not be deleted")] Span),

    #[error("Empty parenthesized expression")]
    #[diagnostic()]
    EmptyParenthesizedExpression(#[label("Expected an expression here")] Span),

    #[error("Undefined export")]
    #[diagnostic()]
    UndefinedExport(Atom, #[label("Export '{0}' is not defined")] Span),

    #[error("Logical expressions and coalesce expressions cannot be mixed")]
    #[diagnostic(help("Wrap either expression by parentheses"))]
    MixedCoalesce(#[label("Logical expressions and coalesce expressions cannot be mixed")] Span),

    #[error("'Unexpected `{0}`")]
    #[diagnostic()]
    UnexpectedKeyword(&'static str, #[label("'{0}' keyword is unexpected here")] Span),

    #[error("{0} loop variable declaration may not have an initializer")]
    #[diagnostic()]
    UnexpectedInitializerInForLoopHead(
        &'static str,
        #[label("{0} loop variable declaration may not have an initializer")] Span,
    ),

    #[error("Only a single declaration is allowed in a `for...{0}` statement")]
    #[diagnostic()]
    MultipleDeclarationInForLoopHead(
        &'static str,
        #[label("Only a single declaration is allowed in a `for...{0}` statement")] Span,
    ),

    #[error("Illegal newline after {0}")]
    #[diagnostic()]
    IllegalNewline(
        &'static str,
        #[label("{0} starts here")] Span,
        #[label("A newline is not expected here")] Span,
    ),

    #[error("Duplicate parameter name not allowed in this context")]
    #[diagnostic()]
    DuplicateParameter(#[label("Duplicate parameter name not allowed in this context")] Span),

    #[error("Illegal 'use strict' directive in function with non-simple parameter list")]
    #[diagnostic()]
    IllegalUseStrict(
        #[label("Illegal 'use strict' directive in function with non-simple parameter list")] Span,
    ),

    #[error("'arguments' is not allowed in {0}")]
    #[diagnostic()]
    UnexpectedArguments(&'static str, #[label("'arguments' is not allowed in {0}")] Span),

    #[error("Unexpected {0} expression")]
    #[diagnostic()]
    UnexpectedExpression(&'static str, #[label("Unexpected {0} expression")] Span),

    #[error("Unexpected exponentiation expression")]
    #[diagnostic(help("Wrap {0} expression in parentheses to enforce operator precedence"))]
    UnexpectedExponential(&'static str, #[label("Unexpected exponentiation expression")] Span),

    #[error("Tagged template expressions are not permitted in an optional chain")]
    #[diagnostic()]
    OptionalChainTaggedTemplate(
        #[label("Tagged template expressions are not permitted in an optional chain")] Span,
    ),

    #[error("A 'get' accessor must not have any formal parameters.")]
    #[diagnostic()]
    GetterParameters(#[label("A 'get' accessor must not have any formal parameters.")] Span),

    #[error("A 'set' accessor must have exactly one parameter.")]
    #[diagnostic()]
    SetterParameters(#[label("A 'set' accessor must have exactly one parameter.")] Span),

    #[error("A 'set' accessor function argument must not be a rest parameter")]
    #[diagnostic()]
    SetterParametersRestPattern(
        #[label("A 'set' accessor function argument must not be a rest parameter")] Span,
    ),

    #[error("{0} expression not allowed in formal parameter")]
    #[diagnostic()]
    AwaitOrYieldInParameter(
        &'static str,
        #[label("{0} expression not allowed in formal parameter")] Span,
    ),

    #[error("Invalid assignment in object literal")]
    #[diagnostic(help(
        "Did you mean to use a ':'? An '=' can only follow a property name when the containing object literal is part of a destructuring pattern."
    ))]
    CoverInitializedNameError(#[label("Assignment is not allowed here")] Span),

    #[error("Super calls are not permitted outside constructors or in nested functions inside constructors.
")]
    #[diagnostic()]
    UnexpectedSuperCall(
        #[label(
            "Super calls are not permitted outside constructors or in nested functions inside constructors."
        )]
        Span,
    ),

    #[error("'super' can only be referenced in members of derived classes or object literal expressions.
")]
    #[diagnostic()]
    UnexpectedSuperReference(
        #[label("'super' can only be referenced in members of derived classes or object literal expressions.
")]
        Span,
    ),

    #[error("'super' can only be used with function calls or in property accesses")]
    #[diagnostic(help("replace with `super()` or `super.prop` or `super[prop]`"))]
    UnexpectedSuper(
        #[label("'super' can only be used with function calls or in property accesses ")] Span,
    ),

    #[error("'super' can only be referenced in a derived class.")]
    #[diagnostic(help("either remove this super, or extend the class"))]
    SuperWithoutDerivedClass(
        #[label("'super' can only be referenced in a derived class.")] Span,
        #[label("class does not have `extends`")] Span,
    ),

    #[error("Private fields cannot be accessed on super")]
    #[diagnostic()]
    SuperPrivate(#[label("Private fields cannot be accessed on super")] Span),

    #[error("Expected function name")]
    #[diagnostic(help("Function name is required in function declaration or named export"))]
    ExpectFunctionName(#[label("Function name is required here")] Span),

    #[error("Missing catch or finally clause")]
    #[diagnostic()]
    ExpectCatchFinally(#[label("Expected `catch` or `finally` here")] Span),

    #[error("Cannot assign to '{0}' because it is a {1}")]
    #[diagnostic()]
    CannotAssignTo(
        Atom,
        &'static str,
        #[label("Cannot assign to '{0}' because this is a {1}")] Span,
    ),

    #[error("A rest parameter cannot have an initializer")]
    #[diagnostic()]
    ARestParameterCannotHaveAnInitializer(
        #[label("A rest parameter cannot have an initializer")] Span,
    ),

    /* TypeScript */
    #[error("TS1015: Parameter cannot have question mark and initializer")]
    #[diagnostic()]
    ParameterCannotHaveQuestionMarkAndInitializer(
        #[label("Parameter cannot have question mark and initializer")] Span,
    ),

    #[error("TS1047: A rest parameter cannot be optional")]
    #[diagnostic()]
    ARestParameterCannotBeOptional(#[label("A rest parameter cannot be optional")] Span),

    #[error("TS1095: A 'set' accessor cannot have a return type annotation")]
    #[diagnostic()]
    ASetAccessorCannotHaveAReturnTypeAnnotation(
        #[label("A 'set' accessor cannot have a return type annotation")] Span,
    ),

    #[error("TS1098: Type parameter list cannot be empty")]
    #[diagnostic()]
    TypeParameterListCannotBeEmpty(#[label("Type parameter list cannot be empty")] Span),

    #[error("TS1099: Type argument list cannot be empty")]
    #[diagnostic()]
    TypeArgumentListCannotBeEmpty(#[label("Type argument list cannot be empty")] Span),

    #[error("TS1108: A 'return' statement can only be used within a function body")]
    #[diagnostic()]
    ReturnStatementOnlyInFunctionBody(
        #[label("A 'return' statement can only be used within a function body.")] Span,
    ),

    #[error("TS1164: Computed property names are not allowed in enums")]
    #[diagnostic()]
    ComputedPropertyNamesAreNotAllowedInEnums(
        #[label("Computed property names are not allowed in enums")] Span,
    ),

    #[error("TS1313: The body of an 'if' statement cannot be the empty statement")]
    #[diagnostic()]
    TheBodyOfAnIfStatementCannotBeTheEmptyStatement(
        #[label("The body of an 'if' statement cannot be the empty statement")] Span,
    ),

    #[error("TS1317: A parameter property cannot be declared using a rest parameter")]
    #[diagnostic()]
    AParameterPropertyCannotBeDeclaredUsingARestParameter(
        #[label("A parameter property cannot be declared using a rest parameter")] Span,
    ),

    #[error("TS2452: An enum member cannot have a numeric name")]
    #[diagnostic()]
    AnEnumMemberCannotHaveANumericName(#[label("An enum member cannot have a numeric name")] Span),
}
