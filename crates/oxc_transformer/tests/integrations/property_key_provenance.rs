use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::{Program, StaticMemberExpression, StringLiteral};
use oxc_ast_visit::{Visit, walk};
use oxc_codegen::Codegen;
use oxc_minifier::{ManglePropertiesOptions, PropertyMangler};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{SPAN, SourceType, Span};
use oxc_transformer::{PropertyKeyOrigin, TransformOptions, Transformer};

fn source_span(source: &str, text: &str) -> Span {
    let start = source.find(text).unwrap();
    Span::new(u32::try_from(start).unwrap(), u32::try_from(start + text.len()).unwrap())
}

fn transform<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    source_type: SourceType,
    options: &TransformOptions,
) -> (Program<'a>, oxc_transformer::TransformerReturn) {
    let parsed = Parser::new(allocator, source, source_type).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let mut program = parsed.program;
    let semantic = SemanticBuilder::new().build(&program);
    assert!(semantic.diagnostics.is_empty(), "{:?}", semantic.diagnostics);
    let ret = Transformer::new(allocator, Path::new("test.js"), options)
        .build_with_scoping(semantic.semantic.into_scoping(), &mut program);
    assert!(ret.diagnostics.is_empty(), "{:?}", ret.diagnostics);
    (program, ret)
}

#[derive(Default)]
struct DerivedProperties {
    strings: Vec<(Span, String)>,
    static_members: Vec<(Span, String)>,
}

impl<'a> Visit<'a> for DerivedProperties {
    fn visit_string_literal(&mut self, literal: &StringLiteral<'a>) {
        self.strings.push((literal.span, literal.value.to_string()));
        walk::walk_string_literal(self, literal);
    }

    fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
        self.static_members.push((member.property.span, member.property.name.to_string()));
        walk::walk_static_member_expression(self, member);
    }
}

#[test]
fn transformed_property_strings_keep_quote_provenance() {
    let source = "
        class C { _field = 1; }
        class D extends C {
            static _read = super._super;
            static _write = (super._set = 1);
            static _bump = super._update++;
        }
        obj._power **= 2;
        const { _rest, '_quoted': quoted, ...tail } = obj;
        ({ _assign, ...assignedTail } = obj);
    ";
    let allocator = Allocator::default();
    let options = TransformOptions::from_target("es2015").unwrap();
    let (mut program, ret) = transform(&allocator, source, SourceType::mjs(), &options);
    let provenance = ret.property_key_provenance;

    let field_span = source_span(source, "_field");
    let power_span = source_span(source, "_power");
    let super_span = source_span(source, "_super");
    let set_span = source_span(source, "_set");
    let update_span = source_span(source, "_update");
    let rest_span = source_span(source, "_rest");
    let quoted_span = source_span(source, "'_quoted'");
    let assignment_span = source_span(source, "_assign");
    assert_eq!(provenance[&field_span], PropertyKeyOrigin::Unquoted);
    assert_eq!(provenance[&power_span], PropertyKeyOrigin::Unquoted);
    assert_eq!(provenance[&super_span], PropertyKeyOrigin::Unquoted);
    assert_eq!(provenance[&set_span], PropertyKeyOrigin::Unquoted);
    assert_eq!(provenance[&update_span], PropertyKeyOrigin::Unquoted);
    assert_eq!(provenance[&rest_span], PropertyKeyOrigin::Unquoted);
    assert_eq!(provenance[&quoted_span], PropertyKeyOrigin::Quoted);
    assert_eq!(provenance[&assignment_span], PropertyKeyOrigin::Unquoted);
    assert!(!provenance.contains_key(&SPAN));

    let mut mangler = PropertyMangler::new(ManglePropertiesOptions::from_pattern("^_").unwrap());
    mangler.collect(&program, Some(&provenance));
    mangler.assign();
    let mapping = mangler.mapping().clone();
    mangler.rewrite(&mut program, &allocator, Some(&provenance));

    let mut properties = DerivedProperties::default();
    properties.visit_program(&program);

    let power_target = mapping["_power"].as_str();
    let power_occurrences = properties
        .strings
        .iter()
        .chain(&properties.static_members)
        .filter(|(span, name)| *span == power_span && name == power_target)
        .count();
    let code = Codegen::new().build(&program).code;
    assert_eq!(power_occurrences, 2, "{code}");
    for (span, original) in [
        (field_span, "_field"),
        (super_span, "_super"),
        (set_span, "_set"),
        (update_span, "_update"),
        (rest_span, "_rest"),
        (assignment_span, "_assign"),
    ] {
        let target = mapping[original].as_str();
        assert!(
            properties
                .strings
                .iter()
                .any(|(candidate_span, value)| { *candidate_span == span && value == target })
        );
    }
    assert!(!mapping.contains_key("_quoted"));
    assert!(
        properties.strings.iter().any(|(span, value)| *span == quoted_span && value == "_quoted")
    );
}

#[test]
fn decorator_metadata_labels_are_not_property_provenance() {
    let source = "
        function dec(value: unknown, context: unknown) {}
        class C { @dec _field: string = ''; }
    ";
    let allocator = Allocator::default();
    let mut options = TransformOptions::from_target("es2015").unwrap();
    options.decorator.legacy = true;
    options.decorator.emit_decorator_metadata = true;
    let (program, ret) = transform(&allocator, source, SourceType::ts(), &options);

    assert_eq!(
        ret.property_key_provenance[&source_span(source, "_field")],
        PropertyKeyOrigin::Unquoted,
    );
    assert!(!ret.property_key_provenance.contains_key(&SPAN));

    let mut properties = DerivedProperties::default();
    properties.visit_program(&program);
    for label in ["design:type", "design:paramtypes", "design:returntype"] {
        for (span, _) in properties.strings.iter().filter(|(_, value)| value == label) {
            assert!(!ret.property_key_provenance.contains_key(span));
        }
    }
}

#[test]
fn private_static_exponentiation_keeps_generated_key_provenance() {
    let source = "
        class A {
            static #field = 0;
            static getClass() { return A; }
            update() {
                A.#field **= 2;
                A.getClass().#field **= 3;
            }
        }
    ";
    let allocator = Allocator::default();
    let options = TransformOptions::from_target("es2015").unwrap();
    let (mut program, ret) = transform(&allocator, source, SourceType::mjs(), &options);
    let provenance = ret.property_key_provenance;

    assert_eq!(
        provenance[&source_span(source, "A.getClass().#field")],
        PropertyKeyOrigin::Unquoted,
    );
    assert!(!provenance.contains_key(&SPAN));

    let mut mangler = PropertyMangler::new(ManglePropertiesOptions::from_pattern("^_$").unwrap());
    mangler.collect(&program, Some(&provenance));
    mangler.assign();
    let target = mangler.mapping()["_"].to_string();
    mangler.rewrite(&mut program, &allocator, Some(&provenance));

    let code = Codegen::new().build(&program).code;
    assert_ne!(target, "_");
    assert!(!code.contains("._"), "{code}");
    assert!(!code.contains("[\"_\"]"), "{code}");
    assert!(code.contains(&format!("[\"{target}\"]")), "{code}");
}
