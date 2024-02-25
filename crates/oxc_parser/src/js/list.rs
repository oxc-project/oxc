use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
    Result,
};
use oxc_span::{Atom, CompactString, GetSpan, Span};
use rustc_hash::FxHashMap;

use crate::{
    diagnostics,
    lexer::Kind,
    list::{NormalList, SeparatedList},
    ParserImpl,
};

#[derive(Debug, Error, Diagnostic)]
#[error("Identifier `{0}` has already been declared")]
#[diagnostic()]
struct Redeclaration(
    pub CompactString,
    #[label("`{0}` has already been declared here")] pub Span,
    #[label("It can not be redeclared here")] pub Span,
);

/// ObjectExpression.properties
pub struct ObjectExpressionProperties<'a> {
    pub elements: Vec<'a, ObjectPropertyKind<'a>>,
    pub trailing_comma: Option<Span>,
}

impl<'a> SeparatedList<'a> for ObjectExpressionProperties<'a> {
    fn new(p: &ParserImpl<'a>) -> Self {
        Self { elements: p.ast.new_vec(), trailing_comma: None }
    }

    fn open(&self) -> Kind {
        Kind::LCurly
    }

    fn close(&self) -> Kind {
        Kind::RCurly
    }

    fn parse_element(&mut self, p: &mut ParserImpl<'a>) -> Result<()> {
        let element = match p.cur_kind() {
            Kind::Dot3 => p.parse_spread_element().map(ObjectPropertyKind::SpreadProperty),
            _ => p.parse_property_definition().map(ObjectPropertyKind::ObjectProperty),
        }?;

        if p.at(Kind::Comma) && p.peek_at(self.close()) {
            self.trailing_comma = Some(p.end_span(p.start_span()));
        }

        self.elements.push(element);
        Ok(())
    }
}

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
                p.error(diagnostics::InvalidBindingRestElement(rest.argument.span()));
            }
            if let Some(r) = self.rest.replace(rest) {
                p.error(diagnostics::BindingRestElementLast(r.span));
            }
        } else {
            let prop = p.parse_binding_property()?;
            self.elements.push(prop);
        }
        Ok(())
    }
}

/// ArrayExpression.elements
pub struct ArrayExpressionList<'a> {
    pub elements: Vec<'a, ArrayExpressionElement<'a>>,
    pub trailing_comma: Option<Span>,
}

impl<'a> SeparatedList<'a> for ArrayExpressionList<'a> {
    fn new(p: &ParserImpl<'a>) -> Self {
        Self { elements: p.ast.new_vec(), trailing_comma: None }
    }

    fn open(&self) -> Kind {
        Kind::LBrack
    }

    fn close(&self) -> Kind {
        Kind::RBrack
    }

    fn parse_element(&mut self, p: &mut ParserImpl<'a>) -> Result<()> {
        let element = match p.cur_kind() {
            Kind::Comma => Ok(p.parse_elision()),
            Kind::Dot3 => p.parse_spread_element().map(ArrayExpressionElement::SpreadElement),
            _ => p.parse_assignment_expression_base().map(ArrayExpressionElement::Expression),
        };

        if p.at(Kind::Comma) && p.peek_at(self.close()) {
            self.trailing_comma = Some(p.end_span(p.start_span()));
        }

        self.elements.push(element?);
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
                    p.error(diagnostics::BindingRestElementLast(r.span));
                }
            }
            _ => {
                let element = p.parse_binding_pattern()?;
                self.elements.push(Some(element));
            }
        }
        Ok(())
    }
}

/// Section 13.3 Arguments for `CallExpression`, `NewExpression`
pub struct CallArguments<'a> {
    pub elements: Vec<'a, Argument<'a>>,
    pub rest_element_with_trilling_comma: Option<Span>,
}

impl<'a> SeparatedList<'a> for CallArguments<'a> {
    fn new(p: &ParserImpl<'a>) -> Self {
        Self { elements: p.ast.new_vec(), rest_element_with_trilling_comma: None }
    }

    fn open(&self) -> Kind {
        Kind::LParen
    }

    fn close(&self) -> Kind {
        Kind::RParen
    }

    fn parse_element(&mut self, p: &mut ParserImpl<'a>) -> Result<()> {
        let element = if p.at(Kind::Dot3) {
            let result = p.parse_spread_element().map(Argument::SpreadElement);
            if p.at(Kind::Comma) {
                if let Ok(Argument::SpreadElement(argument)) = &result {
                    self.rest_element_with_trilling_comma = Some(argument.span);
                }
            }
            result
        } else {
            p.parse_assignment_expression_base().map(Argument::Expression)
        };
        self.elements.push(element?);
        Ok(())
    }
}

pub struct SequenceExpressionList<'a> {
    pub elements: Vec<'a, Expression<'a>>,
}

impl<'a> SeparatedList<'a> for SequenceExpressionList<'a> {
    fn new(p: &ParserImpl<'a>) -> Self {
        Self { elements: p.ast.new_vec() }
    }

    fn open(&self) -> Kind {
        Kind::LParen
    }

    fn close(&self) -> Kind {
        Kind::RParen
    }

    // read everything as expression and map to it to either
    // ParenthesizedExpression or ArrowFormalParameters later
    fn parse_element(&mut self, p: &mut ParserImpl<'a>) -> Result<()> {
        let element = p.parse_assignment_expression_base()?;
        self.elements.push(element);
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
        let readonly = modifiers.readonly();

        match p.cur_kind() {
            Kind::This if p.ts_enabled() => {
                let this_parameter = p.parse_ts_this_parameter()?;
                self.this_param.replace(this_parameter);
            }
            Kind::Dot3 => {
                let rest = p.parse_rest_element()?;
                if let Some(r) = self.rest.replace(rest) {
                    p.error(diagnostics::RestParameterLast(r.span));
                }
            }
            _ => {
                let pattern = p.parse_binding_pattern()?;
                let decorators = p.state.consume_decorators();
                let formal_parameter = p.ast.formal_parameter(
                    p.end_span(span),
                    pattern,
                    accessibility,
                    readonly,
                    decorators,
                );
                self.elements.push(formal_parameter);
            }
        }

        Ok(())
    }
}

/// [Assert Entries](https://tc39.es/proposal-import-assertions)
pub struct AssertEntries<'a> {
    pub elements: Vec<'a, ImportAttribute<'a>>,
    keys: FxHashMap<Atom<'a>, Span>,
}

impl<'a> SeparatedList<'a> for AssertEntries<'a> {
    fn new(p: &ParserImpl<'a>) -> Self {
        Self { elements: p.ast.new_vec(), keys: FxHashMap::default() }
    }

    fn open(&self) -> Kind {
        Kind::LCurly
    }

    fn close(&self) -> Kind {
        Kind::RCurly
    }

    fn parse_element(&mut self, p: &mut ParserImpl<'a>) -> Result<()> {
        let span = p.start_span();
        let key = match p.cur_kind() {
            Kind::Str => ImportAttributeKey::StringLiteral(p.parse_literal_string()?),
            _ => ImportAttributeKey::Identifier(p.parse_identifier_name()?),
        };

        if let Some(old_span) = self.keys.get(&key.as_atom()) {
            p.error(Redeclaration(key.as_atom().into_compact_string(), *old_span, key.span()));
        } else {
            self.keys.insert(key.as_atom(), key.span());
        }

        p.expect(Kind::Colon)?;
        let value = p.parse_literal_string()?;
        let element = ImportAttribute { span: p.end_span(span), key, value };
        self.elements.push(element);
        Ok(())
    }
}

pub struct ExportNamedSpecifiers<'a> {
    pub elements: Vec<'a, ExportSpecifier<'a>>,
}

impl<'a> SeparatedList<'a> for ExportNamedSpecifiers<'a> {
    fn new(p: &ParserImpl<'a>) -> Self {
        Self { elements: p.ast.new_vec() }
    }

    fn open(&self) -> Kind {
        Kind::LCurly
    }

    fn close(&self) -> Kind {
        Kind::RCurly
    }

    fn parse_element(&mut self, p: &mut ParserImpl<'a>) -> Result<()> {
        let specifier_span = p.start_span();
        let peek_kind = p.peek_kind();

        // export { type}              // name: `type`
        // export { type type }        // name: `type`    type-export: `true`
        // export { type as }          // name: `as`      type-export: `true`
        // export { type as as }       // name: `type`    type-export: `false` (aliased to `as`)
        // export { type as as as }    // name: `as`      type-export: `true`, aliased to `as`
        let mut export_kind = ImportOrExportKind::Value;
        if p.ts_enabled() && p.at(Kind::Type) {
            if p.peek_at(Kind::As) {
                if p.nth_at(2, Kind::As) {
                    if p.nth_at(3, Kind::Str) || p.nth_kind(3).is_identifier_name() {
                        export_kind = ImportOrExportKind::Type;
                    }
                } else if !(p.nth_at(2, Kind::Str) || p.nth_kind(2).is_identifier_name()) {
                    export_kind = ImportOrExportKind::Type;
                }
            } else if (matches!(peek_kind, Kind::Str) || peek_kind.is_identifier_name()) {
                export_kind = ImportOrExportKind::Type;
            }
        }

        if export_kind == ImportOrExportKind::Type {
            p.bump_any();
        }

        let local = p.parse_module_export_name()?;
        let exported = if p.eat(Kind::As) { p.parse_module_export_name()? } else { local.clone() };
        let element =
            ExportSpecifier { span: p.end_span(specifier_span), local, exported, export_kind };
        self.elements.push(element);
        Ok(())
    }
}

pub struct ClassElements<'a> {
    pub elements: Vec<'a, ClassElement<'a>>,
}

impl<'a> ClassElements<'a> {
    pub(crate) fn new(p: &ParserImpl<'a>) -> Self {
        Self { elements: p.ast.new_vec() }
    }
}

impl<'a> NormalList<'a> for ClassElements<'a> {
    fn open(&self) -> Kind {
        Kind::LCurly
    }

    fn close(&self) -> Kind {
        Kind::RCurly
    }

    fn parse_element(&mut self, p: &mut ParserImpl<'a>) -> Result<()> {
        // skip empty class element `;`
        while p.at(Kind::Semicolon) {
            p.bump_any();
        }
        if p.at(self.close()) {
            return Ok(());
        }
        let element = p.parse_class_element()?;

        self.elements.push(element);
        Ok(())
    }
}

pub struct SwitchCases<'a> {
    pub elements: Vec<'a, SwitchCase<'a>>,
}

impl<'a> SwitchCases<'a> {
    pub(crate) fn new(p: &ParserImpl<'a>) -> Self {
        Self { elements: p.ast.new_vec() }
    }
}

impl<'a> NormalList<'a> for SwitchCases<'a> {
    fn open(&self) -> Kind {
        Kind::LCurly
    }

    fn close(&self) -> Kind {
        Kind::RCurly
    }

    fn parse_element(&mut self, p: &mut ParserImpl<'a>) -> Result<()> {
        let element = p.parse_switch_case()?;
        self.elements.push(element);
        Ok(())
    }
}

pub struct ImportSpecifierList<'a> {
    pub import_specifiers: Vec<'a, ImportDeclarationSpecifier<'a>>,
}

impl<'a> SeparatedList<'a> for ImportSpecifierList<'a> {
    fn new(p: &ParserImpl<'a>) -> Self {
        Self { import_specifiers: p.ast.new_vec() }
    }

    fn open(&self) -> Kind {
        Kind::LCurly
    }

    fn close(&self) -> Kind {
        Kind::RCurly
    }

    fn parse_element(&mut self, p: &mut ParserImpl<'a>) -> Result<()> {
        let import_specifier = p.parse_import_specifier()?;
        let specifier = ImportDeclarationSpecifier::ImportSpecifier(import_specifier);
        self.import_specifiers.push(specifier);
        Ok(())
    }
}
