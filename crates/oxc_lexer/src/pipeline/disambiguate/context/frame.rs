//! What the walk remembers per frame: the kind of every open bracket or virtual frame,
//! its per-kind state, and the keyword codes the walk reads off the source.

use crate::token::matches_tk;

/// A keyword that is a whole type by itself (`any`, `null`, `this`, ...): it takes no type
/// arguments, so a `<` after it is a comparison.
#[rustfmt::skip::macros(matches_tk)]
pub(super) fn keyword_type(kw: u8) -> bool {
    matches_tk!(
        kw,
        KwAny | KwBigInt | KwBoolean | KwNever | KwNumber | KwObject | KwString | KwSymbol
        | KwUndefined | KwUnknown | KwVoid | KwNull | KwThis | KwTrue | KwFalse
    )
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(super) enum FrameKind {
    #[default]
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
    // A template substitution (`${` .. `}`), opened by a TemplateHead or TemplateMiddle and closed
    // by the next Middle/Tail.
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

// Declarator state on statement frames and for-heads (`state`).
pub(super) const D_NONE: u8 = 0;

pub(super) const D_BINDING: u8 = 1;

pub(super) const D_BOUND: u8 = 2;

pub(super) const D_INIT: u8 = 3;

// Statement register on statement frames (`reg`).
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

// For-head state (`state`).
pub(super) const F_START: u8 = 0;

pub(super) const F_BOUND: u8 = 1;

pub(super) const F_EXPR: u8 = 2;

pub(super) const F_ITER: u8 = 3;

// Member state on Object / ClassBody / TypeLit (`state`).
pub(super) const M_KEY_POS: u8 = 0;

pub(super) const M_KEY_SEEN: u8 = 1;

pub(super) const M_VALUE: u8 = 2;

// Member modifier bits (`mods`) on Object / ClassBody.
pub(super) const MOD_ASYNC: u8 = 1;

pub(super) const MOD_GEN: u8 = 2;

pub(super) const MOD_STATIC: u8 = 4;

// TypeRegion end rule (`state`).
pub(super) const R_ASSERT: u8 = 1; // `<T>x`: ends at its closing `>`
pub(super) const R_ARROW_RET: u8 = 2; // `(a): T =>`: ends at `=>`
pub(super) const R_INLINE: u8 = 3; // declarator/param/member annotation: ends at `=`/`,`/closer/`{`
pub(super) const R_STMT: u8 = 4; // alias / import-equals / bodiless module: ends at `;`/ASI
pub(super) const R_EXPR: u8 = 5; // `as T` / `satisfies T`: ends at any expression token
pub(super) const R_INTERFACE: u8 = 6; // `interface X ... { }`: ends after its body

// What an Angle list is (`state`).
pub(super) const A_DECL_PARAMS: u8 = 1; // type parameters of a declaration head or member
pub(super) const A_EXPR_ARGS: u8 = 2; // type arguments on an expression: `f<T>(x)`
pub(super) const A_IN_TYPE: u8 = 3; // a list inside a type
pub(super) const A_ASSERT: u8 = 4; // `<T>x` assertion or `<T,>() =>` generic arrow

// ClassHead heritage state (`state`), and its interface marker (`reg`).
pub(super) const C_EXTENDS: u8 = 1;
pub(super) const C_IMPLEMENTS: u8 = 2;
pub(super) const C_INTERFACE: u8 = 1;

// TypeLit (`state`): the body of an interface, which ends the statement when closed.
pub(super) const L_INTERFACE_BODY: u8 = 1;

#[derive(Clone, Copy, Default)]
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
    /// TypeRegion: the last token closed a `(` opened inside the region.
    pub(super) inner: bool,
    pub(super) state: u8,
    pub(super) reg: u8,
    /// Member modifier bits (Object / ClassBody).
    pub(super) mods: u8,
    pub(super) head: u8,
    pub(super) open_questions: u16,
    pub(super) prologue: bool,
}

impl Frame {
    /// A class field initializer (`x = ...` in a class body): parsed outside the `yield` and
    /// `await` contexts of whatever encloses the class, as tsc does, so both are identifiers in
    /// it. Computed keys still see the enclosing function.
    pub(super) fn field_init(&self) -> bool {
        self.kind == FrameKind::ClassBody && self.state == M_VALUE
    }

    pub(super) fn next_member(&mut self) {
        self.state = M_KEY_POS;
        self.mods = 0;
    }

    pub(super) fn child(&self, kind: FrameKind) -> Frame {
        let init = self.field_init();
        Frame {
            kind,
            is_generator: self.is_generator && !init,
            is_async: self.is_async && !init,
            strict: self.strict,
            reserved: self.reserved && !init,
            ..Frame::default()
        }
    }
}

impl FrameKind {
    pub(super) fn is_virtual(self) -> bool {
        matches!(
            self,
            FrameKind::Concise
                | FrameKind::TypeRegion
                | FrameKind::Angle
                | FrameKind::FnHead
                | FrameKind::ClassHead
        )
    }

    /// Frames that hold statements (and so declarator state and statement registers).
    pub(super) fn is_stmt_holder(self) -> bool {
        matches!(
            self,
            FrameKind::Root
                | FrameKind::Block
                | FrameKind::FnBody
                | FrameKind::ArrowBody
                | FrameKind::StaticBlock
        )
    }

    pub(super) fn is_type_group(self) -> bool {
        matches!(
            self,
            FrameKind::Angle | FrameKind::TypeParen | FrameKind::TypeBracket | FrameKind::TypeLit
        )
    }

    pub(super) fn closer(self) -> u8 {
        match self {
            FrameKind::Block
            | FrameKind::FnBody
            | FrameKind::ArrowBody
            | FrameKind::ClassBody
            | FrameKind::StaticBlock
            | FrameKind::Object
            | FrameKind::Pattern
            | FrameKind::TypeLit
            | FrameKind::EnumBody
            | FrameKind::ModuleSpec
            | FrameKind::Container => b'}',
            FrameKind::Head
            | FrameKind::Params
            | FrameKind::Call
            | FrameKind::Group
            | FrameKind::TypeParen => b')',
            FrameKind::Index
            | FrameKind::Array
            | FrameKind::ComputedKey
            | FrameKind::TypeBracket
            | FrameKind::ArrayPattern => b']',
            _ => 0,
        }
    }
}
