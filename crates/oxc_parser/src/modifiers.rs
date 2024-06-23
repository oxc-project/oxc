use bitflags::bitflags;

use oxc_ast::ast::TSAccessibility;
use oxc_span::Span;

use crate::{lexer::Kind, ParserImpl};

bitflags! {
  /// Bitflag of modifiers and contextual modifiers.
  /// Useful to cheaply track all already seen modifiers of a member (instead of using a HashSet<ModifierKind>).
  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  pub struct ModifierFlags: u16 {
      const DECLARE       = 1 << 0;
      const PRIVATE       = 1 << 1;
      const PROTECTED     = 1 << 2;
      const PUBLIC        = 1 << 3;
      const STATIC        = 1 << 4;
      const READONLY      = 1 << 5;
      const ABSTRACT      = 1 << 6;
      const OVERRIDE      = 1 << 7;
      const ASYNC         = 1 << 8;
      const CONST         = 1 << 9;
      const IN            = 1 << 10;
      const OUT           = 1 << 11;
      const EXPORT        = 1 << 12;
      const DEFAULT       = 1 << 13;
      const ACCESSOR      = 1 << 14;
      const ACCESSIBILITY = Self::PRIVATE.bits() | Self::PROTECTED.bits() | Self::PUBLIC.bits();
  }
}

/// It is the caller's safety to always check by `Kind::is_modifier_kind`
/// before converting [`Kind`] to [`ModifierFlags`] so that we can assume here that
/// the conversion always succeeds.
impl From<Kind> for ModifierFlags {
    fn from(value: Kind) -> Self {
        match value {
            Kind::Abstract => Self::ABSTRACT,
            Kind::Declare => Self::DECLARE,
            Kind::Private => Self::PRIVATE,
            Kind::Protected => Self::PROTECTED,
            Kind::Public => Self::PUBLIC,
            Kind::Static => Self::STATIC,
            Kind::Readonly => Self::READONLY,
            Kind::Override => Self::OVERRIDE,
            Kind::Async => Self::ASYNC,
            Kind::Const => Self::CONST,
            Kind::In => Self::IN,
            Kind::Out => Self::OUT,
            Kind::Export => Self::EXPORT,
            Kind::Default => Self::DEFAULT,
            Kind::Accessor => Self::ACCESSOR,
            _ => unreachable!(),
        }
    }
}

impl ModifierFlags {
    pub(crate) fn accessibility(self) -> Option<TSAccessibility> {
        if self.contains(Self::PUBLIC) {
            return Some(TSAccessibility::Public);
        }
        if self.contains(Self::PROTECTED) {
            return Some(TSAccessibility::Protected);
        }

        if self.contains(Self::PRIVATE) {
            return Some(TSAccessibility::Private);
        }
        None
    }

    pub(crate) fn readonly(self) -> bool {
        self.contains(Self::READONLY)
    }

    pub(crate) fn declare(self) -> bool {
        self.contains(Self::DECLARE)
    }

    pub(crate) fn r#async(self) -> bool {
        self.contains(Self::ASYNC)
    }

    pub(crate) fn r#override(self) -> bool {
        self.contains(Self::OVERRIDE)
    }

    pub(crate) fn r#abstract(self) -> bool {
        self.contains(Self::ABSTRACT)
    }

    pub(crate) fn r#static(self) -> bool {
        self.contains(Self::STATIC)
    }
}

#[derive(Debug, Hash)]
pub struct Modifier {
    pub span: Span,
    pub kind: ModifierKind,
}

#[derive(Debug, Default, Hash)]
pub struct Modifiers(Option<Vec<Modifier>>);

impl Modifiers {
    pub fn new(modifiers: Vec<Modifier>) -> Self {
        Self(Some(modifiers))
    }

    pub fn empty() -> Self {
        Self(None)
    }

    pub fn contains(&self, target: ModifierKind) -> bool {
        self.0
            .as_ref()
            .map_or(false, |modifiers| modifiers.iter().any(|modifier| modifier.kind == target))
    }

    pub fn iter(&self) -> impl Iterator<Item = &Modifier> + '_ {
        self.0.as_ref().into_iter().flat_map(|modifiers| modifiers.iter())
    }

    pub fn is_contains_const(&self) -> bool {
        self.contains(ModifierKind::Const)
    }

    pub fn is_contains_declare(&self) -> bool {
        self.contains(ModifierKind::Declare)
    }

    pub fn is_contains_abstract(&self) -> bool {
        self.contains(ModifierKind::Abstract)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModifierKind {
    Abstract,
    Accessor,
    Async,
    Const,
    Declare,
    Default,
    Export,
    In,
    Public,
    Private,
    Protected,
    Readonly,
    Static,
    Out,
    Override,
}

impl ModifierKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Abstract => "abstract",
            Self::Accessor => "accessor",
            Self::Async => "async",
            Self::Const => "const",
            Self::Declare => "declare",
            Self::Default => "default",
            Self::Export => "export",
            Self::In => "in",
            Self::Public => "public",
            Self::Private => "private",
            Self::Protected => "protected",
            Self::Readonly => "readonly",
            Self::Static => "static",
            Self::Out => "out",
            Self::Override => "override",
        }
    }
}

impl<'a> ParserImpl<'a> {
    pub(crate) fn eat_modifiers_before_declaration(&mut self) -> (ModifierFlags, Modifiers) {
        let mut flags = ModifierFlags::empty();
        let mut modifiers = vec![];
        while self.at_modifier() {
            let span = self.start_span();
            let modifier_flag = self.cur_kind().into();
            flags.set(modifier_flag, true);
            let kind = self.cur_kind();
            self.bump_any();
            modifiers.push(Self::modifier(kind, self.end_span(span)));
        }
        (flags, Modifiers::new(modifiers))
    }

    fn at_modifier(&mut self) -> bool {
        self.lookahead(Self::at_modifier_worker)
    }

    fn at_modifier_worker(&mut self) -> bool {
        if !self.cur_kind().is_modifier_kind() {
            return false;
        }

        match self.cur_kind() {
            Kind::Const => !self.peek_token().is_on_new_line && self.peek_kind() == Kind::Enum,
            Kind::Export => {
                self.bump_any();
                match self.cur_kind() {
                    Kind::Default => {
                        self.bump_any();
                        self.can_follow_default()
                    }
                    Kind::Type => {
                        self.bump_any();
                        self.can_follow_export()
                    }
                    _ => self.can_follow_export(),
                }
            }
            Kind::Default => {
                self.bump_any();
                self.can_follow_default()
            }
            Kind::Accessor | Kind::Static | Kind::Get | Kind::Set => {
                // These modifiers can cross line.
                self.bump_any();
                Self::can_follow_modifier(self.cur_kind())
            }
            // Rest modifiers cannot cross line
            _ => {
                self.bump_any();
                Self::can_follow_modifier(self.cur_kind()) && !self.cur_token().is_on_new_line
            }
        }
    }

    fn can_follow_default(&mut self) -> bool {
        let at_declaration =
            matches!(self.cur_kind(), Kind::Class | Kind::Function | Kind::Interface);
        let at_abstract_declaration = self.at(Kind::Abstract)
            && self.peek_at(Kind::Class)
            && !self.peek_token().is_on_new_line;
        let at_async_function = self.at(Kind::Async)
            && self.peek_at(Kind::Function)
            && !self.peek_token().is_on_new_line;
        at_declaration | at_abstract_declaration | at_async_function
    }

    fn can_follow_export(&mut self) -> bool {
        // Note that the `export` in export assignment is not a modifier
        // and are handled explicitly in the parser.
        !matches!(self.cur_kind(), Kind::Star | Kind::As | Kind::LCurly)
            && Self::can_follow_modifier(self.cur_kind())
    }

    fn can_follow_modifier(kind: Kind) -> bool {
        kind.is_literal_property_name()
            || matches!(kind, Kind::LCurly | Kind::LBrack | Kind::Star | Kind::Dot3)
    }

    fn modifier(kind: Kind, span: Span) -> Modifier {
        let modifier_kind = match kind {
            Kind::Abstract => ModifierKind::Abstract,
            Kind::Declare => ModifierKind::Declare,
            Kind::Private => ModifierKind::Private,
            Kind::Protected => ModifierKind::Protected,
            Kind::Public => ModifierKind::Public,
            Kind::Static => ModifierKind::Static,
            Kind::Readonly => ModifierKind::Readonly,
            Kind::Override => ModifierKind::Override,
            Kind::Async => ModifierKind::Async,
            Kind::Const => ModifierKind::Const,
            Kind::In => ModifierKind::In,
            Kind::Out => ModifierKind::Out,
            Kind::Export => ModifierKind::Export,
            Kind::Default => ModifierKind::Default,
            Kind::Accessor => ModifierKind::Accessor,
            _ => unreachable!(),
        };
        Modifier { span, kind: modifier_kind }
    }
}
