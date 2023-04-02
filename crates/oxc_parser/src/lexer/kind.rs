//! ECMAScript Token Kinds

use std::fmt;

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
    pub fn is_eof(self) -> bool {
        matches!(self, Eof)
    }

    #[must_use]
    pub fn is_trivia(self) -> bool {
        matches!(self, WhiteSpace | NewLine | Comment | MultiLineComment)
    }

    #[must_use]
    pub fn is_number(self) -> bool {
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
    pub fn is_binding_identifier(self) -> bool {
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
    pub fn is_identifier(self) -> bool {
        self.is_identifier_name() && !self.is_reserved_keyword()
    }

    /// `IdentifierName`
    #[must_use]
    pub fn is_identifier_name(self) -> bool {
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
    pub fn is_literal(self) -> bool {
        matches!(self, Null | True | False | Str | RegExp) || self.is_number()
    }

    #[must_use]
    pub fn is_after_await_or_yield(self) -> bool {
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
    pub fn is_variable_declaration(self) -> bool {
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
    pub fn is_assignment_operator(self) -> bool {
        matches!(self, Eq | PlusEq | MinusEq | StarEq | SlashEq | PercentEq | ShiftLeftEq | ShiftRightEq
            | ShiftRight3Eq | Pipe2Eq | Amp2Eq | PipeEq | CaretEq | AmpEq | Question2Eq
            | Star2Eq)
    }

    #[must_use]
    #[rustfmt::skip]
    pub fn is_binary_operator(self) -> bool {
        matches!(self, Eq2 | Neq | Eq3 | Neq2 | LAngle | LtEq | RAngle | GtEq | ShiftLeft | ShiftRight
            | ShiftRight3 | Plus | Minus | Star | Slash | Percent | Pipe | Caret | Amp | In
            | Instanceof | Star2)
    }

    #[must_use]
    pub fn is_logical_operator(self) -> bool {
        matches!(self, Pipe2 | Amp2 | Question2)
    }

    #[must_use]
    pub fn is_unary_operator(self) -> bool {
        matches!(self, Minus | Plus | Bang | Tilde | Typeof | Void | Delete)
    }

    #[must_use]
    pub fn is_update_operator(self) -> bool {
        matches!(self, Plus2 | Minus2)
    }

    /// [Keywords and Reserved Words](https://tc39.es/ecma262/#sec-keywords-and-reserved-words)
    #[must_use]
    pub fn is_all_keyword(self) -> bool {
        self.is_reserved_keyword()
            || self.is_contextual_keyword()
            || self.is_strict_mode_contextual_keyword()
            || self.is_future_reserved_keyword()
    }

    #[must_use]
    #[rustfmt::skip]
    pub fn is_reserved_keyword(self) -> bool {
        matches!(self, Await | Break | Case | Catch | Class | Const | Continue | Debugger | Default
            | Delete | Do | Else | Enum | Export | Extends | False | Finally | For | Function | If
            | Import | In | Instanceof | New | Null | Return | Super | Switch | This | Throw
            | True | Try | Typeof | Var | Void | While | With | Yield)
    }

    #[must_use]
    #[rustfmt::skip]
    pub fn is_strict_mode_contextual_keyword(self) -> bool {
        matches!(self, Let | Static | Implements | Interface | Package | Private | Protected | Public)
    }

    #[must_use]
    #[rustfmt::skip]
    pub fn is_contextual_keyword(self) -> bool {
        matches!(self, Async | From | Get | Meta | Of | Set | Target | Accessor | Abstract | As | Asserts
            | Assert | Any | Boolean | Constructor | Declare | Infer | Intrinsic | Is | KeyOf | Module
            | Namespace | Never | Out | Readonly | Require | Number | Object | Satisfies | String
            | Symbol | Type | Undefined | Unique | Unknown | Global | BigInt | Override)
    }

    #[must_use]
    #[rustfmt::skip]
    pub fn is_future_reserved_keyword(self) -> bool {
        matches!(self, Implements | Interface | Package | Private | Protected | Public | Static)
    }

    #[must_use]
    #[rustfmt::skip]
    pub fn is_at_expression(self) -> bool {
        self.is_unary_operator()
            || self.is_update_operator()
            || self.is_reserved_keyword()
            || self.is_literal()
            || matches!(self, Neq | LParen | LBrack | LCurly | LAngle | Dot3
                | Slash | SlashEq | TemplateHead | NoSubstitutionTemplate | PrivateIdentifier | Ident | Async)
    }

    #[must_use]
    pub fn is_template_start_of_tagged_template(self) -> bool {
        matches!(self, NoSubstitutionTemplate | TemplateHead)
    }

    #[must_use]
    #[rustfmt::skip]
    pub fn is_modifier_kind(self) -> bool {
        matches!(self, Abstract | Accessor | Async | Const | Declare | Default
          | Export | In | Out | Public | Private | Protected | Readonly | Static | Override)
    }

    #[must_use]
    pub fn match_keyword(s: &str) -> Self {
        let len = s.len();
        if len == 1 || len >= 12 {
            return Ident;
        }
        Self::match_keyword_impl(s)
    }

    fn match_keyword_impl(s: &str) -> Self {
        match s {
            "as" => As,
            "do" => Do,
            "if" => If,
            "in" => In,
            "is" => Is,
            "of" => Of,

            "any" => Any,
            "for" => For,
            "get" => Get,
            "let" => Let,
            "new" => New,
            "out" => Out,
            "set" => Set,
            "try" => Try,
            "var" => Var,

            "case" => Case,
            "else" => Else,
            "enum" => Enum,
            "from" => From,
            "meta" => Meta,
            "null" => Null,
            "this" => This,
            "true" => True,
            "type" => Type,
            "void" => Void,
            "with" => With,

            "async" => Async,
            "await" => Await,
            "break" => Break,
            "catch" => Catch,
            "class" => Class,
            "const" => Const,
            "false" => False,
            "infer" => Infer,
            "keyof" => KeyOf,
            "never" => Never,
            "super" => Super,
            "throw" => Throw,
            "while" => While,
            "yield" => Yield,

            "assert" => Assert,
            "bigint" => BigInt,
            "delete" => Delete,
            "export" => Export,
            "global" => Global,
            "import" => Import,
            "module" => Module,
            "number" => Number,
            "object" => Object,
            "public" => Public,
            "return" => Return,
            "static" => Static,
            "string" => String,
            "switch" => Switch,
            "symbol" => Symbol,
            "target" => Target,
            "typeof" => Typeof,
            "unique" => Unique,

            "asserts" => Asserts,
            "boolean" => Boolean,
            "declare" => Declare,
            "default" => Default,
            "extends" => Extends,
            "finally" => Finally,
            "package" => Package,
            "private" => Private,
            "require" => Require,
            "unknown" => Unknown,

            "abstract" => Abstract,
            "accessor" => Accessor,
            "continue" => Continue,
            "debugger" => Debugger,
            "function" => Function,
            "override" => Override,
            "readonly" => Readonly,

            "interface" => Interface,
            "intrinsic" => Intrinsic,
            "namespace" => Namespace,
            "protected" => Protected,
            "satisfies" => Satisfies,
            "undefined" => Undefined,

            "implements" => Implements,
            "instanceof" => Instanceof,

            "constructor" => Constructor,
            _ => Ident,
        }
    }

    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub fn to_str(self) -> &'static str {
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
    pub fn can_follow_type_arguments_in_expr(self) -> bool {
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
