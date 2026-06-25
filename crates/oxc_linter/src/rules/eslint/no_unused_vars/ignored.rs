use lazy_regex::Regex;
use oxc_ast::{
    AstKind,
    ast::{
        ArrayAssignmentTarget, AssignmentTarget, AssignmentTargetMaybeDefault,
        AssignmentTargetProperty, BindingPattern, ClassElement, ObjectAssignmentTarget,
    },
};

use super::{NoUnusedVars, Symbol, options::IgnorePattern};

#[derive(Debug, Default, Clone, Copy)]
pub(super) enum FoundStatus {
    /// The target identifier was not found
    #[default]
    NotFound,
    /// The target identifier was found and it meets ignore criteria
    Ignored(IgnoreReason),
    /// The target identifier was found and does not meet ignore criteria
    NotIgnored,
}

impl FoundStatus {
    #[inline]
    pub const fn is_found(self) -> bool {
        matches!(self, Self::Ignored(_) | Self::NotIgnored)
    }

    #[inline]
    pub const fn into_ignored(self) -> Ignored {
        match self {
            Self::Ignored(reason) => Ignored::from_reason(reason),
            _ => Ignored::not_ignored(),
        }
    }

    #[inline]
    pub const fn found(is_found: bool) -> Self {
        if is_found { Self::NotIgnored } else { Self::NotFound }
    }

    /// Mark a target as ignored if it's found.
    ///
    /// `false` does not make already ignored values not-ignored.
    #[inline]
    pub fn ignore(self, reason: Ignored) -> Self {
        match (self, reason.0) {
            (Self::NotIgnored, Some(reason)) => Self::Ignored(reason),
            // NamePattern takes priority
            (Self::Ignored(_), Some(IgnoreReason::NamePattern)) => {
                Self::Ignored(IgnoreReason::NamePattern)
            }
            _ => self,
        }
    }
}

/// Informs why a [Symbol] was ignored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum IgnoreReason {
    /// Symbol is ignored because it is 1) rest sibling, 2) one or more of its siblings
    /// are used, and 3) user has configured [ignoreRestSiblings] to `true`.
    ///
    /// [ignoreRestSiblings]: super::NoUnusedVarsOptions::ignore_rest_siblings
    RestSibling,
    /// Symbol is ignored because it's bound name matches a `<foo>IgnorePattern`
    /// setting.
    NamePattern,
    /// Symbol is ignored because `caughtErrors` is set to `"none"`.
    CaughtErrorsNone,
    /// Symbol is ignored because it is a class declaration with a `static`
    /// initializer block.
    ClassStaticInitBlock,
    /// Symbol is ignored because it is declared inside of an ambient scope.
    ///
    /// Not all symbols in ambient scopes are ignored.
    AmbientDeclaration,
}

/// Describes if a [Symbol] is ignored and why.
/// `None` for not ignored, `Some(reason)` when ignored.
#[derive(Debug, Clone, Copy)]
pub(super) struct Ignored(Option<IgnoreReason>);

impl Ignored {
    pub fn new(is_ignored: bool, reason: IgnoreReason) -> Self {
        Self(is_ignored.then_some(reason))
    }

    #[inline]
    pub const fn not_ignored() -> Self {
        Self(None)
    }

    #[inline]
    pub const fn from_reason(reason: IgnoreReason) -> Self {
        Self(Some(reason))
    }

    pub fn or(self, other: Self) -> Self {
        // NamePattern takes priority
        match (*self, *other) {
            (_, Some(IgnoreReason::NamePattern)) | (Some(IgnoreReason::NamePattern), _) => {
                Self::from_reason(IgnoreReason::NamePattern)
            }
            (Some(_), _) => self,
            _ => other,
        }
    }
}

impl std::ops::Deref for Ignored {
    type Target = Option<IgnoreReason>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<FoundStatus> for Ignored {
    fn from(status: FoundStatus) -> Self {
        status.into_ignored()
    }
}

impl From<Ignored> for FoundStatus {
    fn from(ignored: Ignored) -> Self {
        match ignored.0 {
            Some(reason) => FoundStatus::Ignored(reason),
            None => FoundStatus::NotIgnored,
        }
    }
}

impl NoUnusedVars {
    /// Check if a symbol should be ignored based on how it's declared.
    ///
    /// Does not handle ignore checks for re-assignments to array/object destructures.
    pub(super) fn is_ignored(&self, symbol: &Symbol<'_, '_>) -> Ignored {
        let declared_binding = symbol.name();
        match symbol.declaration().kind() {
            m if m.is_module_declaration() => self.is_ignored_var(declared_binding),
            AstKind::BindingRestElement(_)
            | AstKind::ImportDefaultSpecifier(_)
            | AstKind::ImportNamespaceSpecifier(_)
            | AstKind::ImportSpecifier(_)
            | AstKind::TSEnumDeclaration(_)
            | AstKind::TSEnumMember(_)
            | AstKind::TSImportEqualsDeclaration(_)
            | AstKind::TSInterfaceDeclaration(_)
            | AstKind::TSMappedType(_)
            | AstKind::TSModuleDeclaration(_)
            | AstKind::TSTypeAliasDeclaration(_)
            | AstKind::TSTypeParameter(_) => self.is_ignored_var(declared_binding),
            AstKind::Function(func) => {
                // Functions with TypeScript syntax are ignored only if they are truly ambient
                // (i.e., declared or in a declared module). Functions without bodies inside
                // non-declared namespaces should still be checked.
                if func.r#type.is_typescript_syntax() || func.body.is_none() {
                    return Ignored::new(
                        func.declare || symbol.is_in_declared_module(),
                        IgnoreReason::AmbientDeclaration,
                    );
                }
                self.is_ignored_var(declared_binding)
            }
            AstKind::Class(class) => {
                if class.declare {
                    return Ignored::from_reason(IgnoreReason::AmbientDeclaration);
                }
                if self.ignore_class_with_static_init_block
                    && class.body.body.iter().any(ClassElement::is_static_block)
                {
                    return Ignored::from_reason(IgnoreReason::ClassStaticInitBlock);
                }
                self.is_ignored_var(declared_binding)
            }
            AstKind::CatchParameter(catch) => self
                .is_ignored_catch_err(declared_binding)
                .or(self.is_ignored_binding_pattern(symbol, &catch.pattern)),
            AstKind::VariableDeclarator(decl) => self
                .is_ignored_var(declared_binding)
                .or(self.is_ignored_binding_pattern(symbol, &decl.id)),
            AstKind::FormalParameter(param) => self
                .is_ignored_arg(declared_binding)
                .or(self.is_ignored_binding_pattern(symbol, &param.pattern)),
            AstKind::FormalParameterRest(param) => self
                .is_ignored_arg(declared_binding)
                .or(self.is_ignored_binding_pattern(symbol, &param.rest.argument)),
            s => {
                // panic when running test cases so we can find unsupported node kinds
                debug_assert!(
                    false,
                    "is_ignored_decl did not know how to handle node of kind {}",
                    s.debug_name()
                );
                Ignored::not_ignored()
            }
        }
    }

    pub(super) fn is_ignored_binding_pattern<'a>(
        &self,
        symbol: &Symbol<'_, 'a>,
        binding: &BindingPattern<'a>,
    ) -> Ignored {
        if self.should_search_destructures() {
            self.search_binding_pattern(symbol, binding).into()
        } else {
            Ignored::not_ignored()
        }
    }

    pub(super) fn is_ignored_assignment_target<'a>(
        &self,
        symbol: &Symbol<'_, 'a>,
        assignment: &AssignmentTarget<'a>,
    ) -> Ignored {
        if self.should_search_destructures() {
            self.search_assignment_target(symbol, assignment).into()
        } else {
            Ignored::not_ignored()
        }
    }

    /// Do we need to search binding patterns to tell if a symbol is ignored, or
    /// can we just rely on [`SymbolFlags`] + the symbol's name?
    ///
    /// [`SymbolFlags`]: oxc_semantic::SymbolFlags
    #[inline]
    pub fn should_search_destructures(&self) -> bool {
        self.ignore_rest_siblings || self.destructured_array_ignore_pattern.is_some()
    }

    /// This method does the `ignoreRestNeighbors` and
    /// `arrayDestructureIgnorePattern` ignore checks for variable declarations.
    /// Not needed on function/class/interface/etc declarations because those
    /// will never be destructures.
    #[must_use]
    fn search_binding_pattern<'a>(
        &self,
        target: &Symbol<'_, 'a>,
        binding: &BindingPattern<'a>,
    ) -> FoundStatus {
        match &binding {
            // if found, not ignored. Ignoring only happens in destructuring patterns.
            BindingPattern::BindingIdentifier(id) => FoundStatus::found(target == id.as_ref()),
            BindingPattern::AssignmentPattern(id) => self.search_binding_pattern(target, &id.left),
            BindingPattern::ObjectPattern(obj) => {
                for prop in &obj.properties {
                    // check if the prop is a binding identifier (with or
                    // without an assignment) since ignore_rest_siblings does
                    // not apply to spreads that have spreads
                    //
                    // const { x: { y, z }, ...rest } = obj
                    //              ^ not ignored by ignore_rest_siblings
                    let status = self.search_binding_pattern(target, &prop.value);
                    match prop.value.get_binding_identifier() {
                        // property is the target we're looking for and is on
                        // this destructure's "level" - ignore it if the config
                        // says to ignore rest neighbors and this destructure
                        // has a ...rest.
                        Some(_) if status.is_found() => {
                            return status
                                .ignore(self.is_ignored_spread_neighbor(obj.rest.is_some()));
                        }
                        // property is an array/object destructure containing
                        // the target, our search is done. However, since it's
                        // not on this destructure's "level", we don't mess
                        // with the ignored status.
                        None if status.is_found() => {
                            return status;
                        }
                        // target not found, keep looking
                        Some(_) | None => {}
                    }
                }

                // not found in properties, check binding pattern
                obj.rest.as_ref().map_or(FoundStatus::NotFound, |rest| {
                    self.search_binding_pattern(target, &rest.argument)
                })
            }
            BindingPattern::ArrayPattern(arr) => {
                for el in arr.elements.iter().flatten() {
                    let status = self.search_binding_pattern(target, el);
                    match el.get_binding_identifier() {
                        // el is a simple pattern for the symbol we're looking
                        // for. Check if it is ignored.
                        Some(id) if target == id => {
                            return status.ignore(self.is_ignored_array_destructured(&id.name));
                        }
                        // el is a destructuring pattern containing the target
                        // symbol; our search is done, propegate it upwards
                        None if status.is_found() => {
                            debug_assert!(el.is_destructuring_pattern());
                            return status;
                        }
                        // el is a simple pattern for a different symbol, or is
                        // a destructuring pattern that doesn't contain the target. Keep looking
                        Some(_) | None => {}
                    }
                }

                FoundStatus::NotFound
            }
        }
    }

    /// Follows the same logic as [`NoUnusedVars::search_binding_pattern`], but
    /// for assignments instead of declarations
    fn search_assignment_target<'a>(
        &self,
        target: &Symbol<'_, 'a>,
        assignment: &AssignmentTarget<'a>,
    ) -> FoundStatus {
        match assignment {
            AssignmentTarget::AssignmentTargetIdentifier(id) => {
                FoundStatus::found(target == id.as_ref())
            }
            AssignmentTarget::ObjectAssignmentTarget(obj) => {
                self.search_obj_assignment_target(target, obj.as_ref())
            }
            AssignmentTarget::ArrayAssignmentTarget(arr) => {
                self.search_array_assignment_target(target, arr.as_ref())
            }
            // other assignments are going to be member expressions, identifier
            // references, or one of those two wrapped in a ts annotation-like
            // expression. Those will never have destructures and can be safely ignored.
            _ => FoundStatus::NotFound,
        }
    }

    pub(super) fn search_obj_assignment_target<'a>(
        &self,
        target: &Symbol<'_, 'a>,
        obj: &ObjectAssignmentTarget<'a>,
    ) -> FoundStatus {
        for prop in &obj.properties {
            // I'm confused about what's going on here tbh, and I wrote
            // this function... but I don't really know what a
            // `AssignmentTargetProperty::AssignmentTargetPropertyProperty`
            // is, I just know that the name is confusing.
            let status = match prop {
                AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(id) => {
                    FoundStatus::found(target == &id.binding)
                }
                // recurse down nested destructured assignments
                AssignmentTargetProperty::AssignmentTargetPropertyProperty(prop) => {
                    self.search_assignment_maybe_default(target, &prop.binding)
                }
            };

            // found, mark `status` as ignored if this destructure
            // contains a ...rest and the user asks for it
            if status.is_found() {
                let ignore_because_rest = self.is_ignored_spread_neighbor(obj.rest.is_some());
                return status.ignore(ignore_because_rest);
            }
        }

        // symbol not found in properties, try searching through ...rest
        obj.rest.as_ref().map_or(FoundStatus::NotFound, |rest| {
            self.search_assignment_target(target, &rest.target)
        })
    }

    pub(super) fn search_array_assignment_target<'a>(
        &self,
        target: &Symbol<'_, 'a>,
        arr: &ArrayAssignmentTarget<'a>,
    ) -> FoundStatus {
        // check each element in the array spread assignment
        for el in arr.elements.iter().flatten() {
            let status = self.search_assignment_maybe_default(target, el);

            // if we found the target symbol and it's not nested in some
            // other destructure, mark it as ignored if it matches the
            // configured array destructure ignore pattern.
            if status.is_found() {
                return if el.is_simple_assignment_target() {
                    status.ignore(self.is_ignored_array_destructured(target.name()))
                } else {
                    status
                };
            }
            // continue with search
        }

        FoundStatus::NotFound
    }

    fn search_assignment_maybe_default<'a>(
        &self,
        target: &Symbol<'_, 'a>,
        assignment: &AssignmentTargetMaybeDefault<'a>,
    ) -> FoundStatus {
        assignment.as_assignment_target().map_or(FoundStatus::NotFound, |assignment| {
            self.search_assignment_target(target, assignment)
        })
    }

    #[inline]
    fn is_ignored_spread_neighbor(&self, has_rest: bool) -> Ignored {
        Ignored::new(self.ignore_rest_siblings && has_rest, IgnoreReason::RestSibling)
    }

    // =========================================================================
    // ========================= NAME/KIND-ONLY CHECKS =========================
    // =========================================================================

    #[inline]
    pub(super) fn is_ignored_var(&self, name: &str) -> Ignored {
        Ignored::new(
            Self::is_none_or_match(self.vars_ignore_pattern.as_ref(), name),
            IgnoreReason::NamePattern,
        )
    }

    #[inline]
    pub(super) fn is_ignored_arg(&self, name: &str) -> Ignored {
        Ignored::new(
            Self::is_none_or_match(self.args_ignore_pattern.as_ref(), name),
            IgnoreReason::NamePattern,
        )
    }

    #[inline]
    pub(super) fn is_ignored_array_destructured(&self, name: &str) -> Ignored {
        Ignored::new(
            Self::is_none_or_match(self.destructured_array_ignore_pattern.as_ref(), name),
            IgnoreReason::NamePattern,
        )
    }

    #[inline]
    pub(super) fn is_ignored_catch_err(&self, name: &str) -> Ignored {
        if *!self.caught_errors {
            Ignored::from_reason(IgnoreReason::CaughtErrorsNone)
        } else {
            Ignored::new(
                Self::is_none_or_match(self.caught_errors_ignore_pattern.as_ref(), name),
                IgnoreReason::NamePattern,
            )
        }
    }

    #[inline]
    fn is_none_or_match(re: IgnorePattern<&Regex>, haystack: &str) -> bool {
        match re {
            IgnorePattern::None => false,
            IgnorePattern::Some(re) => re.is_match(haystack),
            IgnorePattern::Default => haystack.starts_with('_'),
        }
    }
}

#[cfg(test)]
mod test {
    use oxc_str::Str;

    use super::super::NoUnusedVars;
    use super::{IgnoreReason, Ignored};
    use crate::rule::Rule as _;

    impl std::cmp::PartialEq for super::Ignored {
        fn eq(&self, other: &Ignored) -> bool {
            self.0.eq(&other.0)
        }
    }

    #[test]
    fn test_ignored() {
        let rule = NoUnusedVars::from_configuration(serde_json::json!([
            {
                "varsIgnorePattern": "^_",
                "argsIgnorePattern": "[iI]gnored",
                "caughtErrorsIgnorePattern": "err.*",
                "caughtErrors": "all",
                "destructuredArrayIgnorePattern": "^_",
            }
        ]))
        .unwrap();

        assert_eq!(rule.is_ignored_var("_x"), Ignored::from_reason(IgnoreReason::NamePattern));
        assert_eq!(
            rule.is_ignored_var(&Str::from("_x")),
            Ignored::from_reason(IgnoreReason::NamePattern)
        );
        assert_eq!(rule.is_ignored_var("notIgnored"), Ignored::not_ignored());

        assert_eq!(rule.is_ignored_arg("ignored"), Ignored::from_reason(IgnoreReason::NamePattern));
        assert_eq!(
            rule.is_ignored_arg("alsoIgnored"),
            Ignored::from_reason(IgnoreReason::NamePattern)
        );
        assert_eq!(
            rule.is_ignored_arg(&Str::from("ignored")),
            Ignored::from_reason(IgnoreReason::NamePattern)
        );
        assert_eq!(
            rule.is_ignored_arg(&Str::from("alsoIgnored")),
            Ignored::from_reason(IgnoreReason::NamePattern)
        );

        assert_eq!(
            rule.is_ignored_catch_err("err"),
            Ignored::from_reason(IgnoreReason::NamePattern)
        );
        assert_eq!(
            rule.is_ignored_catch_err("error"),
            Ignored::from_reason(IgnoreReason::NamePattern)
        );
        assert_eq!(rule.is_ignored_catch_err("e"), Ignored::not_ignored());

        assert_eq!(
            rule.is_ignored_array_destructured("_x"),
            Ignored::from_reason(IgnoreReason::NamePattern)
        );
        assert_eq!(
            rule.is_ignored_array_destructured(&Str::from("_x")),
            Ignored::from_reason(IgnoreReason::NamePattern)
        );
        assert_eq!(rule.is_ignored_array_destructured("notIgnored"), Ignored::not_ignored());
    }

    #[test]
    fn test_ignored_catch_errors() {
        let rule = NoUnusedVars::from_configuration(serde_json::json!([
            {
                "caughtErrorsIgnorePattern": "^_",
                "caughtErrors": "all",
            }
        ]))
        .unwrap();
        assert_eq!(rule.is_ignored_catch_err("_"), Ignored::from_reason(IgnoreReason::NamePattern));
        assert_eq!(
            rule.is_ignored_catch_err("_err"),
            Ignored::from_reason(IgnoreReason::NamePattern)
        );
        assert_eq!(rule.is_ignored_catch_err("err"), Ignored::not_ignored());

        let rule = NoUnusedVars::from_configuration(serde_json::json!([
            {
                "caughtErrors": "none",
            }
        ]))
        .unwrap();
        assert_eq!(
            rule.is_ignored_catch_err("_"),
            Ignored::from_reason(IgnoreReason::CaughtErrorsNone)
        );
        assert_eq!(
            rule.is_ignored_catch_err("_err"),
            Ignored::from_reason(IgnoreReason::CaughtErrorsNone)
        );
        assert_eq!(
            rule.is_ignored_catch_err("err"),
            Ignored::from_reason(IgnoreReason::CaughtErrorsNone)
        );
    }
}
