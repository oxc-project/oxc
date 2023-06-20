#![allow(clippy::all, clippy::restriction, clippy::pedantic, clippy::nursery)]

use ezno_checker::{
    events::Event, types::TypeStore, CheckingData, Environment, FSResolver, Root, Scope,
    Span as SourceMapSpan, TypeCheckSettings, TypeId, TypeMappings,
};
pub use ezno_checker::{
    Diagnostic, DiagnosticKind, DiagnosticsContainer, SourceId as EznoSourceId, Span as EznoSpan,
};
use oxc_ast::ast;
use oxc_span::Span;
use statements_and_declarations::synthesize_statements;

mod expressions;
mod functions;
mod interfaces;
mod statements_and_declarations;
mod types;

pub fn synthesize_program<T: FSResolver>(
    program: &ast::Program,
    resolver: T,
) -> (DiagnosticsContainer, Vec<Event>, TypeStore, TypeMappings, Root) {
    let default_settings = TypeCheckSettings::default();
    let mut checking_data = CheckingData::new(default_settings, &resolver);

    let mut root = Root::new_with_primitive_references_and_ezno_magic();

    let (_, stuff, _) = root.new_lexical_environment_fold_into_parent(
        Scope::Block {},
        &mut checking_data,
        |environment, checking_data| {
            synthesize_statements(&program.body, environment, checking_data);
        },
    );

    (
        checking_data.diagnostics_container,
        stuff.expect("block will always return events").0,
        checking_data.types,
        checking_data.type_mappings,
        root,
    )
}

fn oxc_span_to_source_map_span(span: Span) -> SourceMapSpan {
    SourceMapSpan {
        start: span.start,
        end: span.end,
        // TODO!!
        source_id: ezno_checker::SourceId::NULL,
    }
}

fn property_key_to_type<T: FSResolver>(
    key: &ast::PropertyKey,
    environment: &mut Environment,
    checking_data: &mut CheckingData<T>,
) -> TypeId {
    match key {
        ast::PropertyKey::Identifier(ident) => checking_data
            .types
            .new_constant_type(ezno_checker::Constant::String(ident.name.as_str().to_string())),
        ast::PropertyKey::PrivateIdentifier(item) => {
            checking_data.raise_unimplemented_error(
                "private identifier",
                oxc_span_to_source_map_span(item.span),
            );

            TypeId::ERROR_TYPE
        }
        ast::PropertyKey::Expression(expr) => {
            // TODO make key into key
            expressions::synthesize_expression(expr, environment, checking_data)
        }
    }
}

// Marker type
pub enum PartiallyImplemented<T> {
    Ok(T),
    NotImplemented(&'static str, EznoSpan),
}
