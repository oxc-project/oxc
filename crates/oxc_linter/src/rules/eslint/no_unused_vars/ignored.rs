use oxc_ast::{
    ast::{
        ArrayAssignmentTarget, AssignmentTarget, AssignmentTargetMaybeDefault,
        AssignmentTargetProperty, BindingPattern, BindingPatternKind, ClassElement,
        ObjectAssignmentTarget,
    },
    AstKind,
};
use regex::Regex;

use super::{options::IgnorePattern, NoUnusedVars, Symbol};

#[derive(Debug, Default, Clone, Copy)]
pub(super) enum FoundStatus {
    /// The target identifier was not found
    #[default]
    NotFound,
    /// The target identifier was found and it meets ignore criteria
    Ignored,
    /// The target identifier was found and does not meet ignore criteria
    NotIgnored,
}

impl FoundStatus {
    #[inline]
    pub const fn is_found(self) -> bool {
        matches!(self, Self::Ignored | Self::NotIgnored)
    }

    #[inline]
    pub const fn is_ignored(self) -> bool {
        matches!(self, Self::Ignored)
    }

    #[inline]
    pub const fn found(is_found: bool) -> Self {
        if is_found {
            Self::NotIgnored
        } else {
            Self::NotFound
        }
    }

    /// Mark a target as ignored if it's found.
    ///
    /// `false` does not make already ignored values not-ignored.
    #[inline]
    pub fn ignore(self, is_ignored: bool) -> Self {
        match self {
            Self::NotIgnored if is_ignored => Self::Ignored,
            _ => self,
        }
    }
}

impl NoUnusedVars {
    /// Check if a symbol should be ignored based on how it's declared.
    ///
    /// Does not handle ignore checks for re-assignments to array/object destructures.
    pub(super) fn is_ignored(&self, symbol: &Symbol<'_, '_>) -> bool {
        let declared_binding = symbol.name();
        match symbol.declaration().kind() {
            AstKind::BindingRestElement(_)
            | AstKind::Function(_)
            | AstKind::ImportDefaultSpecifier(_)
            | AstKind::ImportNamespaceSpecifier(_)
            | AstKind::ImportSpecifier(_)
            | AstKind::ModuleDeclaration(_)
            | AstKind::TSEnumDeclaration(_)
            | AstKind::TSEnumMember(_)
            | AstKind::TSImportEqualsDeclaration(_)
            | AstKind::TSInterfaceDeclaration(_)
            | AstKind::TSModuleDeclaration(_)
            | AstKind::TSTypeAliasDeclaration(_)
            | AstKind::TSTypeParameter(_) => self.is_ignored_var(declared_binding),
            AstKind::Class(class) => {
                if self.ignore_class_with_static_init_block
                    && class.body.body.iter().any(ClassElement::is_static_block)
                {
                    return true;
                }
                self.is_ignored_var(declared_binding)
            }
            AstKind::CatchParameter(catch) => {
                self.is_ignored_catch_err(declared_binding)
                    || self.is_ignored_binding_pattern(symbol, &catch.pattern)
            }
            AstKind::VariableDeclarator(decl) => {
                self.is_ignored_var(declared_binding)
                    || self.is_ignored_binding_pattern(symbol, &decl.id)
            }
            AstKind::FormalParameter(param) => {
                self.is_ignored_arg(declared_binding)
                    || self.is_ignored_binding_pattern(symbol, &param.pattern)
            }
            s => {
                // panic when running test cases so we can find unsupported node kinds
                debug_assert!(
                    false,
                    "is_ignored_decl did not know how to handle node of kind {}",
                    s.debug_name()
                );
                false
            }
        }
    }

    pub(super) fn is_ignored_binding_pattern<'a>(
        &self,
        symbol: &Symbol<'_, 'a>,
        binding: &BindingPattern<'a>,
    ) -> bool {
        self.should_search_destructures()
            && self.search_binding_pattern(symbol, binding).is_ignored()
    }

    pub(super) fn is_ignored_assignment_target<'a>(
        &self,
        symbol: &Symbol<'_, 'a>,
        assignment: &AssignmentTarget<'a>,
    ) -> bool {
        self.should_search_destructures()
            && self.search_assignment_target(symbol, assignment).is_ignored()
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
        match &binding.kind {
            // if found, not ignored. Ignoring only happens in destructuring patterns.
            BindingPatternKind::BindingIdentifier(id) => FoundStatus::found(target == id.as_ref()),
            BindingPatternKind::AssignmentPattern(id) => {
                self.search_binding_pattern(target, &id.left)
            }
            BindingPatternKind::ObjectPattern(obj) => {
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
                        Some(_) | None => {
                            continue;
                        }
                    }
                }

                // not found in properties, check binding pattern
                obj.rest.as_ref().map_or(FoundStatus::NotFound, |rest| {
                    self.search_binding_pattern(target, &rest.argument)
                })
            }
            BindingPatternKind::ArrayPattern(arr) => {
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
                            debug_assert!(el.kind.is_destructuring_pattern());
                            return status;
                        }
                        // el is a simple pattern for a different symbol, or is
                        // a destructuring pattern that doesn't contain the target. Keep looking
                        Some(_) | None => {
                            continue;
                        }
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
                return status.ignore(
                    el.is_simple_assignment_target()
                        && self.is_ignored_array_destructured(target.name()),
                );
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
    fn is_ignored_spread_neighbor(&self, has_rest: bool) -> bool {
        self.ignore_rest_siblings && has_rest
    }

    // =========================================================================
    // ========================= NAME/KIND-ONLY CHECKS =========================
    // =========================================================================

    #[inline]
    pub(super) fn is_ignored_var(&self, name: &str) -> bool {
        Self::is_none_or_match(self.vars_ignore_pattern.as_ref(), name)
    }

    #[inline]
    pub(super) fn is_ignored_arg(&self, name: &str) -> bool {
        Self::is_none_or_match(self.args_ignore_pattern.as_ref(), name)
    }

    #[inline]
    pub(super) fn is_ignored_array_destructured(&self, name: &str) -> bool {
        Self::is_none_or_match(self.destructured_array_ignore_pattern.as_ref(), name)
    }

    #[inline]
    pub(super) fn is_ignored_catch_err(&self, name: &str) -> bool {
        *!self.caught_errors
            || Self::is_none_or_match(self.caught_errors_ignore_pattern.as_ref(), name)
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
    use oxc_span::Atom;

    use super::super::NoUnusedVars;
    use crate::rule::Rule as _;

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
        ]));

        assert!(rule.is_ignored_var("_x"));
        assert!(rule.is_ignored_var(&Atom::from("_x")));
        assert!(!rule.is_ignored_var("notIgnored"));

        assert!(rule.is_ignored_arg("ignored"));
        assert!(rule.is_ignored_arg("alsoIgnored"));
        assert!(rule.is_ignored_arg(&Atom::from("ignored")));
        assert!(rule.is_ignored_arg(&Atom::from("alsoIgnored")));

        assert!(rule.is_ignored_catch_err("err"));
        assert!(rule.is_ignored_catch_err("error"));
        assert!(!rule.is_ignored_catch_err("e"));

        assert!(rule.is_ignored_array_destructured("_x"));
        assert!(rule.is_ignored_array_destructured(&Atom::from("_x")));
        assert!(!rule.is_ignored_array_destructured("notIgnored"));
    }

    #[test]
    fn test_ignored_catch_errors() {
        let rule = NoUnusedVars::from_configuration(serde_json::json!([
            {
                "caughtErrorsIgnorePattern": "^_",
                "caughtErrors": "all",
            }
        ]));
        assert!(rule.is_ignored_catch_err("_"));
        assert!(rule.is_ignored_catch_err("_err"));
        assert!(!rule.is_ignored_catch_err("err"));

        let rule = NoUnusedVars::from_configuration(serde_json::json!([
            {
                "caughtErrors": "none",
            }
        ]));
        assert!(rule.is_ignored_catch_err("_"));
        assert!(rule.is_ignored_catch_err("_err"));
        assert!(rule.is_ignored_catch_err("err"));
    }
}
