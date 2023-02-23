//! ECMAScript Token Kinds

use std::fmt;

use oxc_ast::Atom;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
#[non_exhaustive]
pub enum Kind {
    Undetermined,
    #[default]
    Eof,
    WhiteSpace,
    NewLine,
    Comment,
    MultiLineComment,
    // 12.6 identifier
    Ident,
    // 12.6.2 keyword
    Await,
    Break,
    Case,
    Catch,
    Class,
    Const,
    Continue,
    Debugger,
    Default,
    Delete,
    Do,
    Else,
    Enum,
    Export,
    Extends,
    Finally,
    For,
    Function,
    If,
    Import,
    In,
    Instanceof,
    New,
    Return,
    Super,
    Switch,
    This,
    Throw,
    Try,
    Typeof,
    Var,
    Void,
    While,
    With,
    // Contextual Keywords
    Async,
    From,
    Get,
    Meta, // import.meta
    Of,
    Set,
    Target, // new.target
    Accessor,
    // TypeScript Contextual Keywords
    Abstract,
    As,
    Asserts,
    Assert,
    Any,
    Boolean,
    Constructor,
    Declare,
    Infer,
    Intrinsic,
    Is,
    KeyOf,
    Module,
    Namespace,
    Never,
    Out,
    Readonly,
    Require,
    Number,
    Object,
    Satisfies,
    String, // the "string" keyword for TypeScript
    Symbol,
    Type,
    Undefined,
    Unique,
    Unknown,
    Global,
    BigInt,
    Override,
    // Future keywords (strict mode reserved words)
    Implements,
    Interface,
    Let,
    Package,
    Private,
    Protected,
    Public,
    Static,
    Yield,
    // 12.7 punctuators
    Amp, // &
    Amp2,
    Amp2Eq,
    AmpEq,
    Bang,
    Caret,
    CaretEq,
    Colon,
    Comma,
    Dot,
    Dot3, // ...
    Eq,
    Eq2,
    Eq3,
    GtEq, // >=
    LAngle,
    LBrack,
    LCurly,
    LParen,
    LtEq, // <=
    Minus,
    Minus2,
    MinusEq,
    Neq,
    Neq2,
    Percent,
    PercentEq,
    Pipe,
    Pipe2,
    Pipe2Eq,
    PipeEq,
    Plus,
    Plus2,
    PlusEq,
    Question,
    Question2,
    Question2Eq,
    QuestionDot,
    RAngle,
    RBrack,
    RCurly,
    RParen,
    Semicolon,
    ShiftLeft,     // <<
    ShiftLeftEq,   // <<=
    ShiftRight,    // >>
    ShiftRight3,   // >>>
    ShiftRight3Eq, // >>>=
    ShiftRightEq,  // >>=
    Slash,
    SlashEq,
    Star,
    Star2,
    Star2Eq,
    StarEq,
    Tilde,
    // arrow function
    Arrow,
    // 12.8.1 Null Literals
    Null,
    // 12.8.2 Boolean Literals
    True,
    False,
    // 12.8.3 Numeric Literals
    Decimal,
    Float,
    Binary,
    Octal,
    Hex,
    // 12.8.4 String Literals
    /// String Type
    Str,
    // 12.8.5 Regular Expression Literals
    RegExp,
    // 12.8.6 Template Literal
    NoSubstitutionTemplate,
    TemplateHead,
    TemplateMiddle,
    TemplateTail,
    // es2022 Private Identifier
    PrivateIdentifier,
    // JSX
    JSXText,
    // Decorator
    At,
}

#[allow(clippy::enum_glob_use)]
use self::Kind::*;
impl Kind {
    #[must_use]
    pub const fn is_eof(self) -> bool {
        matches!(self, Eof)
    }

    #[must_use]
    pub const fn is_trivia(self) -> bool {
        matches!(self, WhiteSpace | NewLine | Comment | MultiLineComment)
    }

    #[must_use]
    pub const fn is_number(self) -> bool {
        matches!(self, Float | Decimal | Binary | Octal | Hex)
    }

    #[must_use]
    pub fn matches_number_char(self, c: char) -> bool {
        match self {
            Decimal => c.is_ascii_digit(),
            Binary => matches!(c, '0'..='1'),
            Octal => matches!(c, '0'..='7'),
            Hex => c.is_ascii_hexdigit(),
            _ => unreachable!(),
        }
    }

    /// [Identifiers](https://tc39.es/ecma262/#sec-identifiers)
    /// `IdentifierReference`
    #[must_use]
    pub fn is_identifier_reference(self, r#yield: bool, r#await: bool) -> bool {
        self.is_identifier() || (!r#yield && self == Yield) || (!r#await && self == Await)
    }

    /// `BindingIdentifier`
    #[must_use]
    pub const fn is_binding_identifier(self) -> bool {
        self.is_identifier() || matches!(self, Yield | Await)
    }

    /// `LabelIdentifier`
    #[must_use]
    pub fn is_label_identifier(self, r#yield: bool, r#await: bool) -> bool {
        self.is_identifier() || (!r#yield && self == Yield) || (!r#await && self == Await)
    }

    /// Identifier
    /// `IdentifierName` but not `ReservedWord`
    #[must_use]
    pub const fn is_identifier(self) -> bool {
        self.is_identifier_name() && !self.is_reserved_keyword()
    }

    /// `IdentifierName`
    #[must_use]
    pub const fn is_identifier_name(self) -> bool {
        matches!(self, Ident) || self.is_all_keyword()
    }

    /// Check the succeeding token of a `let` keyword
    // let { a, b } = c, let [a, b] = c, let ident
    #[must_use]
    pub fn is_after_let(self) -> bool {
        self != Self::In && (matches!(self, LCurly | LBrack | Ident) || self.is_all_keyword())
    }

    /// Section 13.2.4 Literals
    /// Literal :
    ///     `NullLiteral`
    ///     `BooleanLiteral`
    ///     `NumericLiteral`
    ///     `StringLiteral`
    #[must_use]
    pub const fn is_literal(self) -> bool {
        matches!(self, Null | True | False | Str | RegExp) || self.is_number()
    }

    #[must_use]
    pub const fn is_after_await_or_yield(self) -> bool {
        !self.is_binary_operator() && (self.is_literal() || self.is_identifier_name())
    }

    /// Section 13.2.6 Object Initializer
    /// `LiteralPropertyName` :
    ///     `IdentifierName`
    ///     `StringLiteral`
    ///     `NumericLiteral`
    #[must_use]
    pub fn is_literal_property_name(self) -> bool {
        self.is_identifier_name() || self == Str || self.is_number()
    }

    #[must_use]
    pub const fn is_variable_declaration(self) -> bool {
        matches!(self, Var | Let | Const)
    }

    /// Section 15.4 Method Definitions
    /// `ClassElementName`[Yield, Await] :
    ///   `PropertyName`[?Yield, ?Await]
    ///   `PrivateIdentifier`
    /// `PropertyName`[Yield, Await] :
    ///   `LiteralPropertyName`
    ///   `ComputedPropertyName`[?Yield, ?Await]
    #[must_use]
    pub fn is_class_element_name_start(self) -> bool {
        self.is_literal_property_name() || matches!(self, LBrack | PrivateIdentifier)
    }

    #[must_use]
    #[rustfmt::skip]
    pub const fn is_assignment_operator(self) -> bool {
        matches!(self, Eq | PlusEq | MinusEq | StarEq | SlashEq | PercentEq | ShiftLeftEq | ShiftRightEq
            | ShiftRight3Eq | Pipe2Eq | Amp2Eq | PipeEq | CaretEq | AmpEq | Question2Eq
            | Star2Eq)
    }

    #[must_use]
    #[rustfmt::skip]
    pub const fn is_binary_operator(self) -> bool {
        matches!(self, Eq2 | Neq | Eq3 | Neq2 | LAngle | LtEq | RAngle | GtEq | ShiftLeft | ShiftRight
            | ShiftRight3 | Plus | Minus | Star | Slash | Percent | Pipe | Caret | Amp | In
            | Instanceof | Star2)
    }

    #[must_use]
    pub const fn is_logical_operator(self) -> bool {
        matches!(self, Pipe2 | Amp2 | Question2)
    }

    #[must_use]
    pub const fn is_unary_operator(self) -> bool {
        matches!(self, Minus | Plus | Bang | Tilde | Typeof | Void | Delete)
    }

    #[must_use]
    pub const fn is_update_operator(self) -> bool {
        matches!(self, Plus2 | Minus2)
    }

    /// [Keywords and Reserved Words](https://tc39.es/ecma262/#sec-keywords-and-reserved-words)
    #[must_use]
    pub const fn is_all_keyword(self) -> bool {
        self.is_reserved_keyword()
            || self.is_contextual_keyword()
            || self.is_strict_mode_contextual_keyword()
            || self.is_future_reserved_keyword()
    }

    #[must_use]
    #[rustfmt::skip]
    pub const fn is_reserved_keyword(self) -> bool {
        matches!(self, Await | Break | Case | Catch | Class | Const | Continue | Debugger | Default
            | Delete | Do | Else | Enum | Export | Extends | False | Finally | For | Function | If
            | Import | In | Instanceof | New | Null | Return | Super | Switch | This | Throw
            | True | Try | Typeof | Var | Void | While | With | Yield)
    }

    #[must_use]
    #[rustfmt::skip]
    pub const fn is_strict_mode_contextual_keyword(self) -> bool {
        matches!(self, Let | Static | Implements | Interface | Package | Private | Protected | Public)
    }

    #[must_use]
    #[rustfmt::skip]
    pub const fn is_contextual_keyword(self) -> bool {
        matches!(self, Async | From | Get | Meta | Of | Set | Target | Accessor | Abstract | As | Asserts
            | Assert | Any | Boolean | Constructor | Declare | Infer | Intrinsic | Is | KeyOf | Module
            | Namespace | Never | Out | Readonly | Require | Number | Object | Satisfies | String
            | Symbol | Type | Undefined | Unique | Unknown | Global | BigInt | Override)
    }

    #[must_use]
    #[rustfmt::skip]
    pub const fn is_future_reserved_keyword(self) -> bool {
        matches!(self, Implements | Interface | Package | Private | Protected | Public | Static)
    }

    #[must_use]
    #[rustfmt::skip]
    pub const fn is_at_expression(self) -> bool {
        self.is_unary_operator()
            || self.is_update_operator()
            || self.is_reserved_keyword()
            || self.is_literal()
            || matches!(self, Neq | LParen | LBrack | LCurly | LAngle | Dot3
                | Slash | SlashEq | TemplateHead | NoSubstitutionTemplate | PrivateIdentifier | Ident | Async)
    }

    #[must_use]
    pub fn match_keyword(s: &str, ts_enabled: bool) -> (Self, Atom) {
        let len = s.len();
        if len == 1 || len >= 12 {
            return (Ident, Atom::new(s));
        }
        // perf: Atom::new_inline is a `const` fn
        match s {
            "as" => (As, Atom::new_inline("as")),
            "do" => (Do, Atom::new_inline("do")),
            "if" => (If, Atom::new_inline("if")),
            "in" => (In, Atom::new_inline("in")),
            "is" => (Is, Atom::new_inline("is")),
            "of" => (Of, Atom::new_inline("of")),

            "any" => (Any, Atom::new_inline("any")),
            "for" => (For, Atom::new_inline("for")),
            "get" => (Get, Atom::new_inline("get")),
            "let" => (Let, Atom::new_inline("let")),
            "new" => (New, Atom::new_inline("new")),
            "out" => (Out, Atom::new_inline("out")),
            "set" => (Set, Atom::new_inline("set")),
            "try" => (Try, Atom::new_inline("try")),
            "var" => (Var, Atom::new_inline("var")),

            "case" => (Case, Atom::new_inline("case")),
            "else" => (Else, Atom::new_inline("else")),
            "enum" => (Enum, Atom::new_inline("enum")),
            "from" => (From, Atom::new_inline("from")),
            "meta" => (Meta, Atom::new_inline("meta")),
            "null" => (Null, Atom::new_inline("null")),
            "this" => (This, Atom::new_inline("this")),
            "true" => (True, Atom::new_inline("true")),
            "type" => (Type, Atom::new_inline("type")),
            "void" => (Void, Atom::new_inline("void")),
            "with" => (With, Atom::new_inline("with")),

            "async" => (Async, Atom::new_inline("async")),
            "await" => (Await, Atom::new_inline("await")),
            "break" => (Break, Atom::new_inline("break")),
            "catch" => (Catch, Atom::new_inline("catch")),
            "class" => (Class, Atom::new_inline("class")),
            "const" => (Const, Atom::new_inline("const")),
            "false" => (False, Atom::new_inline("false")),
            "infer" => (Infer, Atom::new_inline("infer")),
            "keyof" => (KeyOf, Atom::new_inline("keyof")),
            "never" => (Never, Atom::new_inline("never")),
            "super" => (Super, Atom::new_inline("super")),
            "throw" => (Throw, Atom::new_inline("throw")),
            "while" => (While, Atom::new_inline("while")),
            "yield" => (Yield, Atom::new_inline("yield")),

            "assert" => (Assert, Atom::new_inline("assert")),
            "bigint" => (BigInt, Atom::new_inline("bigint")),
            "delete" => (Delete, Atom::new_inline("delete")),
            "export" => (Export, Atom::new_inline("export")),
            "global" => (Global, Atom::new_inline("global")),
            "import" => (Import, Atom::new_inline("import")),
            "module" => (Module, Atom::new_inline("module")),
            "number" => (Number, Atom::new_inline("number")),
            "object" => (Object, Atom::new_inline("object")),
            "public" => (Public, Atom::new_inline("public")),
            "return" => (Return, Atom::new_inline("return")),
            "static" => (Static, Atom::new_inline("static")),
            "string" => (String, Atom::new_inline("string")),
            "switch" => (Switch, Atom::new_inline("switch")),
            "symbol" => (Symbol, Atom::new_inline("symbol")),
            "target" => (Target, Atom::new_inline("target")),
            "typeof" => (Typeof, Atom::new_inline("typeof")),
            "unique" => (Unique, Atom::new_inline("unique")),

            "asserts" => (Asserts, Atom::new_inline("asserts")),
            "boolean" => (Boolean, Atom::new_inline("boolean")),
            "declare" => (Declare, Atom::new_inline("declare")),
            "default" => (Default, Atom::new_inline("default")),
            "extends" => (Extends, Atom::new_inline("extends")),
            "finally" => (Finally, Atom::new_inline("finally")),
            "package" => (Package, Atom::new_inline("package")),
            "private" => (Private, Atom::new_inline("private")),
            "require" => (Require, Atom::new_inline("require")),
            "unknown" => (Unknown, Atom::new_inline("unknown")),

            "abstract" if ts_enabled => (Abstract, Atom::new_inline("abstract")),
            "accessor" => (Accessor, Atom::new_inline("accessor")),
            "continue" => (Continue, Atom::new_inline("continue")),
            "debugger" => (Debugger, Atom::new_inline("debugger")),
            "function" => (Function, Atom::new_inline("function")),
            "override" => (Override, Atom::new_inline("override")),
            "readonly" => (Readonly, Atom::new_inline("readonly")),

            "interface" => (Interface, Atom::new_inline("interface")),
            "intrinsic" => (Intrinsic, Atom::new_inline("intrinsic")),
            "namespace" => (Namespace, Atom::new_inline("namespace")),
            "protected" => (Protected, Atom::new_inline("protected")),
            "satisfies" => (Satisfies, Atom::new_inline("satisfies")),
            "undefined" => (Undefined, Atom::new_inline("undefined")),

            "implements" => (Implements, Atom::new_inline("implements")),
            "instanceof" => (Instanceof, Atom::new_inline("instanceof")),

            "constructor" => (Constructor, Atom::new_inline("constructor")),
            _ => (Ident, Atom::new(s)),
        }
    }

    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub const fn to_str(self) -> &'static str {
        match self {
            Undetermined => "Unknown",
            Eof => "EOF",
            NewLine => "\n",
            Comment => "//",
            MultiLineComment => "/** */",
            WhiteSpace => " ",
            Ident => "Identifier",
            Await => "await",
            Break => "break",
            Case => "case",
            Catch => "catch",
            Class => "class",
            Const => "const",
            Continue => "continue",
            Debugger => "debugger",
            Default => "default",
            Delete => "delete",
            Do => "do",
            Else => "else",
            Enum => "enum",
            Export => "export",
            Extends => "extends",
            Finally => "finally",
            For => "for",
            Function => "function",
            If => "if",
            Import => "import",
            In => "in",
            Instanceof => "instanceof",
            New => "new",
            Return => "return",
            Super => "super",
            Switch => "switch",
            This => "this",
            Throw => "throw",
            Try => "try",
            Typeof => "typeof",
            Var => "var",
            Void => "void",
            While => "while",
            With => "with",
            As => "as",
            Async => "async",
            From => "from",
            Get => "get",
            Meta => "meta",
            Of => "of",
            Set => "set",
            Asserts => "asserts",
            Accessor => "accessor",
            Abstract => "abstract",
            Readonly => "readonly",
            Declare => "declare",
            Override => "override",
            Type => "type",
            Target => "target",
            Implements => "implements",
            Interface => "interface",
            Package => "package",
            Private => "private",
            Protected => "protected",
            Public => "public",
            Static => "static",
            Let => "let",
            Yield => "yield",
            Amp => "&",
            Amp2 => "&&",
            Amp2Eq => "&&=",
            AmpEq => "&=",
            Bang => "!",
            Caret => "^",
            CaretEq => "^=",
            Colon => ":",
            Comma => ",",
            Dot => ".",
            Dot3 => "...",
            Eq => "=",
            Eq2 => "==",
            Eq3 => "===",
            GtEq => ">=",
            LAngle => "<",
            LBrack => "[",
            LCurly => "{",
            LParen => "(",
            LtEq => "<=",
            Minus => "-",
            Minus2 => "--",
            MinusEq => "-=",
            Neq => "!=",
            Neq2 => "!==",
            Percent => "%",
            PercentEq => "%=",
            Pipe => "|",
            Pipe2 => "||",
            Pipe2Eq => "||=",
            PipeEq => "|=",
            Plus => "+",
            Plus2 => "++",
            PlusEq => "+=",
            Question => "?",
            Question2 => "??",
            Question2Eq => "??=",
            QuestionDot => "?.",
            RAngle => ">",
            RBrack => "]",
            RCurly => "}",
            RParen => ")",
            Semicolon => ";",
            ShiftLeft => "<<",
            ShiftLeftEq => "<<=",
            ShiftRight => ">>",
            ShiftRight3 => ">>>",
            ShiftRight3Eq => ">>>=",
            ShiftRightEq => ">>=",
            Slash => "/",
            SlashEq => "/=",
            Star => "*",
            Star2 => "**",
            Star2Eq => "**=",
            StarEq => "*=",
            Tilde => "~",
            Arrow => "=>",
            Null => "null",
            True => "true",
            False => "false",
            Decimal => "decimal",
            Float => "float",
            Binary => "binary",
            Octal => "octal",
            Hex => "hex",
            Str | String => "string",
            RegExp => "/regexp/",
            NoSubstitutionTemplate => "${}",
            TemplateHead => "${",
            TemplateMiddle => "${expr}",
            TemplateTail => "$}",
            PrivateIdentifier => "#identifier",
            JSXText => "jsx",
            At => "@",
            Assert => "assert",
            Any => "any",
            Boolean => "boolean",
            Constructor => "constructor",
            Infer => "infer",
            Intrinsic => "intrinsic",
            Is => "is",
            KeyOf => "keyof",
            Module => "module",
            Namespace => "namaespace",
            Never => "never",
            Out => "out",
            Require => "require",
            Number => "number",
            Object => "object",
            Satisfies => "satisfies",
            Symbol => "symbol",
            Undefined => "undefined",
            Unique => "unique",
            Unknown => "unknown",
            Global => "global",
            BigInt => "bigint",
        }
    }

    #[must_use]
    #[rustfmt::skip]
    pub const fn can_follow_type_arguments_in_expr(self) -> bool {
        matches!(self, Self::LParen | Self::NoSubstitutionTemplate | Self::TemplateHead
            | Self::Comma | Self::Dot | Self::QuestionDot | Self::RParen | Self::RBrack
            | Self::Colon | Self::Semicolon | Self::Question | Self::Eq3 | Self::Eq2
            | Self::Neq | Self::Neq2 | Self::Amp2 | Self::Pipe2 | Self::Question2
            | Self::Caret | Self::Amp | Self::Pipe | Self::RCurly | Self::Eof)
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}
