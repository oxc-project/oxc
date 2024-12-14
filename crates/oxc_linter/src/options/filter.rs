use std::{borrow::Cow, fmt};

use crate::{LintPlugins, RuleCategory};

use super::AllowWarnDeny;

/// Enables, disables, and sets the severity of lint rules.
///
/// Filters come in 3 forms:
/// 1. Filter by rule name and/or plugin: `no-const-assign`, `eslint/no-const-assign`
/// 2. Filter an entire category: `correctness`
/// 3. Some unknow filter. This is a fallback used when parsing a filter string,
///    and is interpreted uniquely by the linter.
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct LintFilter {
    severity: AllowWarnDeny,
    kind: LintFilterKind,
}

impl LintFilter {
    /// # Errors
    ///
    /// If `kind` is an empty string, or is a `<plugin>/<rule>` filter but is missing either the
    /// plugin or the rule.
    pub fn new<F: TryInto<LintFilterKind>>(
        severity: AllowWarnDeny,
        kind: F,
    ) -> Result<Self, <F as TryInto<LintFilterKind>>::Error> {
        Ok(Self { severity, kind: kind.try_into()? })
    }

    #[must_use]
    pub fn allow<F: Into<LintFilterKind>>(kind: F) -> Self {
        Self { severity: AllowWarnDeny::Allow, kind: kind.into() }
    }

    #[must_use]
    pub fn warn<F: Into<LintFilterKind>>(kind: F) -> Self {
        Self { severity: AllowWarnDeny::Warn, kind: kind.into() }
    }

    #[must_use]
    pub fn deny<F: Into<LintFilterKind>>(kind: F) -> Self {
        Self { severity: AllowWarnDeny::Deny, kind: kind.into() }
    }

    #[inline]
    pub fn severity(&self) -> AllowWarnDeny {
        self.severity
    }

    #[inline]
    pub fn kind(&self) -> &LintFilterKind {
        &self.kind
    }
}

impl Default for LintFilter {
    fn default() -> Self {
        Self {
            severity: AllowWarnDeny::Warn,
            kind: LintFilterKind::Category(RuleCategory::Correctness),
        }
    }
}

impl From<LintFilter> for (AllowWarnDeny, LintFilterKind) {
    fn from(val: LintFilter) -> Self {
        (val.severity, val.kind)
    }
}

impl<'a> From<&'a LintFilter> for (AllowWarnDeny, &'a LintFilterKind) {
    fn from(val: &'a LintFilter) -> Self {
        (val.severity, &val.kind)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum LintFilterKind {
    Generic(Cow<'static, str>),
    /// e.g. `no-const-assign` or `eslint/no-const-assign`
    Rule(LintPlugins, Cow<'static, str>),
    /// e.g. `correctness`
    Category(RuleCategory),
    // TODO: plugin + category? e.g `-A react:correctness`
}

impl LintFilterKind {
    /// # Errors
    ///
    /// If `filter` is an empty string, or is a `<plugin>/<rule>` filter but is missing either the
    /// plugin or the rule.
    pub fn parse(filter: Cow<'static, str>) -> Result<Self, InvalidFilterKind> {
        if filter.is_empty() {
            return Err(InvalidFilterKind::Empty);
        }

        if filter.contains('/') {
            // this is an unfortunate amount of code duplication, but it needs to be done for
            // `filter` to live long enough to avoid a String allocation for &'static str
            let (plugin, rule) = match filter {
                Cow::Borrowed(filter) => {
                    let mut parts = filter.splitn(2, '/');

                    let plugin = parts
                        .next()
                        .ok_or(InvalidFilterKind::PluginMissing(Cow::Borrowed(filter)))?;
                    if plugin.is_empty() {
                        return Err(InvalidFilterKind::PluginMissing(Cow::Borrowed(filter)));
                    }

                    let rule = parts
                        .next()
                        .ok_or(InvalidFilterKind::RuleMissing(Cow::Borrowed(filter)))?;
                    if rule.is_empty() {
                        return Err(InvalidFilterKind::RuleMissing(Cow::Borrowed(filter)));
                    }

                    (LintPlugins::from(plugin), Cow::Borrowed(rule))
                }
                Cow::Owned(filter) => {
                    let mut parts = filter.splitn(2, '/');

                    let plugin = parts
                        .next()
                        .ok_or_else(|| InvalidFilterKind::PluginMissing(filter.clone().into()))?;
                    if plugin.is_empty() {
                        return Err(InvalidFilterKind::PluginMissing(filter.into()));
                    }

                    let rule = parts
                        .next()
                        .ok_or_else(|| InvalidFilterKind::RuleMissing(filter.clone().into()))?;
                    if rule.is_empty() {
                        return Err(InvalidFilterKind::RuleMissing(filter.into()));
                    }

                    (LintPlugins::from(plugin), Cow::Owned(rule.to_string()))
                }
            };
            Ok(LintFilterKind::Rule(plugin, rule))
        } else {
            match RuleCategory::try_from(filter.as_ref()) {
                Ok(category) => Ok(LintFilterKind::Category(category)),
                Err(()) => Ok(LintFilterKind::Generic(filter)),
            }
        }
    }
}

impl TryFrom<String> for LintFilterKind {
    type Error = InvalidFilterKind;

    #[inline]
    fn try_from(filter: String) -> Result<Self, Self::Error> {
        Self::parse(Cow::Owned(filter))
    }
}

impl TryFrom<&'static str> for LintFilterKind {
    type Error = InvalidFilterKind;

    #[inline]
    fn try_from(filter: &'static str) -> Result<Self, Self::Error> {
        Self::parse(Cow::Borrowed(filter))
    }
}

impl TryFrom<Cow<'static, str>> for LintFilterKind {
    type Error = InvalidFilterKind;

    #[inline]
    fn try_from(filter: Cow<'static, str>) -> Result<Self, Self::Error> {
        Self::parse(filter)
    }
}

impl From<RuleCategory> for LintFilterKind {
    #[inline]
    fn from(category: RuleCategory) -> Self {
        LintFilterKind::Category(category)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidFilterKind {
    Empty,
    PluginMissing(Cow<'static, str>),
    RuleMissing(Cow<'static, str>),
}

impl fmt::Display for InvalidFilterKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Empty => "Filter cannot be empty.".fmt(f),
            Self::PluginMissing(filter) => {
                write!(
                    f,
                    "Filter '{filter}' must match <plugin>/<rule> but is missing a plugin name."
                )
            }
            Self::RuleMissing(filter) => {
                write!(
                    f,
                    "Filter '{filter}' must match <plugin>/<rule> but is missing a rule name."
                )
            }
        }
    }
}

impl std::error::Error for InvalidFilterKind {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_category() {
        let correctness: LintFilter = LintFilter::new(AllowWarnDeny::Warn, "correctness").unwrap();
        assert_eq!(correctness.severity(), AllowWarnDeny::Warn);
        assert!(
            matches!(correctness.kind(), LintFilterKind::Category(RuleCategory::Correctness)),
            "{:?}",
            correctness.kind()
        );
    }

    #[test]
    fn test_eslint_deny() {
        let filter = LintFilter::deny(LintFilterKind::try_from("no-const-assign").unwrap());
        assert_eq!(filter.severity(), AllowWarnDeny::Deny);
        assert_eq!(filter.kind(), &LintFilterKind::Generic("no-const-assign".into()));

        let filter = LintFilter::deny(LintFilterKind::try_from("eslint/no-const-assign").unwrap());
        assert_eq!(filter.severity(), AllowWarnDeny::Deny);
        assert_eq!(
            filter.kind(),
            &LintFilterKind::Rule(LintPlugins::from("eslint"), "no-const-assign".into())
        );
        assert!(matches!(filter.kind(), LintFilterKind::Rule(_, _)));
    }

    #[test]
    fn test_parse() {
        #[rustfmt::skip]
        let test_cases: Vec<(&'static str, LintFilterKind)> = vec![
            ("no-const-assign", LintFilterKind::Generic("no-const-assign".into())),
            ("eslint/no-const-assign", LintFilterKind::Rule(LintPlugins::ESLINT, "no-const-assign".into())),
            ("import/namespace", LintFilterKind::Rule(LintPlugins::IMPORT, "namespace".into())),
            ("react-hooks/exhaustive-deps", LintFilterKind::Rule(LintPlugins::REACT, "exhaustive-deps".into())),
            // categories
            ("correctness", LintFilterKind::Category(RuleCategory::Correctness)),
            ("nursery", LintFilterKind::Category("nursery".try_into().unwrap())),
            ("perf", LintFilterKind::Category("perf".try_into().unwrap())),
            // misc
            ("not-a-valid-filter", LintFilterKind::Generic("not-a-valid-filter".into())),
            ("all", LintFilterKind::Generic("all".into())),
        ];

        for (input, expected) in test_cases {
            let actual = LintFilterKind::try_from(input).unwrap();
            assert_eq!(actual, expected, "input: {input}");
        }
    }

    #[test]
    fn test_parse_invalid() {
        let test_cases = vec!["/rules-of-hooks", "import/", "", "/", "//"];

        for input in test_cases {
            let actual = LintFilterKind::parse(Cow::Borrowed(input));
            assert!(
                actual.is_err(),
                "input '{input}' produced filter '{:?}' but it should have errored",
                actual.unwrap()
            );
        }
    }
}
