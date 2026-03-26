use std::{
    fmt::{self, Debug, Display},
    iter, mem,
    num::NonZeroU16,
};

use oxc_allocator::Vec;
use oxc_ast::ast::TSAccessibility;
use oxc_data_structures::fieldless_enum;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

use crate::{ParserConfig as Config, ParserImpl, diagnostics, lexer::Kind};

/// A set of modifier kinds, stored as a bitfield.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ModifierKinds(u16);

impl ModifierKinds {
    /// Create a set from an array of modifier kinds.
    #[inline]
    pub const fn new<const N: usize>(kinds: [ModifierKind; N]) -> Self {
        let mut out = Self::none();
        let mut i = 0;
        while i < N {
            out = out.with(kinds[i]);
            i += 1;
        }
        out
    }

    /// Create a set containing all modifier kinds EXCEPT the ones listed.
    #[inline]
    pub const fn all_except<const N: usize>(kinds: [ModifierKind; N]) -> Self {
        const ALL: ModifierKinds = ModifierKinds::new(ModifierKind::VARIANTS);
        Self(Self::new(kinds).0 ^ ALL.0)
    }

    /// Empty set (no modifiers).
    #[inline]
    pub const fn none() -> Self {
        Self(0)
    }

    /// Check if `kind` is present in this set.
    #[inline]
    pub const fn contains(self, kind: ModifierKind) -> bool {
        self.0 & (1 << (kind as u8)) != 0
    }

    /// Check if this set has any overlap with `other`.
    #[inline]
    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    /// Check if this set contains any kinds not present in `other`.
    #[inline]
    pub const fn has_any_not_in(self, other: Self) -> bool {
        self.0 & !other.0 != 0
    }

    /// Return a new set with `kind` added.
    #[inline]
    pub const fn with(self, kind: ModifierKind) -> Self {
        Self(self.0 | (1 << (kind as u8)))
    }

    /// Return a new set with `kind` removed.
    #[inline]
    pub const fn without(self, kind: ModifierKind) -> Self {
        Self(self.0 & !(1 << (kind as u8)))
    }

    /// Count how many [`ModifierKind`]s are in this set.
    #[inline]
    pub const fn count(self) -> usize {
        self.0.count_ones() as usize
    }

    /// Iterate over all present [`ModifierKind`]s.
    pub fn iter(self) -> impl Iterator<Item = ModifierKind> {
        let mut remaining = self.0;
        iter::from_fn(move || {
            // Exit if there are no more bits set
            let bits = NonZeroU16::new(remaining)?;
            // Get the index of the next set bit
            let bit = bits.trailing_zeros();
            // Unset the bit
            remaining &= remaining - 1;

            // SAFETY: All other methods ensure that only bits for valid `ModifierKind`s are set
            let kind = unsafe { ModifierKind::from_usize_unchecked(bit as usize) };
            Some(kind)
        })
    }

    fn accessibility(self) -> Option<TSAccessibility> {
        if self.contains(ModifierKind::Public) {
            return Some(TSAccessibility::Public);
        }
        if self.contains(ModifierKind::Protected) {
            return Some(TSAccessibility::Protected);
        }
        if self.contains(ModifierKind::Private) {
            return Some(TSAccessibility::Private);
        }
        None
    }
}

impl Display for ModifierKinds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, kind) in self.iter().enumerate() {
            if i != 0 {
                f.write_str(", ")?;
            }
            f.write_str(kind.as_str())?;
        }
        Ok(())
    }
}

impl Debug for ModifierKinds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

#[derive(Debug)]
pub struct Modifier {
    pub span: Span,
    pub kind: ModifierKind,
}

impl Modifier {
    pub fn new(span: Span, kind: ModifierKind) -> Self {
        Self { span, kind }
    }
}

/// Symbol modifiers. Primarily used in TypeScript code, but some are also used
/// in JavaScript.
///
/// ```ts
/// class Foo {
///     public readonly x: number
/// //  ^^^^^^ ^^^^^^^^
/// // these are modifiers
/// }
/// export const f = new foo()
/// // ^^^ This also counts as a modifier, but is also recorded separately as a
/// // named export declaration
/// ```
#[derive(Debug)]
pub struct Modifiers<'a> {
    /// May contain duplicates.
    modifiers: Option<Vec<'a, Modifier>>,
    /// Bitflag representation of modifier kinds stored in [`Self::modifiers`].
    /// Pre-computed to save CPU cycles on [`Self::contains`] checks (`O(1)`
    /// bitflag intersection vs `O(n)` linear search).
    kinds: ModifierKinds,
}

impl Default for Modifiers<'_> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<'a> Modifiers<'a> {
    /// Create a new set of modifiers
    ///
    /// # Invariants
    /// `kinds` must correctly reflect the [`ModifierKind`]s within
    ///  `modifiers`. e.g., if `modifiers` is empty, then so is `kinds`.
    #[must_use]
    pub(crate) fn new(modifiers: Option<Vec<'a, Modifier>>, kinds: ModifierKinds) -> Self {
        // Debug check that `modifiers` and `kinds` are consistent with each other
        #[cfg(debug_assertions)]
        {
            if let Some(modifiers) = &modifiers {
                assert!(!modifiers.is_empty());

                let mut found_kinds = ModifierKinds::none();
                for modifier in modifiers {
                    found_kinds = found_kinds.with(modifier.kind);
                }
                assert_eq!(found_kinds, kinds);
            } else {
                assert_eq!(kinds, ModifierKinds::none());
            }
        }

        Self { modifiers, kinds }
    }

    pub fn empty() -> Self {
        Self { modifiers: None, kinds: ModifierKinds::none() }
    }

    pub fn contains(&self, target: ModifierKind) -> bool {
        self.kinds.contains(target)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Modifier> + '_ {
        self.modifiers.as_ref().into_iter().flat_map(|modifiers| modifiers.iter())
    }

    /// Look up a specific modifier by [`ModifierKind`].
    pub fn get(&self, kind: ModifierKind) -> Option<&Modifier> {
        if self.kinds.contains(kind) {
            let modifier = self.iter().find(|m| m.kind == kind);
            debug_assert!(modifier.is_some());
            modifier
        } else {
            None
        }
    }

    pub fn accessibility(&self) -> Option<TSAccessibility> {
        self.kinds.accessibility()
    }

    #[inline]
    pub fn contains_async(&self) -> bool {
        self.kinds.contains(ModifierKind::Async)
    }

    #[inline]
    pub fn contains_const(&self) -> bool {
        self.kinds.contains(ModifierKind::Const)
    }

    #[inline]
    pub fn contains_declare(&self) -> bool {
        self.kinds.contains(ModifierKind::Declare)
    }

    #[inline]
    pub fn contains_abstract(&self) -> bool {
        self.kinds.contains(ModifierKind::Abstract)
    }

    #[inline]
    pub fn contains_readonly(&self) -> bool {
        self.kinds.contains(ModifierKind::Readonly)
    }

    #[inline]
    pub fn contains_override(&self) -> bool {
        self.kinds.contains(ModifierKind::Override)
    }
}

// `fieldless_enum!` macro provides `ModifierKind::VARIANTS` constant listing all variants
fieldless_enum! {
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    #[repr(u8)]
    pub enum ModifierKind {
        Declare = 0,
        Private = 1,
        Protected = 2,
        Public = 3,
        Static = 4,
        Readonly = 5,
        Abstract = 6,
        Override = 7,
        Async = 8,
        Const = 9,
        In = 10,
        Out = 11,
        Default = 12,
        Accessor = 13,
        Export = 14,
    }
}

impl ModifierKind {
    /// Convert `usize` to [`ModifierKind`] without checks.
    ///
    /// # SAFETY
    /// `value` must be a valid discriminant for [`ModifierKind`].
    #[inline]
    unsafe fn from_usize_unchecked(value: usize) -> Self {
        debug_assert!(Self::VARIANTS.iter().any(|&kind| kind as usize == value));
        // SAFETY: Caller guarantees `value` is a valid discriminant for `ModifierKind`
        #[expect(clippy::cast_possible_truncation)]
        unsafe {
            mem::transmute::<u8, Self>(value as u8)
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Abstract => "abstract",
            Self::Accessor => "accessor",
            Self::Async => "async",
            Self::Const => "const",
            Self::Declare => "declare",
            Self::In => "in",
            Self::Public => "public",
            Self::Private => "private",
            Self::Protected => "protected",
            Self::Readonly => "readonly",
            Self::Static => "static",
            Self::Out => "out",
            Self::Override => "override",
            Self::Default => "default",
            Self::Export => "export",
        }
    }
}

impl TryFrom<Kind> for ModifierKind {
    type Error = ();

    fn try_from(kind: Kind) -> Result<Self, Self::Error> {
        match kind {
            Kind::Abstract => Ok(Self::Abstract),
            Kind::Declare => Ok(Self::Declare),
            Kind::Private => Ok(Self::Private),
            Kind::Protected => Ok(Self::Protected),
            Kind::Public => Ok(Self::Public),
            Kind::Static => Ok(Self::Static),
            Kind::Readonly => Ok(Self::Readonly),
            Kind::Override => Ok(Self::Override),
            Kind::Async => Ok(Self::Async),
            Kind::Const => Ok(Self::Const),
            Kind::In => Ok(Self::In),
            Kind::Out => Ok(Self::Out),
            Kind::Accessor => Ok(Self::Accessor),
            Kind::Default => Ok(Self::Default),
            Kind::Export => Ok(Self::Export),
            _ => Err(()),
        }
    }
}

impl Display for ModifierKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'a, C: Config> ParserImpl<'a, C> {
    pub(crate) fn eat_modifiers_before_declaration(&mut self) -> Modifiers<'a> {
        if !self.at_modifier() {
            return Modifiers::empty();
        }
        let mut kinds = ModifierKinds::none();
        let mut modifiers = self.ast.vec();
        while self.at_modifier() {
            let span = self.start_span();
            let kind = self.cur_kind();
            self.bump_any();
            let modifier = self.modifier(kind, self.end_span(span));
            self.check_modifier(kinds, &modifier);
            kinds = kinds.with(modifier.kind);
            modifiers.push(modifier);
        }
        Modifiers::new(Some(modifiers), kinds)
    }

    fn at_modifier(&mut self) -> bool {
        if !self.cur_kind().is_modifier_kind() {
            return false;
        }
        self.lookahead(Self::at_modifier_worker)
    }

    fn at_modifier_worker(&mut self) -> bool {
        match self.cur_kind() {
            Kind::Const => {
                self.bump_any();
                self.at(Kind::Enum)
            }
            Kind::Accessor | Kind::Static | Kind::Get | Kind::Set => {
                // These modifiers can cross line.
                self.bump_any();
                self.can_follow_modifier()
            }
            // Rest modifiers cannot cross line
            _ => {
                self.bump_any();
                self.can_follow_modifier() && !self.cur_token().is_on_new_line()
            }
        }
    }

    fn modifier(&mut self, kind: Kind, span: Span) -> Modifier {
        let modifier_kind = ModifierKind::try_from(kind).unwrap_or_else(|()| {
            self.set_unexpected();
            ModifierKind::Abstract // Dummy value
        });
        Modifier { span, kind: modifier_kind }
    }

    pub(crate) fn parse_modifiers(
        &mut self,
        permit_const_as_modifier: bool,
        stop_on_start_of_class_static_block: bool,
    ) -> Modifiers<'a> {
        let mut has_seen_static_modifier = false;

        let mut modifiers = None;
        let mut modifier_kinds = ModifierKinds::none();

        while let Some(modifier) = self.try_parse_modifier(
            has_seen_static_modifier,
            permit_const_as_modifier,
            stop_on_start_of_class_static_block,
        ) {
            if modifier.kind == ModifierKind::Static {
                has_seen_static_modifier = true;
            }
            self.check_modifier(modifier_kinds, &modifier);
            modifier_kinds = modifier_kinds.with(modifier.kind);
            modifiers.get_or_insert_with(|| self.ast.vec()).push(modifier);
        }

        Modifiers::new(modifiers, modifier_kinds)
    }

    fn try_parse_modifier(
        &mut self,
        has_seen_static_modifier: bool,
        permit_const_as_modifier: bool,
        stop_on_start_of_class_static_block: bool,
    ) -> Option<Modifier> {
        let span = self.start_span();
        let kind = self.cur_kind();

        if kind == Kind::Const {
            if !permit_const_as_modifier {
                return None;
            }

            // We need to ensure that any subsequent modifiers appear on the same line
            // so that when 'const' is a standalone declaration, we don't issue
            // an error.
            self.try_parse(Self::try_next_token_is_on_same_line_and_can_follow_modifier)?;
        } else if
        // we're at the start of a static block
        (stop_on_start_of_class_static_block
            && kind == Kind::Static
            && self.lexer.peek_token().kind() == Kind::LCurly)
            // we may be at the start of a static block
            || (has_seen_static_modifier && kind == Kind::Static)
            // next token is not a modifier
            || (!self.parse_any_contextual_modifier())
        {
            return None;
        }
        Some(self.modifier(kind, self.end_span(span)))
    }

    pub(crate) fn parse_contextual_modifier(&mut self, kind: Kind) -> bool {
        self.at(kind) && self.try_parse(Self::next_token_can_follow_modifier).is_some()
    }

    fn parse_any_contextual_modifier(&mut self) -> bool {
        self.cur_kind().is_modifier_kind()
            && self.try_parse(Self::next_token_can_follow_modifier).is_some()
    }

    pub(crate) fn next_token_can_follow_modifier(&mut self) {
        let b = match self.cur_kind() {
            Kind::Const => {
                self.bump_any();
                self.at(Kind::Enum)
            }
            Kind::Static => {
                self.bump_any();
                self.can_follow_modifier()
            }
            Kind::Get | Kind::Set => {
                self.bump_any();
                self.can_follow_get_or_set_keyword()
            }
            _ => self.next_token_is_on_same_line_and_can_follow_modifier(),
        };
        if !b {
            self.set_unexpected();
        }
    }

    fn try_next_token_is_on_same_line_and_can_follow_modifier(&mut self) {
        if !self.next_token_is_on_same_line_and_can_follow_modifier() {
            self.set_unexpected();
        }
    }

    fn next_token_is_on_same_line_and_can_follow_modifier(&mut self) -> bool {
        self.bump_any();
        if self.cur_token().is_on_new_line() {
            return false;
        }
        self.can_follow_modifier()
    }

    fn can_follow_modifier(&self) -> bool {
        match self.cur_kind() {
            Kind::PrivateIdentifier | Kind::LBrack | Kind::LCurly | Kind::Star | Kind::Dot3 => true,
            kind => kind.is_identifier_or_keyword(),
        }
    }

    fn can_follow_get_or_set_keyword(&self) -> bool {
        let kind = self.cur_kind();
        kind == Kind::LBrack || kind == Kind::PrivateIdentifier || kind.is_literal_property_name()
    }

    fn check_modifier(&mut self, kinds: ModifierKinds, modifier: &Modifier) {
        match modifier.kind {
            ModifierKind::Public | ModifierKind::Private | ModifierKind::Protected => {
                if kinds.intersects(ModifierKinds::new([
                    ModifierKind::Public,
                    ModifierKind::Private,
                    ModifierKind::Protected,
                ])) {
                    self.error(diagnostics::accessibility_modifier_already_seen(modifier));
                } else if kinds.contains(ModifierKind::Override) {
                    self.error(diagnostics::modifier_must_precede_other_modifier(
                        modifier,
                        ModifierKind::Override,
                    ));
                } else if kinds.contains(ModifierKind::Static) {
                    self.error(diagnostics::modifier_must_precede_other_modifier(
                        modifier,
                        ModifierKind::Static,
                    ));
                } else if kinds.contains(ModifierKind::Accessor) {
                    self.error(diagnostics::modifier_must_precede_other_modifier(
                        modifier,
                        ModifierKind::Accessor,
                    ));
                } else if kinds.contains(ModifierKind::Readonly) {
                    self.error(diagnostics::modifier_must_precede_other_modifier(
                        modifier,
                        ModifierKind::Readonly,
                    ));
                } else if kinds.contains(ModifierKind::Async) {
                    self.error(diagnostics::modifier_must_precede_other_modifier(
                        modifier,
                        ModifierKind::Async,
                    ));
                } else if kinds.contains(ModifierKind::Abstract) {
                    self.error(diagnostics::modifier_must_precede_other_modifier(
                        modifier,
                        ModifierKind::Abstract,
                    ));
                }
            }
            ModifierKind::Static => {
                if kinds.contains(ModifierKind::Static) {
                    self.error(diagnostics::modifier_already_seen(modifier));
                } else if kinds.contains(ModifierKind::Readonly) {
                    self.error(diagnostics::modifier_must_precede_other_modifier(
                        modifier,
                        ModifierKind::Readonly,
                    ));
                } else if kinds.contains(ModifierKind::Async) {
                    self.error(diagnostics::modifier_must_precede_other_modifier(
                        modifier,
                        ModifierKind::Async,
                    ));
                } else if kinds.contains(ModifierKind::Accessor) {
                    self.error(diagnostics::modifier_must_precede_other_modifier(
                        modifier,
                        ModifierKind::Accessor,
                    ));
                } else if kinds.contains(ModifierKind::Override) {
                    self.error(diagnostics::modifier_must_precede_other_modifier(
                        modifier,
                        ModifierKind::Override,
                    ));
                }
            }
            ModifierKind::Override => {
                if kinds.contains(ModifierKind::Override) {
                    self.error(diagnostics::modifier_already_seen(modifier));
                } else if kinds.contains(ModifierKind::Readonly) {
                    self.error(diagnostics::modifier_must_precede_other_modifier(
                        modifier,
                        ModifierKind::Readonly,
                    ));
                } else if kinds.contains(ModifierKind::Accessor) {
                    self.error(diagnostics::modifier_must_precede_other_modifier(
                        modifier,
                        ModifierKind::Accessor,
                    ));
                } else if kinds.contains(ModifierKind::Async) {
                    self.error(diagnostics::modifier_must_precede_other_modifier(
                        modifier,
                        ModifierKind::Async,
                    ));
                }
            }
            ModifierKind::Abstract => {
                if kinds.contains(ModifierKind::Abstract) {
                    self.error(diagnostics::modifier_already_seen(modifier));
                } else if kinds.contains(ModifierKind::Override) {
                    self.error(diagnostics::modifier_must_precede_other_modifier(
                        modifier,
                        ModifierKind::Override,
                    ));
                } else if kinds.contains(ModifierKind::Accessor) {
                    self.error(diagnostics::modifier_must_precede_other_modifier(
                        modifier,
                        ModifierKind::Accessor,
                    ));
                }
            }
            ModifierKind::Export => {
                if kinds.contains(ModifierKind::Export) {
                    self.error(diagnostics::modifier_already_seen(modifier));
                } else if kinds.contains(ModifierKind::Declare) {
                    self.error(diagnostics::modifier_must_precede_other_modifier(
                        modifier,
                        ModifierKind::Declare,
                    ));
                } else if kinds.contains(ModifierKind::Abstract) {
                    self.error(diagnostics::modifier_must_precede_other_modifier(
                        modifier,
                        ModifierKind::Abstract,
                    ));
                } else if kinds.contains(ModifierKind::Async) {
                    self.error(diagnostics::modifier_must_precede_other_modifier(
                        modifier,
                        ModifierKind::Async,
                    ));
                }
            }
            _ => {
                if kinds.contains(modifier.kind) {
                    self.error(diagnostics::modifier_already_seen(modifier));
                }
            }
        }
    }

    #[inline]
    pub(crate) fn verify_modifiers<F>(
        &mut self,
        modifiers: &Modifiers<'a>,
        allowed: ModifierKinds,
        // If `true`, `allowed` is exact match; if `false`, `allowed` is a superset.
        // Used for whether to pass `allowed` to `create_diagnostic` function.
        strict: bool,
        create_diagnostic: F,
    ) where
        F: Fn(&Modifier, Option<ModifierKinds>) -> OxcDiagnostic,
    {
        if modifiers.kinds.has_any_not_in(allowed) {
            // Invalid modifiers are rare, so handle this case in `#[cold]` function.
            // Also `#[inline(never)]` to help `verify_modifiers` to get inlined.
            #[cold]
            #[inline(never)]
            fn report<'a, C: Config, F>(
                parser: &mut ParserImpl<'a, C>,
                modifiers: &Modifiers<'a>,
                allowed: ModifierKinds,
                strict: bool,
                create_diagnostic: F,
            ) where
                F: Fn(&Modifier, Option<ModifierKinds>) -> OxcDiagnostic,
            {
                let mut found_invalid_modifier = false;
                for modifier in modifiers.iter() {
                    if !allowed.contains(modifier.kind) {
                        parser.error(create_diagnostic(modifier, strict.then_some(allowed)));
                        found_invalid_modifier = true;
                    }
                }
                debug_assert!(found_invalid_modifier);
            }
            report(self, modifiers, allowed, strict, create_diagnostic);
        }
    }
}
