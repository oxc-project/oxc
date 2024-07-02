use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_diagnostics::Result;
use oxc_span::GetSpan;

use crate::{diagnostics, lexer::Kind, list::SeparatedList, modifiers::ModifierFlags, ParserImpl};

/// ObjectPattern.properties
pub struct ObjectPatternProperties<'a> {
    pub elements: Vec<'a, BindingProperty<'a>>,
    pub rest: Option<oxc_allocator::Box<'a, BindingRestElement<'a>>>,
}

impl<'a> SeparatedList<'a> for ObjectPatternProperties<'a> {
    fn new(p: &ParserImpl<'a>) -> Self {
        Self { elements: p.ast.new_vec(), rest: None }
    }

    fn open(&self) -> Kind {
        Kind::LCurly
    }

    fn close(&self) -> Kind {
        Kind::RCurly
    }

    fn parse_element(&mut self, p: &mut ParserImpl<'a>) -> Result<()> {
        if p.cur_kind() == Kind::Dot3 {
            let rest = p.parse_rest_element()?;
            if !matches!(&rest.argument.kind, BindingPatternKind::BindingIdentifier(_)) {
                p.error(diagnostics::invalid_binding_rest_element(rest.argument.span()));
            }
            if let Some(r) = self.rest.replace(rest) {
                p.error(diagnostics::binding_rest_element_last(r.span));
            }
        } else {
            let prop = p.parse_binding_property()?;
            self.elements.push(prop);
        }
        Ok(())
    }
}

/// ArrayPattern.elements
pub struct ArrayPatternList<'a> {
    pub elements: Vec<'a, Option<BindingPattern<'a>>>,
    pub rest: Option<oxc_allocator::Box<'a, BindingRestElement<'a>>>,
}

impl<'a> SeparatedList<'a> for ArrayPatternList<'a> {
    fn new(p: &ParserImpl<'a>) -> Self {
        Self { elements: p.ast.new_vec(), rest: None }
    }

    fn open(&self) -> Kind {
        Kind::LBrack
    }

    fn close(&self) -> Kind {
        Kind::RBrack
    }

    fn parse_element(&mut self, p: &mut ParserImpl<'a>) -> Result<()> {
        match p.cur_kind() {
            Kind::Comma => {
                self.elements.push(None);
            }
            Kind::Dot3 => {
                let rest = p.parse_rest_element()?;
                if let Some(r) = self.rest.replace(rest) {
                    p.error(diagnostics::binding_rest_element_last(r.span));
                }
            }
            _ => {
                let element = p.parse_binding_pattern_with_initializer()?;
                self.elements.push(Some(element));
            }
        }
        Ok(())
    }
}

/// Function Parameters
pub struct FormalParameterList<'a> {
    pub elements: Vec<'a, FormalParameter<'a>>,
    pub rest: Option<oxc_allocator::Box<'a, BindingRestElement<'a>>>,
    pub this_param: Option<TSThisParameter<'a>>,
}

impl<'a> SeparatedList<'a> for FormalParameterList<'a> {
    fn new(p: &ParserImpl<'a>) -> Self {
        Self { elements: p.ast.new_vec(), rest: None, this_param: None }
    }

    fn open(&self) -> Kind {
        Kind::LParen
    }

    fn close(&self) -> Kind {
        Kind::RParen
    }

    // Section 15.1 Parameter Lists
    fn parse_element(&mut self, p: &mut ParserImpl<'a>) -> Result<()> {
        let span = p.start_span();
        p.eat_decorators()?;

        let modifiers = p.parse_class_element_modifiers(true);
        let accessibility = modifiers.accessibility();
        let readonly = modifiers.contains_readonly();
        let r#override = modifiers.contains_override();
        p.verify_modifiers(
            &modifiers,
            ModifierFlags::ACCESSIBILITY
                .union(ModifierFlags::READONLY)
                .union(ModifierFlags::OVERRIDE),
            diagnostics::cannot_appear_on_a_parameter,
        );

        match p.cur_kind() {
            Kind::This if p.ts_enabled() => {
                let this_parameter = p.parse_ts_this_parameter()?;
                self.this_param.replace(this_parameter);
            }
            Kind::Dot3 => {
                let rest = p.parse_rest_element()?;
                if let Some(r) = self.rest.replace(rest) {
                    p.error(diagnostics::rest_parameter_last(r.span));
                }
            }
            _ => {
                let pattern = p.parse_binding_pattern_with_initializer()?;
                let decorators = p.consume_decorators();
                let formal_parameter = p.ast.formal_parameter(
                    p.end_span(span),
                    pattern,
                    accessibility,
                    readonly,
                    r#override,
                    decorators,
                );
                self.elements.push(formal_parameter);
            }
        }

        Ok(())
    }
}
