//! Builds an index mapping identifier node-IDs to source locations.
//!
//! Walks the function's oxc AST to collect an [`IdentifierLocEntry`] for every
//! Identifier / JSXIdentifier node (and for identifiers inside TS type
//! annotations). Keyed by node_id (== `span.start`) for identity lookups; each
//! entry also stores `start` (byte offset) for range-containment checks in
//! `gather_captured_context`.
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
//! * every reference / binding identifier → `is_jsx = false`
//! * function / class declaration & expression names → `is_declaration_name = true`
//! * JSX element-name identifiers → `is_jsx = true` plus the enclosing
//!   `JSXOpeningElement`'s loc as `opening_element_loc`
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

use crate::react_compiler_hir::SourceLocation;
use crate::scope::ScopeInfo;

use crate::react_compiler_lowering::FunctionNode;
use crate::react_compiler_lowering::source_loc::LineOffsets;

/// Source location and whether the identifier is a JSXIdentifier.
pub struct IdentifierLocEntry {
    /// The byte offset of the identifier (base.start). Stored here so that
    /// callers iterating by node_id can still do position-range containment
    /// checks without a separate bridge map.
    pub start: u32,
    pub loc: SourceLocation,
    pub is_jsx: bool,
    /// For JSX identifiers that are the root name of a JSXOpeningElement,
    /// stores the JSXOpeningElement's loc (which spans the full tag).
    pub opening_element_loc: Option<SourceLocation>,
    /// True if this identifier is the name of a function/class declaration
    /// (not an expression reference). Used by `gather_captured_context` to
    /// skip non-expression positions, matching the TS behavior where the
    /// Expression visitor doesn't visit declaration names.
    pub is_declaration_name: bool,
    /// True if this identifier sits inside a type annotation subtree
    /// (TypeAnnotation/TSTypeAnnotation/TypeAlias/TSTypeAliasDeclaration).
    /// `gather_captured_context` skips these to match the TS
    /// gatherCapturedContext traverse, which skips those subtrees; the
    /// hoisting analysis and FindContextIdentifiers do NOT skip them in TS.
    pub in_type_annotation: bool,
}

/// Index mapping node_id → IdentifierLocEntry for all Identifier
/// and JSXIdentifier nodes in a function's AST.
pub type IdentifierLocIndex = FxHashMap<u32, IdentifierLocEntry>;

struct IdentifierLocVisitor<'a> {
    line_offsets: &'a LineOffsets,
    index: IdentifierLocIndex,
    /// Tracks the current JSXOpeningElement's loc while walking its name.
    current_opening_element_loc: Option<SourceLocation>,
    /// Depth of TS type subtrees currently being walked. Identifiers recorded
    /// while this is non-zero get `in_type_annotation = true`.
    type_depth: u32,
}

impl<'a> IdentifierLocVisitor<'a> {
    fn record(&mut self, span: oxc_span::Span, is_jsx: bool, is_declaration_name: bool) {
        let opening_element_loc =
            if is_jsx { self.current_opening_element_loc.clone() } else { None };
        // `or_insert` keeps the richer entry already recorded for a node_id.
        // Function/class names are recorded as declaration names *before* the
        // generic binding-identifier walk re-visits them, so the declaration
        // entry wins, matching the original visitor.
        self.index.entry(span.start).or_insert(IdentifierLocEntry {
            start: span.start,
            loc: self.line_offsets.source_location(span),
            is_jsx,
            opening_element_loc,
            is_declaration_name,
            in_type_annotation: self.type_depth > 0,
        });
    }

    /// Record the JSX element name identifiers (and only those) while the
    /// `current_opening_element_loc` is set, mirroring the original
    /// `walk_jsx_element_name` / `walk_jsx_member_expression`.
    fn record_jsx_element_name(&mut self, name: &oxc::JSXElementName<'a>) {
        match name {
            oxc::JSXElementName::Identifier(id) => self.record(id.span, true, false),
            oxc::JSXElementName::IdentifierReference(id) => self.record(id.span, true, false),
            oxc::JSXElementName::ThisExpression(t) => self.record(t.span, true, false),
            oxc::JSXElementName::MemberExpression(m) => self.record_jsx_member_expression(m),
            // JSXNamespacedName identifiers are not visited by the original walker.
            oxc::JSXElementName::NamespacedName(_) => {}
        }
    }

    fn record_jsx_member_expression(&mut self, expr: &oxc::JSXMemberExpression<'a>) {
        match &expr.object {
            oxc::JSXMemberExpressionObject::IdentifierReference(id) => {
                self.record(id.span, true, false);
            }
            oxc::JSXMemberExpressionObject::ThisExpression(t) => self.record(t.span, true, false),
            oxc::JSXMemberExpressionObject::MemberExpression(inner) => {
                self.record_jsx_member_expression(inner);
            }
        }
        self.record(expr.property.span, true, false);
    }
}

impl<'a> Visit<'a> for IdentifierLocVisitor<'a> {
    fn visit_identifier_reference(&mut self, it: &oxc::IdentifierReference<'a>) {
        self.record(it.span, false, false);
    }

    fn visit_identifier_name(&mut self, it: &oxc::IdentifierName<'a>) {
        self.record(it.span, false, false);
    }

    fn visit_binding_identifier(&mut self, it: &oxc::BindingIdentifier<'a>) {
        // `collect_type_idents` only collected IdentifierReference / IdentifierName,
        // never BindingIdentifier, so type-parameter declaration names (`<T>`) and
        // other binding positions inside type subtrees must not be recorded.
        if self.type_depth > 0 {
            return;
        }
        self.record(it.span, false, false);
    }

    fn visit_jsx_identifier(&mut self, it: &oxc::JSXIdentifier<'a>) {
        self.record(it.span, true, false);
    }

    fn visit_function(&mut self, it: &oxc::Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        // The function's own name is a declaration name, not an expression
        // reference. Record it first so the generic binding-identifier walk
        // (via the default traversal below) does not overwrite the flag.
        if let Some(id) = &it.id {
            self.record(id.span, false, true);
        }
        oxc_ast_visit::walk::walk_function(self, it, flags);
    }

    fn visit_class(&mut self, it: &oxc::Class<'a>) {
        // The original immutable walker recorded only the class name and then the
        // class's type-bearing parts (decorators / implements / type params) as
        // RawNodes (type idents only). It did NOT walk `super_class` (the extends
        // clause) nor the class body's method/property members.
        if let Some(id) = &it.id {
            self.record(id.span, false, true);
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
        // Mirror the original walker: the opening element's loc is active only
        // while walking the element name (and its type arguments); it is cleared
        // before attributes and children.
        self.current_opening_element_loc =
            Some(self.line_offsets.source_location(it.opening_element.span));
        self.record_jsx_element_name(&it.opening_element.name);
        if let Some(type_args) = &it.opening_element.type_arguments {
            self.visit_ts_type_parameter_instantiation(type_args);
        }
        self.current_opening_element_loc = None;

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

/// Build an index of all Identifier and JSXIdentifier positions in a function's AST.
///
/// Walks the function's params (`FormalParameters`) and body, mirroring the
/// original Babel `IdentifierLocVisitor`: the function node itself is not
/// re-entered (its own name, if any, is recorded explicitly).
pub fn build_identifier_loc_index(
    func: &FunctionNode<'_>,
    scope_info: &ScopeInfo,
    line_offsets: &LineOffsets,
) -> IdentifierLocIndex {
    // The loc index is purely position-driven; scope tracking is not required.
    let _ = scope_info;

    let mut visitor = IdentifierLocVisitor {
        line_offsets,
        index: FxHashMap::default(),
        current_opening_element_loc: None,
        type_depth: 0,
    };

    match func {
        FunctionNode::Function(f) => {
            // The function's own name is a declaration name.
            if let Some(id) = &f.id {
                visitor.record(id.span, false, true);
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
