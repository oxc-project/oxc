use oxc_allocator::{Box, Vec};
use oxc_ast::ast::*;
use oxc_diagnostics::Result;

use crate::lexer::Kind;
use crate::list::{NormalList, SeparatedList};
use crate::Parser;

pub struct TSEnumMemberList<'a> {
    pub members: Vec<'a, TSEnumMember<'a>>,
}

impl<'a> SeparatedList<'a> for TSEnumMemberList<'a> {
    fn new(p: &Parser<'a>) -> Self {
        Self { members: p.ast.new_vec() }
    }

    fn open(&self) -> Kind {
        Kind::LCurly
    }

    fn close(&self) -> Kind {
        Kind::RCurly
    }

    fn parse_element(&mut self, p: &mut Parser<'a>) -> Result<()> {
        let element = p.parse_ts_enum_member()?;
        self.members.push(element);
        Ok(())
    }
}

pub struct TSTupleElementList<'a> {
    pub elements: Vec<'a, TSTupleElement<'a>>,
}

impl<'a> SeparatedList<'a> for TSTupleElementList<'a> {
    fn new(p: &Parser<'a>) -> Self {
        Self { elements: p.ast.new_vec() }
    }

    fn open(&self) -> Kind {
        Kind::LBrack
    }

    fn close(&self) -> Kind {
        Kind::RBrack
    }

    fn parse_element(&mut self, p: &mut Parser<'a>) -> Result<()> {
        let span = p.start_span();
        if p.is_at_named_tuple_element() {
            let _is_rest = p.eat(Kind::Dot3);
            let label = p.parse_identifier_name()?;
            let optional = p.eat(Kind::Question);
            p.expect(Kind::Colon)?;

            let element_type = p.parse_ts_type()?;
            self.elements.push(TSTupleElement::TSNamedTupleMember(p.ast.alloc(
                TSNamedTupleMember { span: p.end_span(span), element_type, label, optional },
            )));

            return Ok(());
        }

        if p.eat(Kind::Dot3) {
            let type_annotation = p.parse_ts_type()?;
            self.elements.push(TSTupleElement::TSRestType(
                p.ast.alloc(TSRestType { span: p.end_span(span), type_annotation }),
            ));
            return Ok(());
        }

        let type_annotation = p.parse_ts_type()?;
        if p.eat(Kind::Question) {
            self.elements.push(TSTupleElement::TSOptionalType(
                p.ast.alloc(TSOptionalType { span: p.end_span(span), type_annotation }),
            ));
        } else {
            self.elements.push(TSTupleElement::TSType(type_annotation));
        }

        Ok(())
    }
}

pub struct TSTypeParameterList<'a> {
    pub params: Vec<'a, Box<'a, TSTypeParameter<'a>>>,
}

impl<'a> SeparatedList<'a> for TSTypeParameterList<'a> {
    fn new(p: &Parser<'a>) -> Self {
        Self { params: p.ast.new_vec() }
    }

    fn open(&self) -> Kind {
        Kind::LAngle
    }

    fn close(&self) -> Kind {
        Kind::RAngle
    }

    fn parse_element(&mut self, p: &mut Parser<'a>) -> Result<()> {
        let param = p.parse_ts_type_parameter()?;
        p.re_lex_ts_r_angle();
        self.params.push(param);
        Ok(())
    }
}

pub struct TSInterfaceOrObjectBodyList<'a> {
    pub body: Vec<'a, TSSignature<'a>>,
}

impl<'a> TSInterfaceOrObjectBodyList<'a> {
    pub fn new(p: &Parser<'a>) -> Self {
        Self { body: p.ast.new_vec() }
    }
}

impl<'a> NormalList<'a> for TSInterfaceOrObjectBodyList<'a> {
    fn open(&self) -> Kind {
        Kind::LCurly
    }

    fn close(&self) -> Kind {
        Kind::RCurly
    }

    fn parse_element(&mut self, p: &mut Parser<'a>) -> Result<()> {
        let property = p.parse_ts_type_signature()?;
        self.body.push(property);
        Ok(())
    }
}

pub struct TSTypeArgumentList<'a> {
    pub params: Vec<'a, TSType<'a>>,
}

impl<'a> SeparatedList<'a> for TSTypeArgumentList<'a> {
    fn new(p: &Parser<'a>) -> Self {
        Self { params: p.ast.new_vec() }
    }

    fn open(&self) -> Kind {
        Kind::LAngle
    }

    fn close(&self) -> Kind {
        Kind::RAngle
    }

    fn parse_element(&mut self, p: &mut Parser<'a>) -> Result<()> {
        let ty = p.parse_ts_type()?;
        p.re_lex_ts_r_angle();
        self.params.push(ty);
        Ok(())
    }
}
