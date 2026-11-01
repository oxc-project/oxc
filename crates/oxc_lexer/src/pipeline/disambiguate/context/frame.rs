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

/// Declarator state on statement frames.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum Declarator {
    #[default]
    None,
    Binding,
    Bound,
    Init,
}

/// Statement register on statement frames.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum StmtReg {
    #[default]
    None,
    Case,
    Import,
    Export,
    TypeName,
    Break,
    ExportAs,
    DeclareModule,
    Attrs,
}

/// Member state on Object and ClassBody.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum Member {
    #[default]
    KeyPos,
    KeySeen,
    Value,
}

// Member modifier bits (`mods`) on Object / ClassBody.
pub(super) const MOD_ASYNC: u8 = 1;

pub(super) const MOD_GEN: u8 = 2;

pub(super) const MOD_STATIC: u8 = 4;

/// TypeRegion end rule.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum RegionRule {
    /// A declarator, param or member annotation: ends at an equals, comma, closer or brace.
    #[default]
    Inline,
    /// The return type of an arrow: ends at the arrow.
    ArrowRet,
    /// An alias, import equals or bodiless module: ends at a semicolon or ASI.
    Stmt,
    /// An as or satisfies type: ends at any expression token.
    Expr,
}

/// What an Angle list is.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum AngleList {
    /// Declaration type parameters, or the type arguments of a call.
    #[default]
    Value,
    /// A list inside a type.
    InType,
    /// An operand follows: an assertion or leading type parameters.
    Assert,
}

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
    /// ClassHead: an interface head. TypeLit: an interface body, closing it ends the statement.
    pub(super) is_interface: bool,
    pub(super) region: RegionRule,
    pub(super) angle: AngleList,
    pub(super) declarator: Declarator,
    pub(super) member: Member,
    /// Head: of after a value is not the keyword, as it is in a for head.
    pub(super) of_closed: bool,
    /// JsxTag: open angle brackets of its type arguments.
    pub(super) jsx_depth: u8,
    pub(super) reg: StmtReg,
    /// Modifier bits: an Object or ClassBody member's, or the async before a Group.
    pub(super) mods: u8,
    pub(super) open_questions: u16,
}

impl Frame {
    /// A class field initializer (`x = ...` in a class body): parsed outside the `yield` and
    /// `await` contexts of whatever encloses the class, as tsc does, so both are identifiers in
    /// it. Computed keys still see the enclosing function.
    pub(super) fn field_init(&self) -> bool {
        self.kind == FrameKind::ClassBody && self.member == Member::Value
    }

    pub(super) fn next_member(&mut self) {
        self.member = Member::KeyPos;
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
