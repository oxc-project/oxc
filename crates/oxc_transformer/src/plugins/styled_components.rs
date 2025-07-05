use oxc_allocator::{TakeIn, Vec as ArenaVec};
use oxc_ast::{NONE, ast::*};
use oxc_semantic::SymbolId;
use oxc_span::SPAN;
use oxc_traverse::{Ancestor, Traverse};
use serde::Deserialize;

use crate::{
    context::{TransformCtx, TraverseCtx},
    state::TransformState,
};

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct StyledComponentsOptions {
    display_name: bool,
    file_name: bool,
    ssr: bool,
    transpile_template_literals: bool,
    pure: bool,
}

impl Default for StyledComponentsOptions {
    fn default() -> Self {
        Self {
            display_name: true,
            file_name: true,
            ssr: true,
            transpile_template_literals: false,
            pure: false,
        }
    }
}

#[derive(Default)]
struct StyledComponentsBinding {
    namespace: Option<SymbolId>,
    styled: Option<SymbolId>,
    create_global_style: Option<SymbolId>,
    css: Option<SymbolId>,
    inject_global: Option<SymbolId>,
    key_frames: Option<SymbolId>,
    with_theme: Option<SymbolId>,
    use_theme: Option<SymbolId>,
}

pub struct StyledComponents<'a, 'ctx> {
    pub options: StyledComponentsOptions,
    pub ctx: &'ctx TransformCtx<'a>,

    // State
    styled_bindings: StyledComponentsBinding,
}

impl<'a, 'ctx> StyledComponents<'a, 'ctx> {
    pub fn new(options: StyledComponentsOptions, ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { options, ctx, styled_bindings: StyledComponentsBinding::default() }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for StyledComponents<'a, '_> {
    fn enter_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.collect_styled_bindings(program, ctx);
    }

    fn enter_tagged_template_expression(
        &mut self,
        template: &mut TaggedTemplateExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.add_display_name_and_component_id(&mut template.tag, ctx);
    }

    fn enter_call_expression(&mut self, call: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        // Only transform call expression that is not a part of a member expression or a callee of another call expression.
        if !matches!(
            ctx.parent(),
            Ancestor::CallExpressionCallee(_) | Ancestor::StaticMemberExpressionObject(_)
        ) {
            self.add_display_name_and_component_id(&mut call.callee, ctx);
        }
    }
}

impl<'a> StyledComponents<'a, '_> {
    fn add_display_name_and_component_id(&self, expr: &mut Expression<'a>, ctx: &TraverseCtx<'a>) {
        if !self.options.display_name && !self.options.ssr {
            return;
        }

        if let Some(call) = Self::get_with_config(expr) {
            if let Expression::StaticMemberExpression(member) = &call.callee {
                if self.is_styled(&member.object, ctx) {
                    if let Some(Argument::ObjectExpression(object)) = call.arguments.first_mut() {
                        if !object.properties.iter().any(|prop| {
                            matches!(prop, ObjectPropertyKind::ObjectProperty(property)
                                if matches!(&property.key, PropertyKey::StaticIdentifier(ident)
                                if matches!(ident.name.as_str(), "displayName" | "componentId"))
                            )
                        }) {
                            self.add_properties(&mut object.properties, ctx);
                        }
                    }
                }
            }
        } else if self.is_styled(expr, ctx) {
            let mut properties = ctx.ast.vec_with_capacity(
                usize::from(self.options.display_name) + usize::from(self.options.ssr),
            );
            self.add_properties(&mut properties, ctx);
            let object = ctx.ast.alloc_object_expression(SPAN, properties);
            let arguments = ctx.ast.vec1(Argument::ObjectExpression(object));
            let object = expr.take_in(ctx.ast);
            let property = ctx.ast.identifier_name(SPAN, "withConfig");
            let callee =
                Expression::from(ctx.ast.member_expression_static(SPAN, object, property, false));
            let call = ctx.ast.expression_call(SPAN, callee, NONE, arguments, false);
            *expr = call;
        }
    }

    fn get_with_config<'b>(expr: &'b mut Expression<'a>) -> Option<&'b mut CallExpression<'a>> {
        let is_with_config = |call: &CallExpression<'a>| {
            if let Expression::StaticMemberExpression(member) = &call.callee {
                member.property.name == "withConfig"
            } else {
                false
            }
        };

        let mut current = expr;
        loop {
            match current {
                Expression::CallExpression(call) => {
                    if is_with_config(call) {
                        return Some(call);
                    }
                    current = &mut call.callee;
                }
                Expression::StaticMemberExpression(member) => {
                    current = &mut member.object;
                }
                _ => return None,
            }
        }
    }

    fn add_properties(
        &self,
        properties: &mut ArenaVec<'a, ObjectPropertyKind<'a>>,
        ctx: &TraverseCtx<'a>,
    ) {
        if self.options.display_name {
            let value = self.get_display_name(ctx);
            properties.push(Self::create_object_property("displayName", value, ctx));
        }
        if self.options.ssr {
            properties.push(Self::create_object_property("componentId", None, ctx));
        }
    }

    fn create_object_property(
        key: &'static str,
        value: Option<&'a str>,
        ctx: &TraverseCtx<'a>,
    ) -> ObjectPropertyKind<'a> {
        let key = ctx.ast.property_key_static_identifier(SPAN, key);
        let value = ctx.ast.expression_string_literal(SPAN, value.unwrap_or(""), None);
        ctx.ast.object_property_kind_object_property(
            SPAN,
            PropertyKind::Init,
            key,
            value,
            false,
            false,
            false,
        )
    }

    fn get_component_name(ctx: &TraverseCtx<'a>) -> Option<Atom<'a>> {
        let mut assignment_name = None;
        for ancestor in ctx.ancestors() {
            match ancestor {
                // `X = styled`
                Ancestor::AssignmentExpressionRight(assignment) => match assignment.left() {
                    // we've got an displayName (if we need it) no need to continue
                    // However if this is an assignment expression like X = styled then we
                    // want to keep going up incase there is Y = X = styled; in this case we
                    // want to pick the outer name because react-refresh will add HMR variables
                    // like this: X = _a = styled. We could also consider only doing this if the
                    // name starts with an underscore.
                    AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                        assignment_name = Some(ident.name);
                    }
                    AssignmentTarget::StaticMemberExpression(member) => {
                        assignment_name = Some(member.property.name);
                    }
                    _ => return None,
                },
                // `const X = styled`
                Ancestor::VariableDeclaratorInit(declarator) => {
                    return if let BindingPatternKind::BindingIdentifier(ident) =
                        &declarator.id().kind
                    {
                        Some(ident.name)
                    } else {
                        None
                    };
                }
                // `const X = { Y: styled }`
                Ancestor::ObjectPropertyValue(property) => {
                    return if let PropertyKey::StaticIdentifier(ident) = property.key() {
                        Some(ident.name)
                    } else {
                        None
                    };
                }
                // `class Y { (static) X = styled }`
                Ancestor::PropertyDefinitionValue(property) => {
                    return if let PropertyKey::StaticIdentifier(ident) = property.key() {
                        Some(ident.name)
                    } else {
                        None
                    };
                }
                _ => {
                    if ancestor.is_parent_of_statement() {
                        // we've hit a statement, we should stop crawling up
                        return assignment_name;
                    }
                }
            }
        }

        unreachable!()
    }

    fn get_display_name(&self, ctx: &TraverseCtx<'a>) -> Option<&'a str> {
        let component_name = Self::get_component_name(ctx);

        if self.options.file_name
            && let Some(file_stem) = self.ctx.source_path.file_stem().and_then(|stem| stem.to_str())
        {
            if let Some(component_name) = component_name {
                if file_stem != component_name {
                    return Some(
                        ctx.ast
                            .atom_from_strs_array([file_stem, "__", component_name.as_str()])
                            .as_str(),
                    );
                }
            } else {
                return Some(ctx.ast.str(file_stem));
            }
        }

        component_name.map(|name| name.as_str())
    }

    /// Note: didn't support commonjs yet.
    fn is_styled(&self, callee: &Expression<'a>, ctx: &TraverseCtx<'a>) -> bool {
        match callee.without_parentheses() {
            Expression::StaticMemberExpression(member) => {
                if let Expression::Identifier(ident) = &member.object {
                    !is_helper(&member.property.name) && self.is_reference_of_styled(ident, ctx)
                } else {
                    false
                }
            }
            Expression::CallExpression(call) => match &call.callee {
                Expression::Identifier(ident) => self.is_reference_of_styled(ident, ctx),
                Expression::StaticMemberExpression(member) => self.is_styled(&member.object, ctx),
                Expression::SequenceExpression(sequence) => {
                    if let Some(last) = sequence.expressions.last() {
                        match last {
                            Expression::Identifier(ident) => {
                                self.is_reference_of_styled(ident, ctx)
                            }
                            Expression::StaticMemberExpression(member) => {
                                self.is_styled(&member.object, ctx)
                            }
                            _ => false,
                        }
                    } else {
                        false
                    }
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn is_reference_of_styled(
        &self,
        ident: &IdentifierReference<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
        self.styled_bindings.styled.is_some_and(|styled_binding| {
            let reference_id = ident.reference_id();
            ctx.scoping()
                .get_reference(reference_id)
                .symbol_id()
                .is_some_and(|reference_symbol_id| reference_symbol_id == styled_binding)
        })
    }

    fn collect_styled_bindings(&mut self, program: &Program<'a>, _ctx: &mut TraverseCtx<'a>) {
        for statement in &program.body {
            let Statement::ImportDeclaration(import) = &statement else { continue };

            if let Some(specifiers) = &import.specifiers {
                if !is_valid_styled_component_source(&import.source.value) {
                    continue;
                }

                for specifier in specifiers {
                    match specifier {
                        ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                            let get_symbol_id = || Some(specifier.local.symbol_id());
                            match specifier.imported.name().as_str() {
                                "default" => {
                                    self.styled_bindings.styled = get_symbol_id();
                                }
                                "createGlobalStyle" => {
                                    self.styled_bindings.create_global_style = get_symbol_id();
                                }
                                "css" => {
                                    self.styled_bindings.css = get_symbol_id();
                                }
                                "injectGlobal" => {
                                    self.styled_bindings.inject_global = get_symbol_id();
                                }
                                "keyframes" => {
                                    self.styled_bindings.key_frames = get_symbol_id();
                                }
                                "withTheme" => {
                                    self.styled_bindings.with_theme = get_symbol_id();
                                }
                                "useTheme" => {
                                    self.styled_bindings.use_theme = get_symbol_id();
                                }
                                _ => {}
                            }
                        }
                        ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                            self.styled_bindings.styled = Some(specifier.local.symbol_id());
                        }
                        ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                            self.styled_bindings.namespace = Some(specifier.local.symbol_id());
                        }
                    }
                }
            }
        }
    }
}

fn is_helper(name: &str) -> bool {
    matches!(
        name,
        "createGlobalStyle" | "css" | "injectGlobal" | "keyframes" | "withTheme" | "useTheme"
    )
}

#[expect(unused)]
fn is_pure_helper(name: &str) -> bool {
    matches!(name, "createGlobalStyle" | "css" | "keyframes" | "withTheme" | "useTheme")
}

fn is_valid_styled_component_source(source: &str) -> bool {
    matches!(
        source,
        "styled-components"
            | "styled-components/no-tags"
            | "styled-components/native"
            | "styled-components/primitives"
    )
}
