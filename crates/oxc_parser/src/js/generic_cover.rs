//! Shared representation of a simple generic prefix before an async argument/parameter list.

use oxc_allocator::ArenaBox;
use oxc_ast::ast::*;
use oxc_span::{FileExtension, Span};
use oxc_str::Ident;

use super::Tristate;
use crate::{ParserConfig as Config, ParserImpl, diagnostics, lexer::Kind};

/// A parsed `<T>` or `<T,>` whose interpretation is still undecided.
/// No type-parameter or type-reference AST has been allocated yet.
pub(super) struct SimpleGenericPrefix<'a> {
    span: Span,
    name_span: Span,
    name: Ident<'a>,
    trailing_comma: bool,
}

impl SimpleGenericPrefix<'_> {
    /// Preserve the existing JSX override, including its error-recovery commitment.
    /// In TSX, `<T>` cannot introduce an arrow and `<T,>` must introduce one.
    pub(super) fn arrow_kind(&self, is_jsx: bool) -> Tristate {
        if !is_jsx {
            Tristate::Maybe
        } else if self.trailing_comma {
            Tristate::True
        } else {
            Tristate::False
        }
    }
}

impl<'a, C: Config> ParserImpl<'a, C> {
    /// Consume a simple generic prefix, leaving the opening `(` current.
    ///
    /// The caller has already consumed `async` as an identifier. It must separately
    /// preserve the no-line-terminator restriction and reject escaped `async` only
    /// if the list is committed to an arrow. Other generic forms remain untouched.
    pub(super) fn try_parse_simple_generic_cover_prefix(
        &mut self,
    ) -> Option<SimpleGenericPrefix<'a>> {
        if !self.is_ts || !self.at(Kind::LAngle) || self.has_fatal_error() {
            return None;
        }
        let checkpoint = self.checkpoint();
        let start = self.cur_start();
        self.bump_any();
        if !self.at(Kind::Ident) {
            self.rewind(checkpoint);
            return None;
        }
        let (name_span, name) = self.parse_identifier_kind(Kind::Ident);
        let trailing_comma = self.eat(Kind::Comma);
        if !self.eat(Kind::RAngle) || !self.at(Kind::LParen) {
            self.rewind(checkpoint);
            return None;
        }
        Some(SimpleGenericPrefix { span: self.end_span(start), name_span, name, trailing_comma })
    }

    /// Finalize the retained prefix only after the list is committed to an arrow.
    pub(super) fn simple_generic_cover_type_parameters(
        &mut self,
        prefix: &SimpleGenericPrefix<'a>,
    ) -> ArenaBox<'a, TSTypeParameterDeclaration<'a>> {
        let name = BindingIdentifier::new(prefix.name_span, prefix.name, self);
        self.check_reserved_type_name(&name, "Type parameter");
        if !prefix.trailing_comma
            && matches!(self.source_type.extension(), Some(FileExtension::Mts | FileExtension::Cts))
        {
            self.error(diagnostics::jsx_type_parameter_in_mts_cts(prefix.name_span));
        }
        let parameter =
            TSTypeParameter::new(prefix.name_span, name, None, None, false, false, false, self);
        TSTypeParameterDeclaration::boxed(prefix.span, [parameter], self)
    }

    /// Finalize the retained prefix only after the list is committed to a call.
    pub(super) fn simple_generic_cover_type_arguments(
        &self,
        prefix: &SimpleGenericPrefix<'a>,
    ) -> ArenaBox<'a, TSTypeParameterInstantiation<'a>> {
        let name = TSTypeName::new_identifier_reference(prefix.name_span, prefix.name, self);
        let argument = TSType::new_ts_type_reference(prefix.name_span, name, None, self);
        TSTypeParameterInstantiation::boxed(prefix.span, [argument], self)
    }
}
