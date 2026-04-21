/// React Pure Annotations
///
/// Adds `/*#__PURE__*/` annotations to calls to specific React top-level methods,
/// so that terser and other minifiers can safely remove them during dead code elimination.
///
/// Port of [@babel/plugin-transform-react-pure-annotations](https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-react-pure-annotations)
use oxc_ast::ast::*;
use oxc_semantic::SymbolId;
use oxc_traverse::Traverse;

use crate::{context::TraverseCtx, state::TransformState};

#[derive(Copy, Clone)]
#[repr(u8)]
enum PureMethod {
    // react
    CloneElement = 0,
    CreateContext = 1,
    CreateElement = 2,
    CreateFactory = 3,
    CreateRef = 4,
    ForwardRef = 5,
    IsValidElement = 6,
    Memo = 7,
    Lazy = 8,
    // react-dom
    CreatePortal = 9,
}

const PURE_METHOD_COUNT: usize = 10;

impl PureMethod {
    /// Convert string to [`PureMethod`] for `react` module.
    fn from_react(name: &str) -> Option<Self> {
        match name {
            "cloneElement" => Some(Self::CloneElement),
            "createContext" => Some(Self::CreateContext),
            "createElement" => Some(Self::CreateElement),
            "createFactory" => Some(Self::CreateFactory),
            "createRef" => Some(Self::CreateRef),
            "forwardRef" => Some(Self::ForwardRef),
            "isValidElement" => Some(Self::IsValidElement),
            "memo" => Some(Self::Memo),
            "lazy" => Some(Self::Lazy),
            _ => None,
        }
    }

    /// Convert string to [`PureMethod`] for `react-dom` module.
    fn from_react_dom(name: &str) -> Option<Self> {
        match name {
            "createPortal" => Some(Self::CreatePortal),
            _ => None,
        }
    }
}

/// Tracks symbol IDs for imports from `react` and `react-dom`.
#[derive(Default)]
struct PureCallBindings {
    /// Named imports matching pure methods, e.g. `import { forwardRef } from 'react'`.
    named: [Option<SymbolId>; PURE_METHOD_COUNT],
    /// `import React from 'react'`
    react_default: Option<SymbolId>,
    /// `import * as React from 'react'`
    react_namespace: Option<SymbolId>,
    /// `import ReactDOM from 'react-dom'`
    react_dom_default: Option<SymbolId>,
    /// `import * as ReactDOM from 'react-dom'`
    react_dom_namespace: Option<SymbolId>,
}

impl PureCallBindings {
    fn set_named_symbol_id(&mut self, method: PureMethod, symbol_id: SymbolId) {
        self.named[method as usize] = Some(symbol_id);
    }
}

pub struct ReactPureAnnotations {
    bindings: PureCallBindings,
}

impl ReactPureAnnotations {
    pub fn new() -> Self {
        Self { bindings: PureCallBindings::default() }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for ReactPureAnnotations {
    fn enter_program(&mut self, program: &mut Program<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.collect_bindings(program);
    }

    fn enter_call_expression(&mut self, call: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.is_pure_call(call, ctx) {
            call.pure = true;
        }
    }
}

impl ReactPureAnnotations {
    /// Scans import declarations and stores symbol IDs for known pure React calls.
    fn collect_bindings(&mut self, program: &Program<'_>) {
        for statement in &program.body {
            let Statement::ImportDeclaration(import) = statement else { continue };
            let Some(specifiers) = &import.specifiers else { continue };

            let source = import.source.value.as_str();
            let is_react = source == "react";
            let is_react_dom = source == "react-dom";
            if !is_react && !is_react_dom {
                continue;
            }

            for specifier in specifiers {
                match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                        let name = specifier.imported.name();
                        let method = if is_react {
                            PureMethod::from_react(name.as_str())
                        } else {
                            PureMethod::from_react_dom(name.as_str())
                        };
                        if let Some(method) = method {
                            self.bindings.set_named_symbol_id(method, specifier.local.symbol_id());
                        }
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                        let target = if is_react {
                            &mut self.bindings.react_default
                        } else {
                            &mut self.bindings.react_dom_default
                        };
                        *target = Some(specifier.local.symbol_id());
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        let target = if is_react {
                            &mut self.bindings.react_namespace
                        } else {
                            &mut self.bindings.react_dom_namespace
                        };
                        *target = Some(specifier.local.symbol_id());
                    }
                }
            }
        }
    }

    /// Checks if the call expression is a pure React call.
    fn is_pure_call(&self, call: &CallExpression<'_>, ctx: &TraverseCtx<'_>) -> bool {
        match &call.callee {
            // `forwardRef(...)` — named import
            Expression::Identifier(ident) => self.is_named_pure_reference(ident, ctx),
            // `React.forwardRef(...)` — default or namespace import
            Expression::StaticMemberExpression(member) => {
                self.is_member_pure_reference(member, ctx)
            }
            _ => false,
        }
    }

    /// Checks if the identifier references a named import of a known pure method.
    fn is_named_pure_reference(
        &self,
        ident: &IdentifierReference<'_>,
        ctx: &TraverseCtx<'_>,
    ) -> bool {
        let Some(ref_symbol_id) = ctx.scoping().get_reference(ident.reference_id()).symbol_id()
        else {
            return false;
        };
        self.bindings.named.iter().any(|binding| binding.is_some_and(|id| id == ref_symbol_id))
    }

    /// Checks if a member expression like `React.forwardRef` references a default/namespace
    /// import and the property is a known pure method.
    fn is_member_pure_reference(
        &self,
        member: &StaticMemberExpression<'_>,
        ctx: &TraverseCtx<'_>,
    ) -> bool {
        let Expression::Identifier(object) = &member.object else {
            return false;
        };

        let property = member.property.name.as_str();

        if (Self::references_binding(self.bindings.react_default, object, ctx)
            || Self::references_binding(self.bindings.react_namespace, object, ctx))
            && PureMethod::from_react(property).is_some()
        {
            return true;
        }

        if (Self::references_binding(self.bindings.react_dom_default, object, ctx)
            || Self::references_binding(self.bindings.react_dom_namespace, object, ctx))
            && PureMethod::from_react_dom(property).is_some()
        {
            return true;
        }

        false
    }

    /// Checks if the identifier is a reference to the given binding.
    fn references_binding(
        binding: Option<SymbolId>,
        ident: &IdentifierReference<'_>,
        ctx: &TraverseCtx<'_>,
    ) -> bool {
        binding.is_some_and(|symbol_id| {
            ctx.scoping()
                .get_reference(ident.reference_id())
                .symbol_id()
                .is_some_and(|ref_symbol_id| ref_symbol_id == symbol_id)
        })
    }
}
