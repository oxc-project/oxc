#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
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
            .map(|rest| rest.check_unused_binding_pattern(options, symbol))
            .flatten();
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
            let res = el.check_unused_binding_pattern(options, symbol);
            if res.is_some() {
                return res;
            }
        }
        None
    }
}
