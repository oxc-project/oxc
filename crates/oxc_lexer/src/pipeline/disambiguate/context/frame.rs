use crate::token::TokenKind;

macro_rules! kw_codes {
    ($($name:ident = $kind:ident),* $(,)?) => {
        $(pub(super) const $name: u8 = TokenKind::$kind as u8;)*
    };
}

kw_codes! {
    K_BREAK = KwBreak, K_CASE = KwCase, K_CATCH = KwCatch,
    K_CLASS = KwClass, K_CONST = KwConst, K_CONTINUE = KwContinue,
    K_DEBUGGER = KwDebugger, K_DEFAULT = KwDefault, K_DELETE = KwDelete,
    K_DO = KwDo, K_ELSE = KwElse, K_ENUM = KwEnum, K_EXPORT = KwExport,
    K_EXTENDS = KwExtends, K_FALSE = KwFalse, K_FINALLY = KwFinally,
    K_FOR = KwFor, K_FUNCTION = KwFunction, K_IF = KwIf,
    K_IMPORT = KwImport, K_IN = KwIn, K_INSTANCEOF = KwInstanceof,
    K_NEW = KwNew, K_NULL = KwNull, K_RETURN = KwReturn, K_SUPER = KwSuper,
    K_SWITCH = KwSwitch, K_THIS = KwThis, K_THROW = KwThrow,
    K_TRUE = KwTrue, K_TRY = KwTry, K_TYPEOF = KwTypeof, K_VAR = KwVar,
    K_VOID = KwVoid, K_WHILE = KwWhile, K_WITH = KwWith, K_YIELD = KwYield,
    K_LET = KwLet, K_STATIC = KwStatic, K_ASYNC = KwAsync,
    K_AWAIT = KwAwait, K_OF = KwOf, K_FROM = KwFrom, K_AS = KwAs,
    K_ABSTRACT = KwAbstract, K_ACCESSOR = KwAccessor,
    K_ASSERTS = KwAsserts, K_DECLARE = KwDeclare, K_GLOBAL = KwGlobal,
    K_IMPLEMENTS = KwImplements, K_INFER = KwInfer,
    K_INTERFACE = KwInterface, K_IS = KwIs, K_KEYOF = KwKeyof,
    K_MODULE = KwModule, K_NAMESPACE = KwNamespace,
    K_OVERRIDE = KwOverride, K_PRIVATE = KwPrivate,
    K_PROTECTED = KwProtected, K_PUBLIC = KwPublic,
    K_READONLY = KwReadonly, K_SATISFIES = KwSatisfies, K_TYPE = KwType,
    K_UNIQUE = KwUnique, K_USING = KwUsing,
    K_ANY = KwAny, K_BIGINT = KwBigInt, K_BOOLEAN = KwBoolean, K_NEVER = KwNever,
    K_NUMBER = KwNumber, K_OBJECT = KwObject, K_STRING = KwString, K_SYMBOL = KwSymbol,
    K_UNDEFINED = KwUndefined, K_UNKNOWN = KwUnknown,
}

/// any, null, this...: whole types that take no type arguments.
pub(super) fn keyword_type(kw: u8) -> bool {
    matches!(
        kw,
        K_ANY
            | K_BIGINT
            | K_BOOLEAN
            | K_NEVER
            | K_NUMBER
            | K_OBJECT
            | K_STRING
            | K_SYMBOL
            | K_UNDEFINED
            | K_UNKNOWN
            | K_VOID
            | K_NULL
            | K_THIS
            | K_TRUE
            | K_FALSE
    )
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum FrameKind {
    Root,
    // Braces.
    Block,
    FnBody,
    ArrowBody,
    ClassBody,
    StaticBlock,
    Object,
    Pattern,
    TypeLit,
    EnumBody,
    ModuleSpec,
    Container,
    // A template substitution, from a TemplateHead or Middle to the next Middle or Tail.
    Sub,
    // Parens.
    Head,
    Params,
    Call,
    Group,
    TypeParen,
    // Brackets.
    Index,
    Array,
    ComputedKey,
    TypeBracket,
    ArrayPattern,
    // JSX.
    JsxTag,
    JsxElem,
    // Virtual frames (no bracket of their own).
    Concise,
    TypeRegion,
    Angle,
    FnHead,
    ClassHead,
}

// Head kinds.
pub(super) const H_IF: u8 = 1;

pub(super) const H_WHILE: u8 = 2;

pub(super) const H_FOR: u8 = 3;

pub(super) const H_WITH: u8 = 4;

pub(super) const H_SWITCH: u8 = 5;

pub(super) const H_CATCH: u8 = 6;

// Declarator state on statement frames and for-heads (state).
pub(super) const D_NONE: u8 = 0;

pub(super) const D_BINDING: u8 = 1;

pub(super) const D_BOUND: u8 = 2;

pub(super) const D_INIT: u8 = 3;

// Statement register on statement frames (reg).
pub(super) const S_NONE: u8 = 0;

pub(super) const S_CASE: u8 = 1;

pub(super) const S_LABEL: u8 = 2;

pub(super) const S_IMPORT: u8 = 3;

pub(super) const S_EXPORT: u8 = 4;

pub(super) const S_TYPE: u8 = 5;

pub(super) const S_TYPE_NAME: u8 = 6;

pub(super) const S_BREAK: u8 = 7;

pub(super) const S_NAMESPACE: u8 = 8;

pub(super) const S_ENUM: u8 = 9;

pub(super) const S_EXPORT_AS: u8 = 11;

pub(super) const S_EXPORT_AS_NS: u8 = 12;

pub(super) const S_IMPORT_NAME: u8 = 13;

pub(super) const S_DECLARE_MODULE: u8 = 14;

// For-head state (state).
pub(super) const F_START: u8 = 0;

pub(super) const F_BOUND: u8 = 1;

pub(super) const F_EXPR: u8 = 2;

pub(super) const F_ITER: u8 = 3;

// Member state on Object / ClassBody / TypeLit (state).
pub(super) const M_KEY_POS: u8 = 0;

pub(super) const M_KEY_SEEN: u8 = 1;

pub(super) const M_VALUE: u8 = 2;

// Member modifier bits (mods) on Object / ClassBody.
pub(super) const MOD_ASYNC: u8 = 1;

pub(super) const MOD_GEN: u8 = 2;

pub(super) const MOD_STATIC: u8 = 4;

// TypeRegion end rule (state).
pub(super) const R_ASSERT: u8 = 1; // `<T>x`: ends at its closing `>`
pub(super) const R_ARROW_RET: u8 = 2; // `(a): T =>`: ends at `=>`
pub(super) const R_INLINE: u8 = 3; // declarator/param/member annotation: ends at `=`/`,`/closer/`{`
pub(super) const R_STMT: u8 = 4; // alias / import-equals / bodiless module: ends at `;`/ASI
pub(super) const R_EXPR: u8 = 5; // `as T` / `satisfies T`: ends at any expression token
pub(super) const R_INTERFACE: u8 = 6; // `interface X ... { }`: ends after its body

// What an Angle list is (state).
pub(super) const A_DECL_PARAMS: u8 = 1; // type parameters of a declaration head or member
pub(super) const A_EXPR_ARGS: u8 = 2; // type arguments on an expression: `f<T>(x)`
pub(super) const A_IN_TYPE: u8 = 3; // a list inside a type
pub(super) const A_ASSERT: u8 = 4; // `<T>x` assertion or `<T,>() =>` generic arrow

// ClassHead heritage state (state), and its interface marker (reg).
pub(super) const C_EXTENDS: u8 = 1;
pub(super) const C_IMPLEMENTS: u8 = 2;
pub(super) const C_INTERFACE: u8 = 1;

// TypeLit (state): the body of an interface, which ends the statement when closed.
pub(super) const L_INTERFACE_BODY: u8 = 1;

#[derive(Clone, Copy)]
pub(super) struct Frame {
    pub(super) kind: FrameKind,
    pub(super) is_generator: bool,
    pub(super) is_async: bool,
    pub(super) strict: bool,
    pub(super) reserved: bool,
    /// Braces: closing this frame ends a value.
    pub(super) is_value: bool,
    /// Type frames: part of a declaration type (vs embedded in an expression).
    pub(super) decl: bool,
    /// TypeRegion: the last token completed a type.
    pub(super) atom: bool,
    /// TypeRegion: the last token closed a ( opened inside the region.
    pub(super) inner: bool,
    pub(super) state: u8,
    pub(super) reg: u8,
    /// Member modifier bits (Object / ClassBody).
    pub(super) mods: u8,
    /// For-head: the binding came from a declaration (for (let x of).
    pub(super) decl_binding: bool,
    pub(super) head: u8,
    pub(super) open_questions: u16,
    pub(super) prologue: u8,
}

impl Frame {
    /// Field initializers sit outside the enclosing yield / await context, as in tsc.
    pub(super) fn field_init(&self) -> bool {
        self.kind == FrameKind::ClassBody && self.state == M_VALUE
    }

    pub(super) fn child(&self, kind: FrameKind) -> Frame {
        let init = self.field_init();
        Frame {
            kind,
            is_generator: self.is_generator && !init,
            is_async: self.is_async && !init,
            strict: self.strict,
            reserved: self.reserved && !init,
            is_value: false,
            decl: false,
            atom: false,
            inner: false,
            state: 0,
            reg: 0,
            mods: 0,
            decl_binding: false,
            head: 0,
            open_questions: 0,
            prologue: 0,
        }
    }
}

pub(super) fn is_stmt_holder(k: FrameKind) -> bool {
    matches!(
        k,
        FrameKind::Root
            | FrameKind::Block
            | FrameKind::FnBody
            | FrameKind::ArrowBody
            | FrameKind::StaticBlock
    )
}
