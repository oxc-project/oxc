//! What the walk remembers per frame: the kind of every open bracket or virtual frame,
//! its per-kind state, and the keyword codes the walk reads off the source.

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(super) enum FrameKind {
    #[default]
    Root,
    // Braces.
    Block,
    FnBody,
    ArrowBody,
    ClassBody,
    Object,
    TypeLit,
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
    Array,
    ComputedKey,
    TypeBracket,
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

// Declarator state on statement frames and for-heads (`state`).
pub(super) const D_NONE: u8 = 0;

pub(super) const D_BINDING: u8 = 1;

pub(super) const D_BOUND: u8 = 2;

pub(super) const D_INIT: u8 = 3;

// Statement register on statement frames (`reg`).
pub(super) const S_NONE: u8 = 0;

pub(super) const S_CASE: u8 = 1;

pub(super) const S_IMPORT: u8 = 3;

pub(super) const S_EXPORT: u8 = 4;

pub(super) const S_TYPE_NAME: u8 = 6;

pub(super) const S_BREAK: u8 = 7;

pub(super) const S_EXPORT_AS: u8 = 11;

pub(super) const S_DECLARE_MODULE: u8 = 14;

pub(super) const S_ATTRS: u8 = 15;

// Head state: a for head before its first semicolon, where of after a value is the keyword.
pub(super) const F_OF: u8 = 0;

pub(super) const F_NO_OF: u8 = 1;

// Member state on Object / ClassBody / TypeLit (`state`).
pub(super) const M_KEY_POS: u8 = 0;

pub(super) const M_KEY_SEEN: u8 = 1;

pub(super) const M_VALUE: u8 = 2;

// Member modifier bits (`mods`) on Object / ClassBody.
pub(super) const MOD_ASYNC: u8 = 1;

pub(super) const MOD_GEN: u8 = 2;

pub(super) const MOD_STATIC: u8 = 4;

// TypeRegion end rule (`state`).
pub(super) const R_ARROW_RET: u8 = 2; // `(a): T =>`: ends at `=>`
pub(super) const R_INLINE: u8 = 3; // declarator/param/member annotation: ends at `=`/`,`/closer/`{`
pub(super) const R_STMT: u8 = 4; // alias / import-equals / bodiless module: ends at `;`/ASI
pub(super) const R_EXPR: u8 = 5; // `as T` / `satisfies T`: ends at any expression token

// What an Angle list is (`state`).
pub(super) const A_VALUE: u8 = 1; // declaration type parameters or `f<T>(x)` type arguments
pub(super) const A_IN_TYPE: u8 = 3; // a list inside a type
pub(super) const A_ASSERT: u8 = 4; // `<T>x` assertion or `<T,>() =>` generic arrow

// ClassHead: an interface head (reg).
pub(super) const C_INTERFACE: u8 = 1;

// TypeLit (`state`): the body of an interface, which ends the statement when closed.
pub(super) const L_INTERFACE_BODY: u8 = 1;

#[derive(Clone, Copy, Default)]
pub(super) struct Frame {
    pub(super) kind: FrameKind,
    pub(super) is_generator: bool,
    pub(super) is_async: bool,
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
    /// Modifier bits: an Object or ClassBody member's, or the async before a Group.
    pub(super) mods: u8,
    pub(super) open_questions: u16,
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
            FrameKind::Root | FrameKind::Block | FrameKind::FnBody | FrameKind::ArrowBody
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
            | FrameKind::Object
            | FrameKind::TypeLit
            | FrameKind::ModuleSpec
            | FrameKind::Container => b'}',
            FrameKind::Head
            | FrameKind::Params
            | FrameKind::Call
            | FrameKind::Group
            | FrameKind::TypeParen => b')',
            FrameKind::Array | FrameKind::ComputedKey | FrameKind::TypeBracket => b']',
            _ => 0,
        }
    }
}
