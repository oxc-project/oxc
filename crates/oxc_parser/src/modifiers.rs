use bitflags::bitflags;

use oxc_allocator::Vec;
use oxc_ast::ast::TSAccessibility;
use oxc_diagnostics::Result;
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
pub struct Modifiers<'a>(Option<Vec<'a, Modifier>>);

impl<'a> Modifiers<'a> {
    pub fn new(modifiers: Vec<'a, Modifier>) -> Self {
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
    pub(crate) fn eat_modifiers_before_declaration(&mut self) -> (ModifierFlags, Modifiers<'a>) {
        let mut flags = ModifierFlags::empty();
        let mut modifiers = self.ast.new_vec();
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
                    Kind::Default => self.next_token_can_follow_default_keyword(),
                    Kind::Type => {
                        self.bump_any();
                        self.can_follow_export_modifier()
                    }
                    _ => self.can_follow_modifier(),
                }
            }
            Kind::Default => self.next_token_can_follow_default_keyword(),
            Kind::Accessor | Kind::Static | Kind::Get | Kind::Set => {
                // These modifiers can cross line.
                self.bump_any();
                self.can_follow_modifier()
            }
            // Rest modifiers cannot cross line
            _ => {
                self.bump_any();
                self.can_follow_modifier() && !self.cur_token().is_on_new_line
            }
        }
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

    pub(crate) fn parse_modifiers(
        &mut self,
        _allow_decorators: bool,
        permit_const_as_modifier: bool,
        stop_on_start_of_class_static_block: bool,
    ) -> Modifiers<'a> {
        let mut has_seen_static_modifier = false;
        // let mut has_leading_modifier = false;
        // let mut has_trailing_decorator = false;
        let mut modifiers = self.ast.new_vec();

        // parse leading decorators
        // if (allowDecorators && token() === SyntaxKind.AtToken) {
        // while (decorator = tryParseDecorator()) {
        // list = append(list, decorator);
        // }
        // }

        // parse leading modifiers
        while let Some(modifier) = self.try_parse_modifier(
            has_seen_static_modifier,
            permit_const_as_modifier,
            stop_on_start_of_class_static_block,
        ) {
            if modifier.kind == ModifierKind::Static {
                has_seen_static_modifier = true;
            }
            modifiers.push(modifier);
        }

        // parse trailing decorators, but only if we parsed any leading modifiers
        // if (hasLeadingModifier && allowDecorators && token() === SyntaxKind.AtToken) {
        // while (decorator = tryParseDecorator()) {
        // list = append(list, decorator);
        // hasTrailingDecorator = true;
        // }
        // }

        // parse trailing modifiers, but only if we parsed any trailing decorators
        // if (hasTrailingDecorator) {
        // while (modifier = tryParseModifier(hasSeenStaticModifier, permitConstAsModifier, stopOnStartOfClassStaticBlock)) {
        // if (modifier.kind === SyntaxKind.StaticKeyword) hasSeenStaticModifier = true;
        // list = append(list, modifier);
        // }
        // }

        Modifiers::new(modifiers)
    }

    fn try_parse_modifier(
        &mut self,
        _has_seen_static_modifier: bool,
        _permit_const_as_modifier: bool,
        _stop_on_start_of_class_static_block: bool,
    ) -> Option<Modifier> {
        let span = self.start_span();
        let kind = self.cur_kind();
        // if (token() === SyntaxKind.ConstKeyword && permitConstAsModifier) {
        // We need to ensure that any subsequent modifiers appear on the same line
        // so that when 'const' is a standalone declaration, we don't issue an error.
        // if (!tryParse(nextTokenIsOnSameLineAndCanFollowModifier)) {
        // return undefined;
        // }
        // }
        // else if (stopOnStartOfClassStaticBlock && token() === SyntaxKind.StaticKeyword && lookAhead(nextTokenIsOpenBrace)) {
        // return undefined;
        // }
        // else if (hasSeenStaticModifier && token() === SyntaxKind.StaticKeyword) {
        // return undefined;
        // }
        // else {
        if !self.parse_any_contextual_modifier() {
            return None;
        }
        // }
        Some(Self::modifier(kind, self.end_span(span)))
    }

    fn parse_any_contextual_modifier(&mut self) -> bool {
        self.cur_kind().is_modifier_kind()
            && self.try_parse(Self::next_token_can_follow_modifier).is_some()
    }

    fn next_token_can_follow_modifier(&mut self) -> Result<()> {
        let b = match self.cur_kind() {
            Kind::Const => self.peek_at(Kind::Enum),
            Kind::Export => {
                self.bump_any();
                match self.cur_kind() {
                    Kind::Default => self.lookahead(Self::next_token_can_follow_default_keyword),
                    Kind::Type => self.lookahead(Self::next_token_can_follow_export_modifier),
                    _ => self.can_follow_export_modifier(),
                }
            }
            Kind::Default => self.next_token_can_follow_default_keyword(),
            Kind::Static | Kind::Get | Kind::Set => {
                self.bump_any();
                self.can_follow_modifier()
            }
            _ => self.next_token_is_on_same_line_and_can_follow_modifier(),
        };
        if b {
            Ok(())
        } else {
            Err(self.unexpected())
        }
    }

    fn next_token_is_on_same_line_and_can_follow_modifier(&mut self) -> bool {
        self.bump_any();
        if self.cur_token().is_on_new_line {
            return false;
        }
        self.can_follow_modifier()
    }

    fn next_token_can_follow_default_keyword(&mut self) -> bool {
        self.bump_any();
        match self.cur_kind() {
            Kind::Class | Kind::Function | Kind::Interface | Kind::At => true,
            Kind::Abstract if self.lookahead(Self::next_token_is_class_keyword_on_same_line) => {
                true
            }
            Kind::Async if self.lookahead(Self::next_token_is_function_keyword_on_same_line) => {
                true
            }
            _ => false,
        }
    }

    fn next_token_can_follow_export_modifier(&mut self) -> bool {
        self.bump_any();
        self.can_follow_export_modifier()
    }

    fn can_follow_export_modifier(&mut self) -> bool {
        let kind = self.cur_kind();
        kind == Kind::At
            && kind != Kind::Star
            && kind != Kind::As
            && kind != Kind::LCurly
            && self.can_follow_modifier()
    }

    fn can_follow_modifier(&mut self) -> bool {
        match self.cur_kind() {
            Kind::LBrack | Kind::LCurly | Kind::Star | Kind::Dot3 => true,
            kind => kind.is_literal_property_name(),
        }
    }

    fn next_token_is_class_keyword_on_same_line(&mut self) -> bool {
        self.bump_any();
        self.cur_kind() == Kind::Class && !self.cur_token().is_on_new_line
    }

    fn next_token_is_function_keyword_on_same_line(&mut self) -> bool {
        self.bump_any();
        self.cur_kind() == Kind::Function && !self.cur_token().is_on_new_line
    }
}
