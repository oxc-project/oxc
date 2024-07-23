#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_semantic::{Semantic, SymbolId};
use oxc_span::{GetSpan, Span};
use std::ops::BitOr;

use super::{options::NoUnusedVarsOptions, symbol::Symbol};

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct UnusedBindingResult(Span, bool);

impl UnusedBindingResult {
    pub fn is_ignore(&self) -> bool {
        self.1
    }
    pub fn ignore(mut self) -> Self {
        self.1 = true;
        self
    }
}

impl GetSpan for UnusedBindingResult {
    fn span(&self) -> Span {
        self.0
    }
}
impl BitOr<bool> for UnusedBindingResult {
    type Output = Self;

    fn bitor(mut self, ignored: bool) -> Self {
        self.1 = self.1 || ignored;
        self
    }
}
impl From<Span> for UnusedBindingResult {
    fn from(span: Span) -> Self {
        Self(span, false)
    }
}

pub(super) trait CheckBinding<'a> {
    fn check_unused_binding_pattern(
        &self,
        options: &NoUnusedVarsOptions,
        symbol: &Symbol<'_, 'a>,
    ) -> Option<UnusedBindingResult>;
}

// =============================================================================
// ================================= BINDINGS ==================================
// =============================================================================

impl<'a> CheckBinding<'a> for BindingPattern<'a> {
    fn check_unused_binding_pattern(
        &self,
        options: &NoUnusedVarsOptions,
        symbol: &Symbol<'_, 'a>,
    ) -> Option<UnusedBindingResult> {
        self.kind.check_unused_binding_pattern(options, symbol)
    }
}

impl<'a> CheckBinding<'a> for BindingPatternKind<'a> {
    fn check_unused_binding_pattern(
        &self,
        options: &NoUnusedVarsOptions,
        symbol: &Symbol<'_, 'a>,
    ) -> Option<UnusedBindingResult> {
        match self {
            Self::BindingIdentifier(id) => id.check_unused_binding_pattern(options, symbol),
            Self::AssignmentPattern(id) => id.check_unused_binding_pattern(options, symbol),
            Self::ObjectPattern(id) => id.check_unused_binding_pattern(options, symbol),
            Self::ArrayPattern(id) => id.check_unused_binding_pattern(options, symbol),
        }
    }
}

impl<'a> CheckBinding<'a> for BindingIdentifier<'a> {
    fn check_unused_binding_pattern(
        &self,
        _options: &NoUnusedVarsOptions,
        symbol: &Symbol<'_, 'a>,
    ) -> Option<UnusedBindingResult> {
        (symbol == self).then(|| UnusedBindingResult::from(self.span()))
    }
}

impl<'a> CheckBinding<'a> for AssignmentPattern<'a> {
    fn check_unused_binding_pattern(
        &self,
        options: &NoUnusedVarsOptions,
        symbol: &Symbol<'_, 'a>,
    ) -> Option<UnusedBindingResult> {
        self.left.check_unused_binding_pattern(options, symbol)
    }
}

impl<'a> CheckBinding<'a> for ObjectPattern<'a> {
    fn check_unused_binding_pattern(
        &self,
        options: &NoUnusedVarsOptions,
        symbol: &Symbol<'_, 'a>,
    ) -> Option<UnusedBindingResult> {
        for el in &self.properties {
            if let Some(res) = el.check_unused_binding_pattern(options, symbol) {
                // has a rest sibling and the rule is configured to
                // ignore variables that have them
                let is_ignorable = options.ignore_rest_siblings && self.rest.is_some();
                return Some(res | is_ignorable);
            }
        }
        return self
            .rest
            .as_ref()
            .and_then(|rest| rest.check_unused_binding_pattern(options, symbol));
    }
}

impl<'a> CheckBinding<'a> for BindingProperty<'a> {
    fn check_unused_binding_pattern(
        &self,
        options: &NoUnusedVarsOptions,
        symbol: &Symbol<'_, 'a>,
    ) -> Option<UnusedBindingResult> {
        self.value.check_unused_binding_pattern(options, symbol)
    }
}

impl<'a> CheckBinding<'a> for BindingRestElement<'a> {
    fn check_unused_binding_pattern(
        &self,
        options: &NoUnusedVarsOptions,
        symbol: &Symbol<'_, 'a>,
    ) -> Option<UnusedBindingResult> {
        self.argument.check_unused_binding_pattern(options, symbol)
    }
}

impl<'a> CheckBinding<'a> for ArrayPattern<'a> {
    fn check_unused_binding_pattern(
        &self,
        options: &NoUnusedVarsOptions,
        symbol: &Symbol<'_, 'a>,
    ) -> Option<UnusedBindingResult> {
        for el in &self.elements {
            let Some(el) = el.as_ref() else {
                continue;
            };
            // const [a, _b, c] = arr; console.log(a, b)
            // here, res will contain data for _b, and we want to check if it
            // can be ignored (if it matches destructuredArrayIgnorePattern)
            if let Some(res) = el.check_unused_binding_pattern(options, symbol) {
                // const [{ _a }] = arr shouldn't get ignored since _a is inside
                // an object destructure
                if el.kind.is_destructuring_pattern() {
                    return Some(res);
                }
                let is_ignorable = options
                    .destructured_array_ignore_pattern
                    .as_ref()
                    .is_some_and(|pattern| pattern.is_match(symbol.name()));
                return Some(res | is_ignorable);
            }
        }
        None
    }
}

// =============================================================================
// ============================== RE-ASSIGNMENTS ===============================
// =============================================================================

impl<'a> CheckBinding<'a> for AssignmentExpression<'a> {
    fn check_unused_binding_pattern(
        &self,
        options: &NoUnusedVarsOptions,
        symbol: &Symbol<'_, 'a>,
    ) -> Option<UnusedBindingResult> {
        self.left.check_unused_binding_pattern(options, symbol)
    }
}

impl<'a> CheckBinding<'a> for AssignmentTarget<'a> {
    fn check_unused_binding_pattern(
        &self,
        options: &NoUnusedVarsOptions,
        symbol: &Symbol<'_, 'a>,
    ) -> Option<UnusedBindingResult> {
        match self {
            AssignmentTarget::AssignmentTargetIdentifier(id) => {
                id.check_unused_binding_pattern(options, symbol)
            }
            AssignmentTarget::ArrayAssignmentTarget(arr) => {
                arr.check_unused_binding_pattern(options, symbol)
            }
            AssignmentTarget::ObjectAssignmentTarget(obj) => {
                obj.check_unused_binding_pattern(options, symbol)
            }
            _ => None,
        }
    }
}

impl<'a> CheckBinding<'a> for ObjectAssignmentTarget<'a> {
    fn check_unused_binding_pattern(
        &self,
        options: &NoUnusedVarsOptions,
        symbol: &Symbol<'_, 'a>,
    ) -> Option<UnusedBindingResult> {
        if options.ignore_rest_siblings && self.rest.is_some() {
            return Some(UnusedBindingResult::from(self.span()).ignore());
        }
        for el in &self.properties {
            if let Some(res) = el.check_unused_binding_pattern(options, symbol) {
                // has a rest sibling and the rule is configured to
                // ignore variables that have them
                let is_ignorable = options.ignore_rest_siblings && self.rest.is_some();
                return Some(res | is_ignorable);
            }
        }
        return self
            .rest
            .as_ref()
            .and_then(|rest| rest.target.check_unused_binding_pattern(options, symbol));
    }
}

impl<'a> CheckBinding<'a> for AssignmentTargetMaybeDefault<'a> {
    fn check_unused_binding_pattern(
        &self,
        options: &NoUnusedVarsOptions,
        symbol: &Symbol<'_, 'a>,
    ) -> Option<UnusedBindingResult> {
        match self {
            Self::AssignmentTargetWithDefault(target) => {
                target.binding.check_unused_binding_pattern(options, symbol)
            }
            target @ match_assignment_target!(Self) => {
                let target = target.as_assignment_target().expect("match_assignment_target matched a node that couldn't be converted into an AssignmentTarget");
                target.check_unused_binding_pattern(options, symbol)
            }
        }
    }
}

impl<'a> CheckBinding<'a> for ArrayAssignmentTarget<'a> {
    fn check_unused_binding_pattern(
        &self,
        options: &NoUnusedVarsOptions,
        symbol: &Symbol<'_, 'a>,
    ) -> Option<UnusedBindingResult> {
        for el in &self.elements {
            let Some(el) = el.as_ref() else {
                continue;
            };
            // const [a, _b, c] = arr; console.log(a, b)
            // here, res will contain data for _b, and we want to check if it
            // can be ignored (if it matches destructuredArrayIgnorePattern)
            let res = el.check_unused_binding_pattern(options, symbol).map(|res| {
                let is_ignorable = options
                    .destructured_array_ignore_pattern
                    .as_ref()
                    .is_some_and(|pattern| pattern.is_match(symbol.name()));
                res | is_ignorable
            });

            if res.is_some() {
                return res;
            }
        }
        None
    }
}
impl<'a> CheckBinding<'a> for AssignmentTargetProperty<'a> {
    fn check_unused_binding_pattern(
        &self,
        options: &NoUnusedVarsOptions,
        symbol: &Symbol<'_, 'a>,
    ) -> Option<UnusedBindingResult> {
        // self.binding.check_unused_binding_pattern(options, symbol)
        match self {
            Self::AssignmentTargetPropertyIdentifier(id) => {
                id.binding.check_unused_binding_pattern(options, symbol)
            }
            Self::AssignmentTargetPropertyProperty(prop) => {
                prop.binding.check_unused_binding_pattern(options, symbol)
            }
        }
    }
}

impl<'a> CheckBinding<'a> for IdentifierReference<'a> {
    fn check_unused_binding_pattern(
        &self,
        _options: &NoUnusedVarsOptions,
        symbol: &Symbol<'_, 'a>,
    ) -> Option<UnusedBindingResult> {
        (symbol == self).then(|| UnusedBindingResult::from(self.span()))
    }
}

#[derive(Clone, Copy)]
pub(super) struct BindingContext<'s, 'a> {
    pub options: &'s NoUnusedVarsOptions,
    pub semantic: &'s Semantic<'a>,
    // pub symbol: &'s Symbol<'s, 'a>,
}
impl<'s, 'a> BindingContext<'s, 'a> {
    #[inline]
    pub fn symbol(&self, symbol_id: SymbolId) -> Symbol<'s, 'a> {
        Symbol::new(self.semantic, symbol_id)
    }
    #[inline]
    pub fn has_usages(&self, symbol_id: SymbolId) -> bool {
        self.symbol(symbol_id).has_usages(self.options)
    }
}

pub(super) trait HasAnyUsedBinding<'a> {
    fn has_any_used_binding(&self, ctx: BindingContext<'_, 'a>) -> bool;
}

impl<'a> HasAnyUsedBinding<'a> for BindingPattern<'a> {
    #[inline]
    fn has_any_used_binding(&self, ctx: BindingContext<'_, 'a>) -> bool {
        self.kind.has_any_used_binding(ctx)
    }
}
impl<'a> HasAnyUsedBinding<'a> for BindingPatternKind<'a> {
    fn has_any_used_binding(&self, ctx: BindingContext<'_, 'a>) -> bool {
        match self {
            Self::BindingIdentifier(id) => id.has_any_used_binding(ctx),
            Self::AssignmentPattern(id) => id.left.has_any_used_binding(ctx),
            Self::ObjectPattern(id) => id.has_any_used_binding(ctx),
            Self::ArrayPattern(id) => id.has_any_used_binding(ctx),
        }
    }
}

impl<'a> HasAnyUsedBinding<'a> for BindingIdentifier<'a> {
    fn has_any_used_binding(&self, ctx: BindingContext<'_, 'a>) -> bool {
        self.symbol_id.get().is_some_and(|symbol_id| ctx.has_usages(symbol_id))
    }
}
impl<'a> HasAnyUsedBinding<'a> for ObjectPattern<'a> {
    fn has_any_used_binding(&self, ctx: BindingContext<'_, 'a>) -> bool {
        if ctx.options.ignore_rest_siblings && self.rest.is_some() {
            return true;
        }
        self.properties.iter().any(|p| p.value.has_any_used_binding(ctx))
            || self.rest.as_ref().map_or(false, |rest| rest.argument.has_any_used_binding(ctx))
    }
}
impl<'a> HasAnyUsedBinding<'a> for ArrayPattern<'a> {
    fn has_any_used_binding(&self, ctx: BindingContext<'_, 'a>) -> bool {
        self.elements.iter().flatten().any(|el| {
            // if the destructured element is ignored, it is considered used
            el.get_identifier().is_some_and(|name| ctx.options.is_ignored_array_destructured(&name))
                || el.has_any_used_binding(ctx)
        }) || self.rest.as_ref().map_or(false, |rest| rest.argument.has_any_used_binding(ctx))
    }
}
