use rustc_hash::FxHashSet;

use crate::{
    options::LintPlugins, rules::RULES, AllowWarnDeny, FixKind, FrameworkFlags, LintConfig,
    LintFilter, LintFilterKind, LintOptions, Linter, Oxlintrc, RuleCategory, RuleEnum,
    RuleWithSeverity,
};
use std::{
    cell::{Ref, RefCell},
    fmt,
};

#[must_use = "You dropped your builder without building a Linter! Did you mean to call .build()?"]
pub struct LinterBuilder {
    rules: FxHashSet<RuleWithSeverity>,
    options: LintOptions,
    config: LintConfig,
    cache: RulesCache,
}

impl Default for LinterBuilder {
    fn default() -> Self {
        Self { rules: Self::warn_correctness(LintPlugins::ESLINT), ..Self::empty() }
    }
}

impl LinterBuilder {
    /// Create a [`LinterBuilder`] with default plugins enabled and no
    /// configured rules.
    ///
    /// You can think of this as `oxlint -A all`.
    pub fn empty() -> Self {
        let options = LintOptions::default();
        let cache = RulesCache::new(options.plugins);
        Self { rules: FxHashSet::default(), options, config: LintConfig::default(), cache }
    }

    /// Warn on all rules in all plugins and categories, including those in `nursery`.
    /// This is the kitchen sink.
    ///
    /// You can think of this as `oxlint -W all -W nursery`.
    pub fn all() -> Self {
        let options = LintOptions { plugins: LintPlugins::all(), ..LintOptions::default() };
        let cache = RulesCache::new(options.plugins);
        Self {
            rules: RULES
                .iter()
                .map(|rule| RuleWithSeverity { rule: rule.clone(), severity: AllowWarnDeny::Warn })
                .collect(),
            options,
            config: LintConfig::default(),
            cache,
        }
    }

    /// Create a [`LinterBuilder`] from a loaded or manually built [`Oxlintrc`].
    /// `start_empty` will configure the builder to contain only the
    /// configuration settings from the config. When this is `false`, the config
    /// will be applied on top of a default [`Oxlintrc`].
    ///
    /// # Example
    /// Here's how to create a [`Linter`] from a `.oxlintrc.json` file.
    /// ```
    /// use oxc_linter::{LinterBuilder, Oxlintrc};
    /// let oxlintrc = Oxlintrc::from_file("path/to/.oxlintrc.json").unwrap();
    /// let linter = LinterBuilder::from_oxlintrc(true, oxlintrc).build();
    /// // you can use `From` as a shorthand for `from_oxlintrc(false, oxlintrc)`
    /// let linter = LinterBuilder::from(oxlintrc).build();
    /// ```
    pub fn from_oxlintrc(start_empty: bool, oxlintrc: Oxlintrc) -> Self {
        // TODO: monorepo config merging, plugin-based extends, etc.
        let Oxlintrc {
            plugins,
            extends: _,
            root: _,
            settings,
            env,
            globals,
            rules: oxlintrc_rules,
        } = oxlintrc;

        let config = LintConfig { settings, env, globals };
        let options = LintOptions { plugins, ..Default::default() };
        let rules =
            if start_empty { FxHashSet::default() } else { Self::warn_correctness(plugins) };
        let cache = RulesCache::new(options.plugins);
        let mut builder = Self { rules, options, config, cache };

        {
            let all_rules = builder.cache.borrow();
            oxlintrc_rules.override_rules(&mut builder.rules, all_rules.as_slice());
        }

        builder
    }

    #[inline]
    pub fn with_framework_hints(mut self, flags: FrameworkFlags) -> Self {
        self.options.framework_hints = flags;
        self
    }

    #[inline]
    pub fn and_framework_hints(mut self, flags: FrameworkFlags) -> Self {
        self.options.framework_hints |= flags;
        self
    }

    #[inline]
    pub fn with_fix(mut self, fix: FixKind) -> Self {
        self.options.fix = fix;
        self
    }

    #[inline]
    pub fn with_plugins(mut self, plugins: LintPlugins) -> Self {
        self.options.plugins = plugins;
        self.cache.set_plugins(plugins);
        self
    }

    #[inline]
    pub fn and_plugins(mut self, plugins: LintPlugins, enabled: bool) -> Self {
        self.options.plugins.set(plugins, enabled);
        self.cache.set_plugins(self.options.plugins);
        self
    }

    #[cfg(test)]
    pub(crate) fn with_rule(mut self, rule: RuleWithSeverity) -> Self {
        self.rules.insert(rule);
        self
    }

    pub fn with_filters<I: IntoIterator<Item = LintFilter>>(mut self, filters: I) -> Self {
        for filter in filters {
            self = self.with_filter(filter);
        }
        self
    }

    pub fn with_filter(mut self, filter: LintFilter) -> Self {
        let (severity, filter) = filter.into();
        let all_rules = self.cache.borrow();

        match severity {
            AllowWarnDeny::Deny | AllowWarnDeny::Warn => match filter {
                LintFilterKind::Category(category) => {
                    self.rules.extend(
                        all_rules
                            .iter()
                            .filter(|rule| rule.category() == category)
                            .map(|rule| RuleWithSeverity::new(rule.clone(), severity)),
                    );
                }
                LintFilterKind::Rule(_, name) => {
                    self.rules.extend(
                        all_rules
                            .iter()
                            .filter(|rule| rule.name() == name)
                            .map(|rule| RuleWithSeverity::new(rule.clone(), severity)),
                    );
                }
                LintFilterKind::Generic(name_or_category) => {
                    if name_or_category == "all" {
                        self.rules.extend(
                            all_rules
                                .iter()
                                .filter(|rule| rule.category() != RuleCategory::Nursery)
                                .map(|rule| RuleWithSeverity::new(rule.clone(), severity)),
                        );
                    } else {
                        self.rules.extend(
                            all_rules
                                .iter()
                                .filter(|rule| rule.name() == name_or_category)
                                .map(|rule| RuleWithSeverity::new(rule.clone(), severity)),
                        );
                    }
                }
            },
            AllowWarnDeny::Allow => match filter {
                LintFilterKind::Category(category) => {
                    self.rules.retain(|rule| rule.category() != category);
                }
                LintFilterKind::Rule(_, name) => {
                    self.rules.retain(|rule| rule.name() != name);
                }
                LintFilterKind::Generic(name_or_category) => {
                    if name_or_category == "all" {
                        self.rules.clear();
                    } else {
                        self.rules.retain(|rule| rule.name() != name_or_category);
                    }
                }
            },
        }
        drop(all_rules);

        self
    }

    #[must_use]
    pub fn build(self) -> Linter {
        let mut rules = self.rules.into_iter().collect::<Vec<_>>();
        rules.sort_unstable_by_key(|r| r.id());
        Linter::new(rules, self.options, self.config)
    }

    /// Warn for all correctness rules in the given set of plugins.
    fn warn_correctness(plugins: LintPlugins) -> FxHashSet<RuleWithSeverity> {
        RULES
            .iter()
            .filter(|rule| {
                // NOTE: this logic means there's no way to disable ESLint
                // correctness rules. I think that's fine for now.
                rule.category() == RuleCategory::Correctness
                    && plugins.contains(LintPlugins::from(rule.plugin_name()))
            })
            .map(|rule| RuleWithSeverity { rule: rule.clone(), severity: AllowWarnDeny::Warn })
            .collect()
    }
}

impl From<Oxlintrc> for LinterBuilder {
    #[inline]
    fn from(oxlintrc: Oxlintrc) -> Self {
        Self::from_oxlintrc(false, oxlintrc)
    }
}

impl fmt::Debug for LinterBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LinterBuilder")
            .field("rules", &self.rules)
            .field("options", &self.options)
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

struct RulesCache(RefCell<Option<Vec<RuleEnum>>>, LintPlugins);
impl RulesCache {
    #[inline]
    #[must_use]
    pub fn new(plugins: LintPlugins) -> Self {
        Self(RefCell::new(None), plugins)
    }

    pub fn set_plugins(&mut self, plugins: LintPlugins) {
        self.1 = plugins;
        self.clear();
    }

    #[must_use]
    fn borrow(&self) -> Ref<'_, Vec<RuleEnum>> {
        let cached = self.0.borrow();
        if cached.is_some() {
            Ref::map(cached, |cached| cached.as_ref().unwrap())
        } else {
            drop(cached);
            self.initialize();
            Ref::map(self.0.borrow(), |cached| cached.as_ref().unwrap())
        }
    }

    /// # Panics
    /// If the cache cell is currently borrowed.
    fn clear(&self) {
        *self.0.borrow_mut() = None;
    }

    /// Forcefully initialize this cache with all rules in all plugins currently
    /// enabled.
    ///
    /// This will clobber whatever value is currently stored. It should only be
    /// called when the cache is not populated, either because it has not been
    /// initialized yet or it was cleared with [`Self::clear`].
    ///
    /// # Panics
    /// If the cache cell is currently borrowed.
    fn initialize(&self) {
        debug_assert!(
            self.0.borrow().is_none(),
            "Cannot re-initialize a populated rules cache. It must be cleared first."
        );

        let mut all_rules: Vec<_> = if self.1.is_all() {
            RULES.clone()
        } else {
            RULES
                .iter()
                .filter(|rule| self.1.contains(LintPlugins::from(rule.plugin_name())))
                .cloned()
                .collect()
        };
        all_rules.sort_unstable(); // TODO: do we need to sort? is is already sorted?

        *self.0.borrow_mut() = Some(all_rules);
    }
}
