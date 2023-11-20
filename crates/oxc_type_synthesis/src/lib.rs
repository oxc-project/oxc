#![allow(clippy::all, clippy::restriction, clippy::pedantic, clippy::nursery)]

use std::borrow::Cow;

use ezno_checker::{
    source_map::{SourceId as SourceMapSourceId, Span as SourceMapSpan},
    types::properties::PropertyKey,
    ASTImplementation, CheckingData, Environment, ReadFromFS, TypeId,
};

pub use ezno_checker::check_project;

use oxc_ast::ast;
use oxc_span::{GetSpan, Span};

mod expressions;
mod functions;
mod interfaces;
mod statements_and_declarations;
mod types;

pub struct OxcAST;

impl<'a> ASTImplementation for OxcAST {
    type ParseOptions = oxc_span::SourceType;
    type ParseError = oxc_parser::ParserReturn<'a>;
    type DefinitionFile = oxc_parser::ParserReturn<'a>;
    type TypeAnnotation = oxc_ast::ast::TSTypeAnnotation<'a>;
    type TypeParameter = oxc_ast::ast::TSTypeParameter<'a>;

    type Expression = oxc_ast::ast::Expression<'a>;
    type ClassMethod = oxc_ast::ast::Function<'a>;

    type Module = oxc_ast::ast::Program<'a>;
    /// Currently dropping the modules as having problems with lifetimes
    type OwnedModule = ();

    fn module_from_string(
        source_id: SourceMapSourceId,
        string: String,
        options: &Self::ParseOptions,
    ) -> Result<Self::Module, Self::ParseError> {
        todo!()
    }

    fn definition_module_from_string(
        source_id: SourceMapSourceId,
        string: String,
    ) -> Result<Self::DefinitionFile, Self::ParseError> {
        todo!()
    }

    fn synthesise_module<T: ezno_checker::ReadFromFS>(
        module: &Self::Module,
        source_id: SourceMapSourceId,
        root: &mut Environment,
        checking_data: &mut ezno_checker::CheckingData<T, Self>,
    ) {
        todo!()
    }

    fn type_definition_file<T: ezno_checker::ReadFromFS>(
        file: Self::DefinitionFile,
        root: &ezno_checker::RootContext,
        checking_data: &mut CheckingData<T, Self>,
    ) -> (ezno_checker::context::Names, ezno_checker::Facts) {
        todo!()
    }

    fn type_parameter_name(parameter: &Self::TypeParameter) -> &str {
        parameter.name.name.as_str()
    }

    fn synthesise_expression<T: ezno_checker::ReadFromFS>(
        expression: &Self::Expression,
        expected_type: TypeId,
        environment: &mut Environment,
        checking_data: &mut ezno_checker::CheckingData<T, Self>,
    ) -> TypeId {
        crate::expressions::synthesise_expression(
            expression,
            expected_type,
            environment,
            checking_data,
        )
    }

    fn synthesise_type_annotation<T: ezno_checker::ReadFromFS>(
        annotation: &Self::TypeAnnotation,
        environment: &mut Environment,
        checking_data: &mut ezno_checker::CheckingData<T, Self>,
    ) -> TypeId {
        todo!()
    }

    fn expression_position(expression: &Self::Expression) -> ezno_checker::Span {
        oxc_span_to_source_map_span(expression.span())
    }

    // TODO temp throw away
    fn owned_module_from_module(_module: Self::Module) -> Self::OwnedModule {
        ()
    }
}

fn oxc_span_to_source_map_span(span: Span) -> SourceMapSpan {
    SourceMapSpan { start: span.start, end: span.end, source: () }
}

fn property_key_to_type<T: ReadFromFS>(
    key: &ast::PropertyKey,
    environment: &mut Environment,
    checking_data: &mut CheckingData<T, OxcAST>,
) -> PropertyKey<'static> {
    match key {
        ast::PropertyKey::Identifier(ident) => {
            PropertyKey::String(Cow::Owned(ident.name.as_str().to_string()))
        }
        ast::PropertyKey::PrivateIdentifier(item) => {
            checking_data.raise_unimplemented_error(
                "private identifier",
                oxc_span_to_source_map_span(item.span)
                    .with_source(environment.get_environment_id()),
            );

            TypeId::ERROR_TYPE
        }
        ast::PropertyKey::Expression(expr) => PropertyKey::Type(
            expressions::synthesise_expression(expr, TypeId::ANY_TYPE, environment, checking_data),
        ),
    }
}

// Marker type
pub enum PartiallyImplemented<T> {
    Ok(T),
    NotImplemented(&'static str, SourceMapSpan),
}
