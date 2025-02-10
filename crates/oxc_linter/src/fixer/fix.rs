use std::{borrow::Cow, ops::Deref};

use bitflags::bitflags;
use oxc_span::{GetSpan, Span, SPAN};

bitflags! {
    /// Flags describing an automatic code fix.
    ///
    /// These are used by lint rules when they provide a code fix or suggestion.
    /// These are also used by the `LintService` to decide which kinds of
    /// changes to apply.
    ///
    /// [`FixKind`] is designed to be interoperable with [`bool`]. `true` turns
    /// into [`FixKind::Fix`] (applies only safe fixes) and `false` turns into
    /// [`FixKind::None`] (do not apply any fixes or suggestions).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct FixKind: u8 {
        /// An automatic code fix. Most of these are applied with `--fix`
        ///
        const Fix = 1 << 0;
        /// A recommendation about how to fix a rule violation. These are usually
        /// safe to apply, in that they shouldn't cause parse or runtime errors,
        /// but may change the meaning of the code.
        const Suggestion = 1 << 1;
        /// Marks a fix or suggestion as dangerous. Dangerous fixes/suggestions
        /// may break the code. Covers cases that are
        /// - Aggressive (e.g. some code removal)
        /// - Are under development. Think of this as similar to the `nursery`
        ///   rule category.
        const Dangerous = 1 << 2;

        const SafeFix = Self::Fix.bits();
        const SafeFixOrSuggestion = Self::Fix.bits() | Self::Suggestion.bits();
        const DangerousFix = Self::Dangerous.bits() | Self::Fix.bits();
        const DangerousSuggestion = Self::Dangerous.bits() | Self::Suggestion.bits();
        const DangerousFixOrSuggestion = Self::Dangerous.bits() | Self::Fix.bits() | Self::Suggestion.bits();

        /// Used to specify that no fixes should be applied.
        const None = 0;
        /// Fixes and Suggestions that are safe or dangerous.
        const All = Self::Dangerous.bits() | Self::Fix.bits() | Self::Suggestion.bits();
    }
}

// explicit definition for clarity
impl Default for FixKind {
    #[inline]
    fn default() -> Self {
        Self::None
    }
}

impl FixKind {
    #[inline]
    pub const fn is_none(self) -> bool {
        self.is_empty()
    }

    #[inline]
    pub const fn is_some(self) -> bool {
        self.bits() > 0
    }

    #[inline]
    pub const fn is_dangerous(self) -> bool {
        self.contains(Self::Dangerous)
    }

    /// Check if a fix produced by a lint rule is allowed to be applied
    /// to the source code.
    ///
    /// Here, `self` is the set of allowed [`FixKind`]s, and `rule_fix` is the
    /// kind of fixed produced by the rule.
    ///
    /// # Example
    /// ```
    /// use oxc_linter::FixKind;
    ///
    /// // `none` means no fixes will be applied at all
    /// assert!(!FixKind::None.can_apply(FixKind::SafeFix));
    ///
    /// // allow safe fixes
    /// assert!(FixKind::SafeFix.can_apply(FixKind::SafeFix));
    /// assert!(!FixKind::SafeFix.can_apply(FixKind::DangerousFix)); // not safe
    /// assert!(!FixKind::SafeFix.can_apply(FixKind::Suggestion));   // not a fix
    /// ```
    #[inline]
    pub fn can_apply(self, rule_fix: Self) -> bool {
        self.contains(rule_fix)
    }

    /// # Panics
    /// If this [`FixKind`] is only [`FixKind::Dangerous`] without a
    /// [`FixKind::Fix`] or [`FixKind::Suggestion`] qualifier.
    pub fn emoji(self) -> &'static str {
        if self.is_empty() {
            return "";
        }
        match self {
            Self::Fix => "🛠️",
            Self::Suggestion => "💡",
            Self::SafeFixOrSuggestion => "🛠️💡",
            Self::DangerousFixOrSuggestion => "⚠️🛠️️💡",
            Self::DangerousFix => "⚠️🛠️️",
            Self::DangerousSuggestion => "⚠️💡",
            Self::Dangerous => panic!(
                "Fix kinds cannot just be dangerous, they must also be 'Fix' or 'Suggestion'."
            ),
            _ => {
                debug_assert!(false, "Please add an emoji for FixKind: {self:?}");
                ""
            }
        }
    }
}

// TODO: rename
#[derive(Debug, Default)]
#[must_use = "Fixes must be used. If you don't need a fix, use `LintContext::diagnostic`, or create an empty fix using `RuleFixer::noop`."]
pub struct RuleFix<'a> {
    kind: FixKind,
    /// A suggestion message. Will be shown in editors via code actions.
    ///
    /// NOTE: code actions aren't implemented yet.
    message: Option<Cow<'a, str>>,
    /// The actual that will be applied to the source code.
    ///
    /// See: [`Fix`]
    fix: CompositeFix<'a>,
}

macro_rules! impl_from {
    ($($ty:ty),*) => {
        $(
            impl<'a> From<$ty> for RuleFix<'a> {
                fn from(fix: $ty) -> Self {
                    Self { kind: FixKind::SafeFix, message: None, fix: fix.into() }
                }
            }
        )*
    };
}
// I'd like to use
//    impl<'a, F: Into<CompositeFix<'a>>> From<F> for RuleFix<'a> b
// but this breaks when implementing `From<RuleFix<'a>> for CompositeFix<'a>`.
impl_from!(CompositeFix<'a>, Fix<'a>, Option<Fix<'a>>, Vec<Fix<'a>>);

impl<'a> FromIterator<Fix<'a>> for RuleFix<'a> {
    fn from_iter<T: IntoIterator<Item = Fix<'a>>>(iter: T) -> Self {
        Self {
            kind: FixKind::SafeFix,
            message: None,
            fix: iter.into_iter().collect::<Vec<_>>().into(),
        }
    }
}

impl<'a> From<RuleFix<'a>> for CompositeFix<'a> {
    #[inline]
    fn from(val: RuleFix<'a>) -> Self {
        val.fix
    }
}

impl<'a> RuleFix<'a> {
    #[inline]
    pub(super) fn new(kind: FixKind, message: Option<Cow<'a, str>>, fix: CompositeFix<'a>) -> Self {
        Self { kind, message, fix }
    }

    /// Create a new safe fix.
    #[inline]
    pub fn fix(fix: CompositeFix<'a>) -> Self {
        Self { kind: FixKind::Fix, message: None, fix }
    }

    /// Create a new suggestion
    #[inline]
    pub const fn suggestion(fix: CompositeFix<'a>, message: Cow<'a, str>) -> Self {
        Self { kind: FixKind::Suggestion, message: Some(message), fix }
    }

    /// Create a dangerous fix.
    #[inline]
    pub fn dangerous(fix: CompositeFix<'a>) -> Self {
        Self { kind: FixKind::DangerousFix, message: None, fix }
    }

    /// Mark this [`RuleFix`] as dangerous.
    ///
    /// This is especially useful for fixer functions that are safe in some
    /// cases but not in others.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use oxc_linter::fixer::{RuleFix, RuleFixer};
    /// use oxc_ast::ast::Expression;
    ///
    /// fn my_fixer<'a>(fixer: RuleFixer<'a>, bad_node: &Expression<'a>) -> RuleFix<'a> {
    ///   fixer.delete(bad_node).dangerously()
    /// }
    ///
    /// fn is_dangerous(bad_node: &Expression<'_>) -> bool {
    ///   // some check on bad_node
    /// #  true
    /// }
    ///
    /// fn maybe_dangerous_fixer<'a>(fixer: RuleFixer<'a>, bad_node: &Expression<'a>) -> RuleFix<'a> {
    ///   let fix = fixer.delete(bad_node);
    ///   if is_dangerous() {
    ///     fix.dangerously()
    ///   } else {
    ///     fix
    ///   }
    /// }
    /// ```
    pub fn dangerously(mut self) -> Self {
        self.kind.set(FixKind::Dangerous, true);
        self
    }

    #[inline]
    pub fn with_message<S: Into<Cow<'a, str>>>(mut self, message: S) -> Self {
        self.message = Some(message.into());
        self
    }

    #[inline]
    pub fn kind(&self) -> FixKind {
        self.kind
    }

    #[inline]
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    #[inline]
    pub fn into_fix(self, source_text: &str) -> Fix<'a> {
        self.fix.normalize_fixes(source_text)
    }

    #[inline]
    pub fn extend<F: Into<CompositeFix<'a>>>(mut self, fix: F) -> Self {
        self.fix = self.fix.concat(fix.into());
        self
    }

    #[inline]
    pub fn push<F: Into<CompositeFix<'a>>>(&mut self, fix: F) {
        self.fix.push(fix.into());
    }
}

impl GetSpan for RuleFix<'_> {
    fn span(&self) -> Span {
        self.fix.span()
    }
}

impl<'a> Deref for RuleFix<'a> {
    type Target = CompositeFix<'a>;

    fn deref(&self) -> &Self::Target {
        &self.fix
    }
}

/// A completed, normalized fix ready to be applied to the source code.
///
/// Used internally by this module. Lint rules should use [`RuleFix`].
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Fix<'a> {
    pub content: Cow<'a, str>,
    pub span: Span,
}

impl Default for Fix<'_> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<'a> Fix<'a> {
    pub const fn delete(span: Span) -> Self {
        Self { content: Cow::Borrowed(""), span }
    }

    pub fn new<T: Into<Cow<'a, str>>>(content: T, span: Span) -> Self {
        Self { content: content.into(), span }
    }

    /// Creates a [`Fix`] that doesn't change the source code.
    #[inline]
    pub const fn empty() -> Self {
        Self { content: Cow::Borrowed(""), span: SPAN }
    }
}

// NOTE (@DonIsaac): having these variants is effectively the same as interning
// single or 0-element Vecs. I experimented with using smallvec here, but the
// resulting struct size was larger (40 bytes vs 32). So, we're sticking with
// this (at least for now).
#[derive(Debug, Default)]
pub enum CompositeFix<'a> {
    /// No fixes
    #[default]
    None,
    Single(Fix<'a>),
    /// Several fixes that will be merged into one, in order.
    Multiple(Vec<Fix<'a>>),
}

impl<'a> From<Fix<'a>> for CompositeFix<'a> {
    fn from(fix: Fix<'a>) -> Self {
        CompositeFix::Single(fix)
    }
}

impl<'a> From<Option<Fix<'a>>> for CompositeFix<'a> {
    fn from(fix: Option<Fix<'a>>) -> Self {
        match fix {
            Some(fix) => CompositeFix::Single(fix),
            None => CompositeFix::None,
        }
    }
}

impl<'a> From<Vec<Fix<'a>>> for CompositeFix<'a> {
    fn from(mut fixes: Vec<Fix<'a>>) -> Self {
        match fixes.len() {
            0 => CompositeFix::None,
            // fixes[0] doesn't correctly move the vec's entry
            1 => CompositeFix::Single(fixes.pop().unwrap()),
            _ => CompositeFix::Multiple(fixes),
        }
    }
}

impl<'a> From<Vec<CompositeFix<'a>>> for CompositeFix<'a> {
    fn from(fixes: Vec<Self>) -> Self {
        fixes.into_iter().reduce(Self::concat).unwrap_or_default()
    }
}

impl GetSpan for CompositeFix<'_> {
    fn span(&self) -> Span {
        match self {
            CompositeFix::Single(fix) => fix.span,
            CompositeFix::Multiple(fixes) => {
                fixes.iter().map(|fix| fix.span).reduce(Span::merge).unwrap_or(SPAN)
            }
            CompositeFix::None => SPAN,
        }
    }
}

impl<'a> CompositeFix<'a> {
    pub fn push(&mut self, fix: CompositeFix<'a>) {
        match self {
            Self::None => *self = fix,
            Self::Single(fix1) => match fix {
                Self::None => {}
                Self::Single(other_fix) => {
                    *self = Self::Multiple(vec![std::mem::take(fix1), other_fix]);
                }
                Self::Multiple(mut fixes) => {
                    fixes.insert(0, std::mem::take(fix1));
                    *self = Self::Multiple(fixes);
                }
            },
            Self::Multiple(fixes) => match fix {
                Self::None => {}
                Self::Single(fix) => {
                    fixes.push(fix);
                }
                Self::Multiple(other_fixes) => fixes.extend(other_fixes),
            },
        }
    }

    #[cold]
    #[must_use]
    pub fn concat(self, fix: CompositeFix<'a>) -> Self {
        match (self, fix) {
            (Self::None, f) | (f, Self::None) => f,
            (Self::Single(fix1), Self::Single(fix2)) => Self::Multiple(vec![fix1, fix2]),
            (Self::Single(fix), Self::Multiple(mut fixes)) => {
                fixes.insert(0, fix);
                Self::Multiple(fixes)
            }
            (Self::Multiple(mut fixes), Self::Single(fix)) => {
                fixes.push(fix);
                Self::Multiple(fixes)
            }
            (Self::Multiple(mut fixes1), Self::Multiple(fixes2)) => {
                fixes1.extend(fixes2);
                Self::Multiple(fixes1)
            }
        }
    }

    /// Gets the number of [`Fix`]es contained in this [`CompositeFix`].
    pub fn len(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Single(_) => 1,
            Self::Multiple(fs) => {
                debug_assert!(
                    fs.len() > 1,
                    "Single-element or empty composite fix vecs should have been turned into CompositeFix::None or CompositeFix::Single"
                );
                fs.len()
            }
        }
    }

    /// Returns `true` if this [`CompositeFix`] contains no [`Fix`]es
    pub fn is_empty(&self) -> bool {
        match self {
            Self::None => true,
            Self::Single(_) => false,
            Self::Multiple(fs) => {
                debug_assert!(
                    !fs.is_empty(),
                    "Empty CompositeFix vecs should have been turned into CompositeFix::None"
                );
                fs.is_empty()
            }
        }
    }

    // TODO: do we want this?
    // pub fn extend(&mut self, fix: CompositeFix<'a>) {
    //     match self {
    //         Self::None => *self = fix,
    //         Self::Single(fix1) => {
    //             match fix {
    //                 Self::None => {},
    //                 Self::Single(fix2) => *self = Self::Multiple(vec![fix1.clone(), fix2]),
    //                 Self::Multiple(mut fixes) => {
    //                     fixes.insert(0, fix1.clone());
    //                     *self = Self::Multiple(fixes);
    //                 }
    //             }
    //         }
    //         Self::Multiple(fixes) => {
    //             match fix {
    //                 Self::None => {},
    //                 Self::Single(fix2) => fixes.push(fix2),
    //                 Self::Multiple(fixes2) => fixes.extend(fixes2),
    //             }
    //         }
    //     }
    // }

    /// Gets one fix from the fixes. If we retrieve multiple fixes, this merges those into one.
    /// <https://github.com/eslint/eslint/blob/v9.9.1/lib/linter/report-translator.js#L181-L203>
    pub fn normalize_fixes(self, source_text: &str) -> Fix<'a> {
        match self {
            CompositeFix::Single(fix) => fix,
            CompositeFix::Multiple(fixes) => Self::merge_fixes(fixes, source_text),
            CompositeFix::None => Fix::empty(),
        }
    }

    /// Merges multiple fixes to one, returns an [`Fix::empty`] (which will not fix anything) if:
    ///
    /// 1. `fixes` is empty
    /// 2. contains overlapped ranges
    /// 3. contains negative ranges (span.start > span.end)
    ///
    /// <https://github.com/eslint/eslint/blob/v9.9.1/lib/linter/report-translator.js#L147-L179>
    fn merge_fixes(fixes: Vec<Fix<'a>>, source_text: &str) -> Fix<'a> {
        let mut fixes = fixes;
        if fixes.is_empty() {
            // Do nothing
            return Fix::empty();
        } else if fixes.len() == 1 {
            return fixes.pop().unwrap();
        }

        fixes.sort_unstable_by(|a, b| a.span.cmp(&b.span));

        // safe, as fixes.len() > 1
        let start = fixes[0].span.start;
        let end = fixes[fixes.len() - 1].span.end;
        let mut last_pos = start;
        let mut output = String::new();

        for fix in fixes {
            let Fix { content, span } = fix;
            // negative range or overlapping ranges is invalid
            if span.start > span.end {
                debug_assert!(false, "Negative range is invalid: {span:?}");
                return Fix::empty();
            }
            if last_pos > span.start {
                debug_assert!(
                    false,
                    "Fix must not be overlapped, last_pos: {}, span.start: {}",
                    last_pos, span.start
                );
                return Fix::empty();
            }

            let Some(before) = source_text.get((last_pos) as usize..span.start as usize) else {
                debug_assert!(false, "Invalid range: {}, {}", last_pos, span.start);
                return Fix::empty();
            };

            output.reserve(before.len() + content.len());
            output.push_str(before);
            output.push_str(&content);
            last_pos = span.end;
        }

        let Some(after) = source_text.get(last_pos as usize..end as usize) else {
            debug_assert!(false, "Invalid range: {:?}", last_pos as usize..end as usize);
            return Fix::empty();
        };

        output.push_str(after);
        output.shrink_to_fit();
        Fix::new(output, Span::new(start, end))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl PartialEq for Fix<'_> {
        fn eq(&self, other: &Self) -> bool {
            self.span == other.span && self.content == other.content
        }
    }

    impl Clone for CompositeFix<'_> {
        fn clone(&self) -> Self {
            match self {
                Self::None => Self::None,
                Self::Single(f) => Self::Single(f.clone()),
                Self::Multiple(fs) => Self::Multiple(fs.clone()),
            }
        }
    }

    impl PartialEq for CompositeFix<'_> {
        fn eq(&self, other: &Self) -> bool {
            match self {
                Self::None => matches!(other, CompositeFix::None),
                Self::Single(fix) => {
                    let Self::Single(other) = other else {
                        return false;
                    };
                    fix == other
                }
                Self::Multiple(fixes) => {
                    let Self::Multiple(other) = other else {
                        return false;
                    };
                    if fixes.len() != other.len() {
                        return false;
                    }
                    fixes.iter().zip(other.iter()).all(|(a, b)| a == b)
                }
            }
        }
    }

    #[test]
    fn test_none() {
        assert!(FixKind::None.is_none());
        assert!(!FixKind::SafeFix.is_none());
        assert_eq!(FixKind::default(), FixKind::None);
    }

    #[test]
    fn test_can_apply() {
        assert!(FixKind::SafeFix.can_apply(FixKind::SafeFix));
        assert!(!FixKind::SafeFix.can_apply(FixKind::Suggestion));
        assert!(!FixKind::SafeFix.can_apply(FixKind::DangerousFix));

        assert!(FixKind::DangerousFix.can_apply(FixKind::SafeFix));
        assert!(FixKind::DangerousFix.can_apply(FixKind::DangerousFix));
        assert!(!FixKind::DangerousFix.can_apply(FixKind::Suggestion));

        assert!(!FixKind::None.can_apply(FixKind::SafeFix));
        assert!(!FixKind::None.can_apply(FixKind::Suggestion));
        assert!(!FixKind::None.can_apply(FixKind::DangerousFix));
    }

    #[test]
    fn test_composite_push_on_none() {
        let f: CompositeFix = Fix::new("foo", Span::empty(4)).into();

        let mut none = CompositeFix::None;
        none.push(CompositeFix::None);
        assert_eq!(none, CompositeFix::None);

        none.push(f.clone());
        assert_eq!(&none, &f);

        let mut none = CompositeFix::None;
        let fixes = CompositeFix::from(vec![f.clone(), f]);
        none.push(fixes.clone());
        assert_eq!(none.len(), 2);
        assert_eq!(none, fixes);
    }

    #[test]
    fn test_composite_push_on_single() {
        let f1 = Fix::new("foo", Span::empty(4));
        let f2 = Fix::new("bar", Span::empty(5));
        let f3 = Fix::new("baz", Span::empty(6));
        let single = || CompositeFix::Single(f1.clone());

        // None.push(single) == single
        let mut f = single();
        f.push(CompositeFix::None);
        assert_eq!(f, single());

        // single1.push(single2) == [single1, single2]
        f.push(CompositeFix::Single(f2.clone()));
        assert_eq!(
            f,
            CompositeFix::Multiple(vec![
                Fix::new("foo", Span::empty(4)),
                Fix::new("bar", Span::empty(5))
            ])
        );

        // single.push([f1, f2]) == [single, f1, f2]
        let mut f = single();
        f.push(vec![f2.clone(), f3.clone()].into());

        assert_eq!(f, CompositeFix::Multiple(vec![f1.clone(), f2.clone(), f3.clone()]));
    }

    #[test]
    fn test_composite_push_on_multiple() {
        let f1 = Fix::new("foo", Span::empty(4));
        let f2 = Fix::new("bar", Span::empty(5));
        let f3 = Fix::new("baz", Span::empty(6));
        let multiple = || CompositeFix::Multiple(vec![f1.clone(), f2.clone()]);

        // None.push(multiple) == multiple
        let mut f = multiple();
        f.push(CompositeFix::None);
        assert_eq!(f, multiple());

        // [f1, f2].push(f3) == [f1, f2, f3]
        let mut f = multiple();
        f.push(CompositeFix::Single(f3.clone()));
        assert_eq!(f, CompositeFix::Multiple(vec![f1.clone(), f2.clone(), f3.clone()]));

        // [f1, f2].push([f3, f3]) == [f1, f2, f3, f3]
        let mut f = multiple();
        f.push(vec![f3.clone(), f3.clone()].into());
        assert_eq!(f, CompositeFix::Multiple(vec![f1, f2, f3.clone(), f3]));
    }

    #[test]
    fn test_emojis() {
        let tests = vec![
            (FixKind::None, ""),
            (FixKind::Fix, "🛠️"),
            (FixKind::Suggestion, "💡"),
            (FixKind::Suggestion | FixKind::Fix, "🛠️💡"),
            (FixKind::DangerousFix, "⚠️🛠️️"),
            (FixKind::DangerousSuggestion, "⚠️💡"),
            (FixKind::DangerousFix.union(FixKind::Suggestion), "⚠️🛠️️💡"),
        ];

        for (kind, expected) in tests {
            assert_eq!(kind.emoji(), expected, "Expected {kind:?} to have emoji '{expected}'.");
        }
    }

    #[test]
    #[should_panic(
        expected = "Fix kinds cannot just be dangerous, they must also be 'Fix' or 'Suggestion'."
    )]
    fn test_emojis_invalid() {
        FixKind::Dangerous.emoji();
    }
}
