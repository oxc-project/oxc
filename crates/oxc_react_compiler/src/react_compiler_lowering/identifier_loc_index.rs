//! Builds an index mapping identifier references and declarations to source
//! locations, keyed by semantic `ReferenceId` / `SymbolId`.
//!
//! Walks the function's oxc AST to collect resolved identifier references and
//! binding declarations (including identifiers inside TS type annotations).
//!
//! This is a translation of the original immutable `IdentifierLocVisitor`, which
//! was driven by the in-tree `AstWalker`/`Visitor`
//! (`crate::react_compiler_ast::visitor`). That walker deliberately visited only
//! a NARROW set of identifier positions, and TS type identifiers came from
//! `collect_type_idents`, which collected only `IdentifierReference` /
//! `IdentifierName` (never `BindingIdentifier`). The overrides below restrict the
//! oxc walk to match those positions instead of relying on oxc's default
//! full-AST traversal. The traversal records:
//!
//! * every identifier reference and (declaration-map) binding identifier
//! * function / class declaration & expression names, into the declaration map
//! * JSX element-name identifier references, carrying the enclosing
//!   `JSXOpeningElement`'s span as `opening_element_span`
//! * identifiers inside TS type subtrees → `in_type_annotation = true`
//!
//! Positions deliberately NOT recorded, matching the original walker:
//!
//! * non-computed member property names (`a.b` → `b`)
//! * non-computed object / class member keys (`{ a: 1 }` → `a`)
//! * JSX attribute names and JSX closing-element names
//! * label identifiers (`LabeledStatement` / `break` / `continue` targets)
//! * class `super_class` (`extends Foo`) and class member bodies
//! * TS declaration statements (type alias / interface / enum / module)
//! * `BindingIdentifier`s inside TS type subtrees (e.g. type-parameter names)

use rustc_hash::FxHashMap;

use oxc_ast::ast as oxc;
use oxc_ast_visit::Visit;

use crate::react_compiler_hir::Span;
use crate::scope::{ReferenceId, SymbolId};

use crate::react_compiler_lowering::FunctionNode;

/// Source location data for a resolved identifier reference.
pub struct IdentifierLocEntry {
    pub span: Span,
    /// For JSX element-name identifiers, the enclosing `JSXOpeningElement`'s
    /// span (which spans the full tag).
    pub opening_element_span: Option<Span>,
    /// True if this identifier sits inside a type annotation subtree
    /// (TypeAnnotation/TSTypeAnnotation/TypeAlias/TSTypeAliasDeclaration).
    /// `gather_captured_context` skips these to match the TS
    /// gatherCapturedContext traverse, which skips those subtrees; the
    /// hoisting analysis and FindContextIdentifiers do NOT skip them in TS.
    pub in_type_annotation: bool,
}

impl IdentifierLocEntry {
    /// True for JSX element-name identifiers. The TS hoisting analysis does
    /// not traverse JSX elements, so hoisting skips these references.
    pub fn is_jsx(&self) -> bool {
        self.opening_element_span.is_some()
    }
}

/// Identifier locations for a function's AST, keyed by semantic identity.
///
/// Only identifiers the original Babel walker visited are recorded, so
/// membership in these maps is itself meaningful: a symbol has a declaration
/// span here iff its declaration identifier sits in a position the walker
/// recorded (inside the compiled function, outside type subtrees).
#[derive(Default)]
pub struct IdentifierLocIndex {
    /// Resolved identifier references, keyed by their `reference_id` cell.
    refs: FxHashMap<ReferenceId, IdentifierLocEntry>,
    /// Declaration (binding) identifier spans, keyed by their `symbol_id` cell.
    /// First declaration wins for redeclared symbols.
    decl_spans: FxHashMap<SymbolId, Span>,
}

impl IdentifierLocIndex {
    pub fn reference(&self, reference_id: ReferenceId) -> Option<&IdentifierLocEntry> {
        self.refs.get(&reference_id)
    }

    pub fn declaration_span(&self, symbol_id: SymbolId) -> Option<Span> {
        self.decl_spans.get(&symbol_id).copied()
    }
}

struct IdentifierLocVisitor {
    index: IdentifierLocIndex,
    /// Tracks the current JSXOpeningElement's span while walking its name.
    current_opening_element_span: Option<Span>,
    /// Depth of TS type subtrees currently being walked. Identifiers recorded
    /// while this is non-zero get `in_type_annotation = true`.
    type_depth: u32,
}

impl IdentifierLocVisitor {
    /// `current_opening_element_span` is set only while walking a JSX
    /// element name, so it doubles as the is-JSX signal.
    fn record_reference(&mut self, ident: &oxc::IdentifierReference<'_>) {
        let Some(reference_id) = ident.reference_id.get() else { return };
        self.index.refs.entry(reference_id).or_insert(IdentifierLocEntry {
            span: ident.span,
            opening_element_span: self.current_opening_element_span,
            in_type_annotation: self.type_depth > 0,
        });
    }

    fn record_declaration(&mut self, ident: &oxc::BindingIdentifier<'_>) {
        let Some(symbol_id) = ident.symbol_id.get() else { return };
        self.index.decl_spans.entry(symbol_id).or_insert(ident.span);
    }

    /// Record the JSX element name identifiers (and only those) while the
    /// `current_opening_element_span` is set, mirroring the original
    /// `walk_jsx_element_name` / `walk_jsx_member_expression`. Lowercase tag
    /// names, `this`, and member-property parts carry no reference and are
    /// never looked up, so only `IdentifierReference` names are recorded.
    fn record_jsx_element_name<'a>(&mut self, name: &oxc::JSXElementName<'a>) {
        match name {
            oxc::JSXElementName::IdentifierReference(id) => self.record_reference(id),
            oxc::JSXElementName::MemberExpression(m) => self.record_jsx_member_expression(m),
            oxc::JSXElementName::Identifier(_)
            | oxc::JSXElementName::ThisExpression(_)
            | oxc::JSXElementName::NamespacedName(_) => {}
        }
    }

    fn record_jsx_member_expression<'a>(&mut self, expr: &oxc::JSXMemberExpression<'a>) {
        match &expr.object {
            oxc::JSXMemberExpressionObject::IdentifierReference(id) => {
                self.record_reference(id);
            }
            oxc::JSXMemberExpressionObject::ThisExpression(_) => {}
            oxc::JSXMemberExpressionObject::MemberExpression(inner) => {
                self.record_jsx_member_expression(inner);
            }
        }
    }
}

impl<'a> Visit<'a> for IdentifierLocVisitor {
    fn visit_identifier_reference(&mut self, it: &oxc::IdentifierReference<'a>) {
        self.record_reference(it);
    }

    fn visit_binding_identifier(&mut self, it: &oxc::BindingIdentifier<'a>) {
        // `collect_type_idents` only collected IdentifierReference / IdentifierName,
        // never BindingIdentifier, so type-parameter declaration names (`<T>`) and
        // other binding positions inside type subtrees must not be recorded.
        if self.type_depth > 0 {
            return;
        }
        self.record_declaration(it);
    }

    fn visit_class(&mut self, it: &oxc::Class<'a>) {
        // The original immutable walker recorded only the class name and then the
        // class's type-bearing parts (decorators / implements / type params) as
        // RawNodes (type idents only). It did NOT walk `super_class` (the extends
        // clause) nor the class body's method/property members.
        if let Some(id) = &it.id {
            self.record_declaration(id);
        }
        if let Some(type_parameters) = &it.type_parameters {
            self.visit_ts_type_parameter_declaration(type_parameters);
        }
        if let Some(super_type_arguments) = &it.super_type_arguments {
            self.visit_ts_type_parameter_instantiation(super_type_arguments);
        }
        self.type_depth += 1;
        self.visit_ts_class_implements_list(&it.implements);
        self.type_depth -= 1;
    }

    fn visit_static_member_expression(&mut self, it: &oxc::StaticMemberExpression<'a>) {
        // Original walked the property only when computed; a static member is
        // non-computed, so the property name is never recorded.
        self.visit_expression(&it.object);
    }

    fn visit_object_property(&mut self, it: &oxc::ObjectProperty<'a>) {
        // Original walked the key only when computed.
        if it.computed {
            self.visit_property_key(&it.key);
        }
        self.visit_expression(&it.value);
    }

    fn visit_jsx_element(&mut self, it: &oxc::JSXElement<'a>) {
        // Mirror the original walker: the opening element's span is active only
        // while walking the element name (and its type arguments); it is cleared
        // before attributes and children.
        self.current_opening_element_span = Some(it.opening_element.span);
        self.record_jsx_element_name(&it.opening_element.name);
        self.current_opening_element_span = None;
        if let Some(type_args) = &it.opening_element.type_arguments {
            self.visit_ts_type_parameter_instantiation(type_args);
        }

        // The original walker visited only attribute VALUES and spread arguments,
        // never attribute names, and had no closing-element handling.
        for attr in &it.opening_element.attributes {
            match attr {
                oxc::JSXAttributeItem::Attribute(a) => {
                    if let Some(value) = &a.value {
                        match value {
                            oxc::JSXAttributeValue::ExpressionContainer(c) => {
                                self.visit_jsx_expression_container(c);
                            }
                            oxc::JSXAttributeValue::Element(el) => self.visit_jsx_element(el),
                            oxc::JSXAttributeValue::Fragment(f) => self.visit_jsx_fragment(f),
                            oxc::JSXAttributeValue::StringLiteral(_) => {}
                        }
                    }
                }
                oxc::JSXAttributeItem::SpreadAttribute(a) => {
                    self.visit_expression(&a.argument);
                }
            }
        }
        for child in &it.children {
            self.visit_jsx_child(child);
        }
    }

    fn visit_ts_type(&mut self, it: &oxc::TSType<'a>) {
        self.type_depth += 1;
        oxc_ast_visit::walk::walk_ts_type(self, it);
        self.type_depth -= 1;
    }

    fn visit_ts_type_annotation(&mut self, it: &oxc::TSTypeAnnotation<'a>) {
        self.type_depth += 1;
        oxc_ast_visit::walk::walk_ts_type_annotation(self, it);
        self.type_depth -= 1;
    }

    fn visit_ts_type_parameter_instantiation(
        &mut self,
        it: &oxc::TSTypeParameterInstantiation<'a>,
    ) {
        self.type_depth += 1;
        oxc_ast_visit::walk::walk_ts_type_parameter_instantiation(self, it);
        self.type_depth -= 1;
    }

    fn visit_ts_type_parameter_declaration(&mut self, it: &oxc::TSTypeParameterDeclaration<'a>) {
        self.type_depth += 1;
        oxc_ast_visit::walk::walk_ts_type_parameter_declaration(self, it);
        self.type_depth -= 1;
    }

    // The original immutable walker treated these TS declaration statements as
    // no-ops (nothing inside them was recorded). Override to skip entirely.
    fn visit_ts_type_alias_declaration(&mut self, _it: &oxc::TSTypeAliasDeclaration<'a>) {}

    fn visit_ts_interface_declaration(&mut self, _it: &oxc::TSInterfaceDeclaration<'a>) {}

    fn visit_ts_enum_declaration(&mut self, _it: &oxc::TSEnumDeclaration<'a>) {}

    fn visit_ts_module_declaration(&mut self, _it: &oxc::TSModuleDeclaration<'a>) {}
}

/// Build an index of the resolved identifier references and binding
/// declarations in a function's AST.
///
/// Walks the function's params (`FormalParameters`) and body, mirroring the
/// original Babel `IdentifierLocVisitor`: the function node itself is not
/// re-entered (its own name, if any, is recorded explicitly).
pub fn build_identifier_loc_index(func: &FunctionNode<'_, '_>) -> IdentifierLocIndex {
    let mut visitor = IdentifierLocVisitor {
        index: IdentifierLocIndex::default(),
        current_opening_element_span: None,
        type_depth: 0,
    };

    match func {
        FunctionNode::Function(f) => {
            // The function's own name is a declaration name.
            if let Some(id) = &f.id {
                visitor.record_declaration(id);
            }
            if let Some(type_parameters) = &f.type_parameters {
                visitor.visit_ts_type_parameter_declaration(type_parameters);
            }
            if let Some(this_param) = &f.this_param {
                visitor.visit_ts_this_parameter(this_param);
            }
            visitor.visit_formal_parameters(&f.params);
            if let Some(return_type) = &f.return_type {
                visitor.visit_ts_type_annotation(return_type);
            }
            if let Some(body) = &f.body {
                visitor.visit_function_body(body);
            }
        }
        FunctionNode::Arrow(arrow) => {
            if let Some(type_parameters) = &arrow.type_parameters {
                visitor.visit_ts_type_parameter_declaration(type_parameters);
            }
            visitor.visit_formal_parameters(&arrow.params);
            if let Some(return_type) = &arrow.return_type {
                visitor.visit_ts_type_annotation(return_type);
            }
            if arrow.expression {
                if let Some(oxc::Statement::ExpressionStatement(es)) = arrow.body.statements.first()
                {
                    visitor.visit_expression(&es.expression);
                } else {
                    visitor.visit_function_body(&arrow.body);
                }
            } else {
                visitor.visit_function_body(&arrow.body);
            }
        }
    }

    visitor.index
}
