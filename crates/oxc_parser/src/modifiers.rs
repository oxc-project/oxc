use std::{
    fmt::{self, Debug, Display},
    iter,
    mem::{self, MaybeUninit},
    num::NonZeroU16,
};

use oxc_ast::ast::TSAccessibility;
use oxc_data_structures::fieldless_enum;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

use crate::{ParserConfig as Config, ParserImpl, diagnostics, lexer::Kind};

#[derive(Debug)]
pub struct Modifier {
    pub span_start: u32,
    pub kind: ModifierKind,
}

impl Modifier {
    #[inline]
    pub const fn new(span_start: u32, kind: ModifierKind) -> Self {
        Self { span_start, kind }
    }

    #[inline]
    pub const fn span(&self) -> Span {
        Span::sized(self.span_start, self.kind.len())
    }
}

// Wrapped in a module to avoid exposing `offsets` and `kinds` fields of `Modifiers`.
// The two must be kept in sync to satisfy safety invariants.
#[expect(clippy::module_inception)]
mod modifiers {
    use super::*;

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
    ///
    /// Stored as a fixed-size array of start offsets indexed by [`ModifierKind`] discriminant.
    /// The `kinds` bitfield tracks which entries are populated.
    /// Full `Span`s are reconstructed on demand, since each modifier keyword has a fixed length.
    ///
    /// `#[repr(C)]` to make `kinds` field first. This is preferable as it is the most commonly accessed field.
    #[repr(C)]
    pub struct Modifiers {
        /// Bitfield of which modifier kinds are present.
        kinds: ModifierKinds,
        /// Start offset for each modifier, indexed by `ModifierKind` discriminant.
        /// Entries whose corresponding bit is set in `kinds` are initialized, other entries may not be.
        /// Therefore it is only safe to assume that `offsets[kind as usize]` is initialized if `kinds.contains(kind)`.
        offsets: [MaybeUninit<u32>; ModifierKind::VARIANTS.len()],
    }

    impl Modifiers {
        /// Create an empty set of modifiers.
        #[inline]
        pub const fn empty() -> Self {
            Self {
                kinds: ModifierKinds::none(),
                offsets: [MaybeUninit::uninit(); ModifierKind::VARIANTS.len()],
            }
        }

        /// Create a set of modifiers from a single modifier.
        #[inline]
        pub const fn new_single(kind: ModifierKind, start: u32) -> Self {
            let mut modifiers = Self::empty();
            modifiers.add(kind, start);
            modifiers
        }

        /// Add a modifier.
        /// If a modifier with this [`ModifierKind`] has already been added, it is overwritten.
        #[inline]
        pub(super) const fn add(&mut self, kind: ModifierKind, start: u32) {
            self.kinds = self.kinds.with(kind);
            self.offsets[kind as usize] = MaybeUninit::new(start);
        }

        #[inline]
        pub fn contains(&self, target: ModifierKind) -> bool {
            self.kinds.contains(target)
        }

        #[inline]
        pub fn kinds(&self) -> ModifierKinds {
            self.kinds
        }

        /// Iterate over all present modifiers.
        ///
        /// Order follows discriminant order (not source order).
        pub fn iter(&self) -> impl Iterator<Item = Modifier> {
            self.kinds.iter().map(|kind| {
                // SAFETY: Bits in `kinds` are set and the corresponding offset in `offsets` are initialized together
                // (in `add` method). `kinds.iter()` only yields kinds whose bit is set. So `offsets[kind as usize]`
                // must be initialized.
                let start = unsafe { self.offsets[kind as usize].assume_init() };
                Modifier::new(start, kind)
            })
        }

        /// Look up a specific modifier by [`ModifierKind`].
        pub fn get(&self, kind: ModifierKind) -> Option<Modifier> {
            if self.kinds.contains(kind) {
                // SAFETY: Bits in `kinds` are set and the corresponding offset in `offsets` are initialized together
                // (in `add` method). Here, bit for `kind` is set, so `offsets[kind as usize]` must be initialized.
                let start = unsafe { self.offsets[kind as usize].assume_init() };
                Some(Modifier::new(start, kind))
            } else {
                None
            }
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

        #[inline]
        pub fn contains_accessibility(&self) -> bool {
            self.kinds.intersects(ModifierKinds::new([
                ModifierKind::Private,
                ModifierKind::Protected,
                ModifierKind::Public,
            ]))
        }

        #[inline]
        pub fn accessibility(&self) -> Option<TSAccessibility> {
            self.kinds.accessibility()
        }
    }

    impl Debug for Modifiers {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_list().entries(self.iter()).finish()
        }
    }
}
pub use modifiers::Modifiers;

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

/// Length of each modifier keyword in bytes, indexed by [`ModifierKind`] discriminant.
static MODIFIER_LENGTHS: [u8; ModifierKind::VARIANTS.len()] = {
    let mut lengths = [0; ModifierKind::VARIANTS.len()];

    let mut i = 0;
    while i < ModifierKind::VARIANTS.len() {
        let kind = ModifierKind::VARIANTS[i];
        #[expect(clippy::cast_possible_truncation)]
        let len = kind.as_str().len() as u8;
        lengths[kind as usize] = len;
        i += 1;
    }

    lengths
};

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

    /// Get this modifier keyword.
    pub const fn as_str(self) -> &'static str {
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

    /// Get length of this modifier keyword in bytes.
    #[inline]
    pub const fn len(self) -> u32 {
        MODIFIER_LENGTHS[self as usize] as u32
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

// Wrapped in a module to avoid exposing `u16` contained in the wrapper type.
// Its value must not be altered from outside, to satisfy safety invariants.
mod modifier_kinds {
    use super::*;

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

        /// Return intersection of this set with `other`.
        ///
        /// # Example
        /// ```ignore
        /// let kinds1 = ModifierKinds::new([ModifierKind::Public, ModifierKind::Async]);
        /// let kinds2 = ModifierKinds::new([ModifierKind::Async, ModifierKind::Static]);
        /// assert_eq!(kinds1.intersection(kinds2), ModifierKinds::new([ModifierKind::Async]));
        /// ```
        #[inline]
        pub const fn intersection(self, other: Self) -> Self {
            Self(self.0 & other.0)
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

        /// Get accessibility modifier as [`TSAccessibility`], if there is one.
        ///
        /// Implemented as branchless lookup into a static 8-byte table. Boils down to 4 instructions.
        /// <https://godbolt.org/z/zc8WEs5o5>
        #[inline]
        pub fn accessibility(self) -> Option<TSAccessibility> {
            // Check that `Private`, `Protected`, and `Public` have discriminants in a tightly packed range
            // i.e. they are one after another
            const MIN_DISCRIMINANT: usize = min(
                min(ModifierKind::Private as usize, ModifierKind::Protected as usize),
                ModifierKind::Public as usize,
            );
            const _: () = {
                assert!(ModifierKind::Private as usize <= MIN_DISCRIMINANT + 2);
                assert!(ModifierKind::Protected as usize <= MIN_DISCRIMINANT + 2);
                assert!(ModifierKind::Public as usize <= MIN_DISCRIMINANT + 2);
            };

            const ACCESSIBILITY_KINDS: ModifierKinds = ModifierKinds::new([
                ModifierKind::Private,
                ModifierKind::Protected,
                ModifierKind::Public,
            ]);

            // Lookup table mapping the 8 possible accessibility modifier combinations
            // to their corresponding `Option<TSAccessibility>`.
            // Priority when multiple are set (syntax error, but handled deterministically):
            // Public > Protected > Private.
            static LUT: [Option<TSAccessibility>; 8] = {
                let combinations: [ModifierKinds; 8] = [
                    ModifierKinds::none(),
                    ModifierKinds::new([ModifierKind::Private]),
                    ModifierKinds::new([ModifierKind::Protected]),
                    ModifierKinds::new([ModifierKind::Public]),
                    ModifierKinds::new([ModifierKind::Private, ModifierKind::Protected]),
                    ModifierKinds::new([ModifierKind::Private, ModifierKind::Public]),
                    ModifierKinds::new([ModifierKind::Protected, ModifierKind::Public]),
                    ACCESSIBILITY_KINDS,
                ];

                let mut lut: [Option<TSAccessibility>; 8] = [None; 8];

                let mut i = 0;
                while i < lut.len() {
                    let combination = combinations[i];
                    let accessibility = match combination {
                        _ if combination.contains(ModifierKind::Public) => {
                            Some(TSAccessibility::Public)
                        }
                        _ if combination.contains(ModifierKind::Protected) => {
                            Some(TSAccessibility::Protected)
                        }
                        _ if combination.contains(ModifierKind::Private) => {
                            Some(TSAccessibility::Private)
                        }
                        _ => None,
                    };
                    let index = (combination.0 >> MIN_DISCRIMINANT) as usize;
                    lut[index] = accessibility;
                    i += 1;
                }

                lut
            };

            // Get only the accessibility modifiers
            let access_kinds = self.intersection(ACCESSIBILITY_KINDS);
            // Shift down to range 0..=7
            let index = (access_kinds.0 >> MIN_DISCRIMINANT) as usize;
            // Convert to `Option<TSAccessibility>` via the lookup table
            LUT[index]
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
}
pub use modifier_kinds::ModifierKinds;

impl<C: Config> ParserImpl<'_, C> {
    pub(crate) fn eat_modifiers_before_declaration(&mut self) -> Modifiers {
        let mut modifiers = Modifiers::empty();
        while let Some(modifier_kind) = self.get_modifier() {
            let modifier = Modifier::new(self.start_span(), modifier_kind);
            self.bump_any();
            self.check_modifier(modifiers.kinds(), &modifier);
            modifiers.add(modifier.kind, modifier.span_start);
        }
        modifiers
    }

    fn get_modifier(&mut self) -> Option<ModifierKind> {
        let modifier_kind = ModifierKind::try_from(self.cur_kind()).ok()?;
        if self.lookahead(Self::get_modifier_worker) { Some(modifier_kind) } else { None }
    }

    fn get_modifier_worker(&mut self) -> bool {
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

    fn modifier(&mut self, kind: Kind, span_start: u32) -> Modifier {
        let modifier_kind = ModifierKind::try_from(kind).unwrap_or_else(|()| {
            self.set_unexpected();
            ModifierKind::Abstract // Dummy value
        });
        Modifier::new(span_start, modifier_kind)
    }

    pub(crate) fn parse_modifiers(
        &mut self,
        permit_const_as_modifier: bool,
        stop_on_start_of_class_static_block: bool,
    ) -> Modifiers {
        let mut modifiers = Modifiers::empty();

        while let Some(modifier) = self.try_parse_modifier(
            modifiers.kinds(),
            permit_const_as_modifier,
            stop_on_start_of_class_static_block,
        ) {
            self.check_modifier(modifiers.kinds(), &modifier);
            modifiers.add(modifier.kind, modifier.span_start);
        }

        modifiers
    }

    fn try_parse_modifier(
        &mut self,
        seen_modifier_kinds: ModifierKinds,
        permit_const_as_modifier: bool,
        stop_on_start_of_class_static_block: bool,
    ) -> Option<Modifier> {
        let span_start = self.start_span();
        let kind = self.cur_kind();

        if kind == Kind::Const {
            if !permit_const_as_modifier {
                return None;
            }

            // We need to ensure that any subsequent modifiers appear on the same line
            // so that when 'const' is a standalone declaration, we don't issue
            // an error.
            if !self.lookahead(Self::next_token_is_on_same_line_and_can_follow_modifier) {
                return None;
            }
            self.bump_any();
        } else if
        // we're at the start of a static block
        (stop_on_start_of_class_static_block
            && kind == Kind::Static
            && self.lexer.peek_token().kind() == Kind::LCurly)
            // we may be at the start of a static block
            || (kind == Kind::Static && seen_modifier_kinds.contains(ModifierKind::Static))
            // next token is not a modifier
            || (!self.parse_any_contextual_modifier())
        {
            return None;
        }
        Some(self.modifier(kind, span_start))
    }

    pub(crate) fn parse_contextual_modifier(&mut self, kind: Kind) -> bool {
        if self.at(kind) && self.lookahead(Self::next_token_can_follow_modifier) {
            self.bump_any();
            true
        } else {
            false
        }
    }

    fn parse_any_contextual_modifier(&mut self) -> bool {
        if self.cur_kind().is_modifier_kind()
            && self.lookahead(Self::next_token_can_follow_modifier)
        {
            self.bump_any();
            true
        } else {
            false
        }
    }

    pub(crate) fn next_token_can_follow_modifier(&mut self) -> bool {
        match self.cur_kind() {
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
}

/// Static lookup table for which modifiers are illegal preceding another modifier.
/// Table is indexed by [`ModifierKind`] discriminant of the later modifier.
/// This is all calculated at compile time, and produces a 30-byte lookup table.
static ILLEGAL_PRECEDING_MODIFIERS: [ModifierKinds; ModifierKind::VARIANTS.len()] = {
    let mut illegal = [ModifierKinds::none(); ModifierKind::VARIANTS.len()];

    let mut i = 0;
    while i < illegal.len() {
        let kind = ModifierKind::VARIANTS[i];

        let illegal_kinds = get_illegal_preceding_modifiers(kind);
        assert!(illegal_kinds.contains(kind), "Same modifier twice is always illegal");
        illegal[kind as usize] = illegal_kinds;

        i += 1;
    }

    illegal
};

/// Get which modifiers are illegal to precede a modifier.
const fn get_illegal_preceding_modifiers(kind: ModifierKind) -> ModifierKinds {
    match kind {
        ModifierKind::Public | ModifierKind::Private | ModifierKind::Protected => {
            ModifierKinds::new([
                ModifierKind::Public,
                ModifierKind::Private,
                ModifierKind::Protected,
                ModifierKind::Override,
                ModifierKind::Static,
                ModifierKind::Accessor,
                ModifierKind::Readonly,
                ModifierKind::Async,
                ModifierKind::Abstract,
            ])
        }
        ModifierKind::Static => ModifierKinds::new([
            ModifierKind::Static,
            ModifierKind::Readonly,
            ModifierKind::Async,
            ModifierKind::Accessor,
            ModifierKind::Override,
        ]),
        ModifierKind::Override => ModifierKinds::new([
            ModifierKind::Override,
            ModifierKind::Readonly,
            ModifierKind::Accessor,
            ModifierKind::Async,
        ]),
        ModifierKind::Abstract => ModifierKinds::new([
            ModifierKind::Abstract,
            ModifierKind::Override,
            ModifierKind::Accessor,
        ]),
        ModifierKind::Export => ModifierKinds::new([
            ModifierKind::Export,
            ModifierKind::Declare,
            ModifierKind::Abstract,
            ModifierKind::Async,
        ]),
        _ => ModifierKinds::new([kind]),
    }
}

impl<C: Config> ParserImpl<'_, C> {
    #[inline]
    fn check_modifier(&mut self, existing_kinds: ModifierKinds, modifier: &Modifier) {
        // Do a quick check that this modifier is not illegal in this position.
        //
        // This is just 2 instructions:
        // 1. Read from a 30-byte lookup table
        // 2. AND operation to compare to `existing_kinds`.
        // https://godbolt.org/z/Mh76WTTYj
        //
        // Only if this quick check fails (syntax error, rare), then call `#[cold]` `#[inline(never)]` function
        // `illegal_modifier_error` to raise an error.
        let illegal_preceding_modifier_kinds = ILLEGAL_PRECEDING_MODIFIERS[modifier.kind as usize];

        if existing_kinds.intersects(illegal_preceding_modifier_kinds) {
            self.illegal_modifier_error(existing_kinds, modifier);
        }
    }

    /// Create an error for an illegal modifier.
    #[cold]
    #[inline(never)]
    fn illegal_modifier_error(&mut self, existing_kinds: ModifierKinds, modifier: &Modifier) {
        const ACCESSIBILITY_KINDS: ModifierKinds = ModifierKinds::new([
            ModifierKind::Public,
            ModifierKind::Private,
            ModifierKind::Protected,
        ]);

        let this_kind = modifier.kind;
        let this_kinds = ModifierKinds::new([this_kind]);

        if this_kinds.intersects(ACCESSIBILITY_KINDS) {
            // This modifier is `public`, `private`, or `protected`.
            // Using multiple accessibility modifiers is illegal.
            if existing_kinds.intersects(ACCESSIBILITY_KINDS) {
                self.error(diagnostics::accessibility_modifier_already_seen(modifier));
                return;
            }
        } else {
            // Modifiers cannot be repeated
            if existing_kinds.intersects(this_kinds) {
                self.error(diagnostics::modifier_already_seen(modifier));
                return;
            }
        }

        let illegal_preceding_modifier_kinds = ILLEGAL_PRECEDING_MODIFIERS[this_kind as usize];

        // `illegal_preceding_modifier_kinds` are modifiers which this modifier cannot follow.
        // Find which of them it *is* following, and raise an error for one of them.
        // If multiple illegal kinds, it's arbitrary which one the error is raised for.
        let illegal_kinds = illegal_preceding_modifier_kinds.intersection(existing_kinds);
        let illegal_kind = illegal_kinds.iter().next().unwrap();
        self.error(diagnostics::modifier_must_precede_other_modifier(modifier, illegal_kind));
    }

    #[inline]
    pub(crate) fn verify_modifiers<F>(
        &mut self,
        modifiers: &Modifiers,
        allowed: ModifierKinds,
        // If `true`, `allowed` is exact match; if `false`, `allowed` is a superset.
        // Used for whether to pass `allowed` to `create_diagnostic` function.
        strict: bool,
        create_diagnostic: F,
    ) where
        F: Fn(&Modifier, Option<ModifierKinds>) -> OxcDiagnostic,
    {
        if modifiers.kinds().has_any_not_in(allowed) {
            // Invalid modifiers are rare, so handle this case in `#[cold]` function.
            // Also `#[inline(never)]` to help `verify_modifiers` to get inlined.
            #[cold]
            #[inline(never)]
            fn report<C: Config, F>(
                parser: &mut ParserImpl<'_, C>,
                modifiers: &Modifiers,
                allowed: ModifierKinds,
                strict: bool,
                create_diagnostic: F,
            ) where
                F: Fn(&Modifier, Option<ModifierKinds>) -> OxcDiagnostic,
            {
                // Sort modifiers to produce errors in source code order
                let mut disallowed_modifiers = modifiers
                    .iter()
                    .filter(|modifier| !allowed.contains(modifier.kind))
                    .collect::<Vec<_>>();
                disallowed_modifiers.sort_unstable_by_key(|modifier| modifier.span_start);

                debug_assert!(!disallowed_modifiers.is_empty());

                for modifier in &disallowed_modifiers {
                    parser.error(create_diagnostic(modifier, strict.then_some(allowed)));
                }
            }
            report(self, modifiers, allowed, strict, create_diagnostic);
        }
    }
}

/// Get the minimum of two `usize`s.
///
/// Equivalent to `std::cmp::min` but can be used in const contexts.
const fn min(a: usize, b: usize) -> usize {
    if a < b { a } else { b }
}

#[cfg(test)]
mod tests {
    use oxc_ast::ast::TSAccessibility;

    use super::{ModifierKind, ModifierKinds};

    #[test]
    fn accessibility_none() {
        assert_eq!(ModifierKinds::none().accessibility(), None);
    }

    #[test]
    fn accessibility_non_accessibility_modifiers() {
        for kind in ModifierKind::VARIANTS {
            if matches!(
                kind,
                ModifierKind::Private | ModifierKind::Protected | ModifierKind::Public
            ) {
                continue;
            }
            assert_eq!(ModifierKinds::new([kind]).accessibility(), None, "{kind}");
        }
    }

    #[test]
    fn accessibility_single() {
        assert_eq!(
            ModifierKinds::new([ModifierKind::Private]).accessibility(),
            Some(TSAccessibility::Private),
        );
        assert_eq!(
            ModifierKinds::new([ModifierKind::Protected]).accessibility(),
            Some(TSAccessibility::Protected),
        );
        assert_eq!(
            ModifierKinds::new([ModifierKind::Public]).accessibility(),
            Some(TSAccessibility::Public),
        );
    }

    #[test]
    fn accessibility_with_other_modifiers() {
        let kinds = ModifierKinds::new([ModifierKind::Static, ModifierKind::Public]);
        assert_eq!(kinds.accessibility(), Some(TSAccessibility::Public));

        let kinds = ModifierKinds::new([
            ModifierKind::Async,
            ModifierKind::Protected,
            ModifierKind::Override,
        ]);
        assert_eq!(kinds.accessibility(), Some(TSAccessibility::Protected));
    }

    #[test]
    fn accessibility_multiple_accessibility_modifiers() {
        // Multiple accessibility modifiers is a syntax error, but the function should still
        // return a deterministic result. Priority: Public > Protected > Private.
        let kinds = ModifierKinds::new([ModifierKind::Private, ModifierKind::Protected]);
        assert_eq!(kinds.accessibility(), Some(TSAccessibility::Protected));

        let kinds = ModifierKinds::new([ModifierKind::Private, ModifierKind::Public]);
        assert_eq!(kinds.accessibility(), Some(TSAccessibility::Public));

        let kinds = ModifierKinds::new([ModifierKind::Protected, ModifierKind::Public]);
        assert_eq!(kinds.accessibility(), Some(TSAccessibility::Public));

        let kinds = ModifierKinds::new([
            ModifierKind::Private,
            ModifierKind::Protected,
            ModifierKind::Public,
        ]);
        assert_eq!(kinds.accessibility(), Some(TSAccessibility::Public));
    }
}
