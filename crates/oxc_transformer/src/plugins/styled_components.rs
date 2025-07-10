//! Styled Components
//!
//! This plugin adds support for server-side rendering, minification of styles, and
//! a nicer debugging experience when using styled-components.
//!
//! > This plugin is port from the official Babel plugin for styled-components.
//!
//! ## Implementation Status
//!
//! > Note: Currently, this plugin only supports styled-components imported via import statements.
//! The transformation will not be applied if you import it using `require("styled-components")`,
//! in other words, it only supports `ESM` not `CJS`.
//!
//! ### Options:
//! **✅ Fully Supported:**
//! - `displayName`: Adds display names for debugging
//! - `fileName`: Controls filename prefixing in display names
//! - `ssr`: Adds unique component IDs for server-side rendering
//! - `transpileTemplateLiterals`: Converts template literals to function calls
//! - `minify`: Minifies CSS content in template literals
//! - `namespace`: Adds namespace prefixes to component IDs
//! - `meaninglessFileNames`: Controls which filenames are considered meaningless
//!
//! **⚠️ Partially Supported:**
//! - `pure`: Only supports call expressions, not tagged template expressions (bundler limitation)
//!
//! **❌ Not Yet Implemented:**
//! - `cssProp`: JSX css prop transformation
//! - `topLevelImportPaths`: Custom import path handling
//!
//! ## Example
//!
//! Input:
//! ```js
//! import styled from 'styled-components';
//!
//! const Button = styled.div`
//!   color: blue;
//!   padding: 10px;
//! `;
//! ```
//!
//! Output (with default options):
//! ```js
//! import styled from 'styled-components';
//!
//! const Button = styled.div.withConfig({
//!   displayName: "Button",
//!   componentId: "sc-1234567-0"
//! })(["color:blue;padding:10px;"]);
//! ```
//!
//! ## References
//!
//! - Babel plugin: <https://github.com/styled-components/babel-plugin-styled-components>
//! - Documentation: <https://styled-components.com/docs/tooling#babel-plugin>

use rustc_hash::FxHashSet;
use serde::Deserialize;

use std::iter::once;

use oxc_allocator::{StringBuilder, TakeIn, Vec as ArenaVec};
use oxc_ast::{AstBuilder, NONE, ast::*};
use oxc_semantic::SymbolId;
use oxc_span::SPAN;
use oxc_traverse::{Ancestor, Traverse};

use crate::{
    context::{TransformCtx, TraverseCtx},
    state::TransformState,
};

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct StyledComponentsOptions {
    /// Enhances the attached CSS class name on each component with richer output to help
    /// identify your components in the DOM without React DevTools. It also allows you to
    /// see the component's `displayName` in React DevTools.
    ///
    /// When enabled, components show up as `<button class="Button-asdf123 asdf123" />`
    /// instead of just `<button class="asdf123" />`, and display meaningful names like
    /// `MyButton` instead of `styled.button` in React DevTools.
    ///
    /// Default: `true`
    #[serde(default = "default_as_true")]
    pub display_name: bool,

    /// Controls whether the `displayName` of a component will be prefixed with the filename
    /// to make the component name as unique as possible.
    ///
    /// When `true`, the filename is used to prefix component names. When `false`, only the
    /// component name is used for the `displayName`. This can be useful for testing with
    /// enzyme where you want to search components by displayName.
    ///
    /// Default: `true`
    #[serde(default = "default_as_true")]
    pub file_name: bool,

    /// Adds a unique identifier to every styled component to avoid checksum mismatches
    /// due to different class generation on the client and server during server-side rendering.
    ///
    /// Without this option, React will complain with an HTML attribute mismatch warning
    /// during rehydration when using server-side rendering.
    ///
    /// Default: `true`
    #[serde(default = "default_as_true")]
    pub ssr: bool,

    /// Transpiles styled-components tagged template literals to a smaller representation
    /// than what Babel normally creates, helping to reduce bundle size.
    ///
    /// Converts `styled.div\`width: 100%;\`` to `styled.div(['width: 100%;'])`, which is
    /// more compact than the standard Babel template literal transformation.
    ///
    /// Default: `true`
    #[expect(clippy::doc_link_with_quotes)]
    #[serde(default = "default_as_true")]
    pub transpile_template_literals: bool,

    /// Minifies CSS content by removing all whitespace and comments from your CSS,
    /// keeping valuable bytes out of your bundles.
    ///
    /// This optimization helps reduce the final bundle size by eliminating unnecessary
    /// whitespace and comments in CSS template literals.
    ///
    /// Default: `true`
    #[serde(default = "default_as_true")]
    pub minify: bool,

    /// Enables transformation of JSX `css` prop when using styled-components.
    ///
    /// When enabled, JSX elements with a `css` prop are transformed to work with
    /// styled-components' css prop functionality.
    ///
    /// **Note: This feature is not yet implemented in oxc.**
    ///
    /// Default: `true`
    #[serde(default = "default_as_true")]
    pub css_prop: bool,

    /// Enables "pure annotation" to aid dead code elimination by bundlers.
    ///
    /// Adds `/*#__PURE__*/` comments to styled component calls, helping minifiers
    /// perform better tree-shaking by indicating that these calls have no side effects.
    /// This is particularly useful because styled components are normally assumed to
    /// have side effects and can't be properly eliminated by minifiers.
    ///
    /// **Note: Currently only supports call expressions. Tagged template expressions
    /// are not yet supported due to bundler limitations. See:**
    /// <https://github.com/rollup/rollup/issues/4035>
    ///
    /// Default: `false`
    #[serde(default)]
    pub pure: bool,

    /// Adds a namespace prefix to component identifiers to ensure class names are unique.
    ///
    /// This is particularly useful when working with micro-frontends where class name
    /// collisions can occur. The namespace will be prepended to generated component IDs.
    ///
    /// Example: With `namespace: "my-app"`, generates `componentId: "my-app__sc-3rfj0a-1"`
    ///
    /// Default: `None`
    #[serde(default)]
    pub namespace: Option<String>,

    /// List of file names that are considered meaningless for component naming purposes.
    ///
    /// When the `fileName` option is enabled and a component is in a file with a name
    /// from this list, the directory name will be used instead of the file name for
    /// the component's display name. This is useful for patterns like `Button/index.jsx`
    /// where "index" is not descriptive.
    ///
    /// Example: With "index" in the list, `Button/index.jsx` will generate a display
    /// name of "Button" instead of "index".
    ///
    /// Default: `["index"]`
    #[serde(default = "default_for_meaningless_file_names")]
    pub meaningless_file_names: Vec<String>,

    /// Import paths to be considered as styled-components imports at the top level.
    ///
    /// This option allows specifying additional import paths that should be treated
    /// as styled-components imports, enabling the plugin to work with custom builds
    /// or aliased imports of styled-components.
    ///
    /// **Note: This feature is not yet implemented in oxc.**
    ///
    /// Default: `[]`
    #[serde(default)]
    pub top_level_import_paths: Vec<String>,
}

const fn default_as_true() -> bool {
    true
}

fn default_for_meaningless_file_names() -> Vec<String> {
    vec![String::from("index")]
}

impl Default for StyledComponentsOptions {
    /// Creates the default configuration for styled-components transformation.
    ///
    /// Most options are enabled by default to match the behavior of the official
    /// Babel plugin. Note that some options like `cssProp` and `topLevelImportPaths`
    /// are set but not yet implemented.
    ///
    /// The `pure` option is disabled by default to avoid potential issues with
    /// tree-shaking in some bundlers.
    fn default() -> Self {
        Self {
            display_name: true,
            file_name: true,
            ssr: true,
            transpile_template_literals: true,
            pure: false,
            minify: true,
            namespace: None,
            css_prop: true,
            meaningless_file_names: default_for_meaningless_file_names(),
            top_level_import_paths: vec![],
        }
    }
}

/// Tracks symbol IDs for styled-components imports to identify which variables
/// are bound to styled-components functionality.
#[derive(Default)]
struct StyledComponentsBinding {
    /// `import * as styled from 'styled-components'`
    namespace: Option<SymbolId>,
    /// `import styled from 'styled-components'` or `import { default as styled } from 'styled-components'`
    styled: Option<SymbolId>,
    /// Named imports like `import { createGlobalStyle, css, keyframes } from 'styled-components'`
    helpers: [Option<SymbolId>; 6],
}

impl StyledComponentsBinding {
    fn helper_symbol_id(&self, helper: StyledComponentsHelper) -> Option<SymbolId> {
        self.helpers[helper as usize]
    }

    fn set_helper_symbol_id(&mut self, helper: StyledComponentsHelper, symbol_id: SymbolId) {
        self.helpers[helper as usize] = Some(symbol_id);
    }
}

/// Helper functions.
///
/// Used as index into `StyledComponentsBinding::helpers` array.
#[derive(Copy, Clone)]
#[repr(u8)]
enum StyledComponentsHelper {
    CreateGlobalStyle = 0,
    Css = 1,
    InjectGlobal = 2,
    Keyframes = 3,
    UseTheme = 4,
    WithTheme = 5,
}

impl StyledComponentsHelper {
    fn from_str(name: &str) -> Option<Self> {
        match name {
            "createGlobalStyle" => Some(Self::CreateGlobalStyle),
            "css" => Some(Self::Css),
            "injectGlobal" => Some(Self::InjectGlobal),
            "keyframes" => Some(Self::Keyframes),
            "useTheme" => Some(Self::UseTheme),
            "withTheme" => Some(Self::WithTheme),
            _ => None,
        }
    }
}

pub struct StyledComponents<'a, 'ctx> {
    pub options: StyledComponentsOptions,
    pub ctx: &'ctx TransformCtx<'a>,

    // State
    /// Tracks which variables are bound to styled-components imports
    styled_bindings: StyledComponentsBinding,
    /// Counter for generating unique component IDs
    component_count: usize,
    /// Hash of the current file for component ID generation
    file_hash: Option<&'a str>,
}

impl<'a, 'ctx> StyledComponents<'a, 'ctx> {
    pub fn new(options: StyledComponentsOptions, ctx: &'ctx TransformCtx<'a>) -> Self {
        Self {
            options,
            ctx,
            styled_bindings: StyledComponentsBinding::default(),
            file_hash: None,
            component_count: 0,
        }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for StyledComponents<'a, '_> {
    fn enter_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.collect_styled_bindings(program, ctx);
    }

    fn enter_variable_declarator(
        &mut self,
        variable_declarator: &mut VariableDeclarator<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        self.handle_pure_annotation(variable_declarator, ctx);
    }

    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if matches!(expr, Expression::TaggedTemplateExpression(_)) {
            self.transform_tagged_template_expression(expr, ctx);
        }
    }

    fn enter_call_expression(&mut self, call: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        if (self.options.display_name || self.options.ssr)
            // Only transform call expression that is not a part of a member expression
            // or a callee of another call expression.
            && !matches!(
                ctx.parent(),
                Ancestor::CallExpressionCallee(_) | Ancestor::StaticMemberExpressionObject(_)
            )
        {
            self.add_display_name_and_component_id(&mut call.callee, ctx);
        }
    }
}

impl<'a> StyledComponents<'a, '_> {
    fn transform_tagged_template_expression(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &TraverseCtx<'a>,
    ) {
        let Expression::TaggedTemplateExpression(tagged) = expr else {
            unreachable!();
        };

        let is_styled = if self.options.display_name || self.options.ssr {
            self.add_display_name_and_component_id(&mut tagged.tag, ctx)
        } else {
            self.is_styled(&tagged.tag, ctx)
        };

        if !is_styled
            && !matches!(&tagged.tag, Expression::Identifier(ident) if self.is_helper(ident, ctx))
        {
            return;
        }

        if self.options.minify {
            Self::minify_tagged_template_expression(tagged, ctx);
        }

        if self.options.transpile_template_literals {
            *expr = Self::transpile_template_literals(tagged, ctx);
        }
    }

    fn handle_pure_annotation(
        &self,
        declarator: &mut VariableDeclarator<'a>,
        ctx: &TraverseCtx<'a>,
    ) {
        if !self.options.pure {
            return;
        }

        // TODO: Support adding pure annotation to `TaggedTemplateExpression`.
        // Note: As of now, no bundle can handle pure tagged template expressions.
        // <https://github.com/rollup/rollup/issues/4035>
        if let Some(Expression::CallExpression(call)) = &mut declarator.init {
            if matches!(&call.callee, Expression::Identifier(ident) if self.is_pure_helper(ident, ctx))
                || self.is_styled(&call.callee, ctx)
            {
                call.pure = true;
            }
        }
    }

    fn minify_tagged_template_expression(
        expr: &mut TaggedTemplateExpression<'a>,
        ctx: &TraverseCtx<'a>,
    ) {
        let TemplateLiteral { quasis, expressions, .. } = &mut expr.quasi;

        let (new_raws, remained_expression_indices) = CssMinifier::minify_quasis(quasis, ctx.ast);

        // Remove expressions that are not kept after minification.
        for i in (0..expressions.len()).rev() {
            if !remained_expression_indices.contains(&i) {
                expressions.swap_remove(i);
            }
        }

        // Update the quasis with the new raw values.
        for (new_raw, quais) in new_raws.into_iter().zip(quasis.iter_mut()) {
            quais.value.raw = new_raw;
        }

        // SAFETY:
        unsafe {
            quasis.set_len(expressions.len() + 1);
        }
    }

    fn transpile_template_literals(
        expr: &mut TaggedTemplateExpression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Expression<'a> {
        let TaggedTemplateExpression {
            span,
            tag,
            quasi: TemplateLiteral { span: quasi_span, quasis, expressions },
            type_arguments,
        } = expr.take_in(ctx.ast.allocator);

        let quasis_elements = ctx.ast.vec_from_iter(quasis.into_iter().map(|quasi| {
            ArrayExpressionElement::from(ctx.ast.expression_string_literal(
                quasi.span,
                quasi.value.raw,
                None,
            ))
        }));

        let quasis = Argument::from(ctx.ast.expression_array(quasi_span, quasis_elements));
        let arguments =
            ctx.ast.vec_from_iter(once(quasis).chain(expressions.into_iter().map(Argument::from)));
        ctx.ast.expression_call(span, tag, type_arguments, arguments, false)
    }

    fn add_display_name_and_component_id(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
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
        } else {
            return false;
        }

        true
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
                            let symbol_id = specifier.local.symbol_id();
                            let imported_name = specifier.imported.name();
                            match imported_name.as_str() {
                                "default" => {
                                    self.styled_bindings.styled = Some(symbol_id);
                                }
                                name => {
                                    if let Some(helper) = StyledComponentsHelper::from_str(name) {
                                        self.styled_bindings
                                            .set_helper_symbol_id(helper, symbol_id);
                                    }
                                }
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
        &mut self,
        properties: &mut ArenaVec<'a, ObjectPropertyKind<'a>>,
        ctx: &TraverseCtx<'a>,
    ) {
        if self.options.display_name {
            let value = self.get_display_name(ctx);
            properties.push(Self::create_object_property("displayName", value, ctx));
        }
        if self.options.ssr {
            let value = Some(self.get_component_id(ctx));
            properties.push(Self::create_object_property("componentId", value, ctx));
        }
    }

    fn get_component_name(ctx: &TraverseCtx<'a>) -> Option<Atom<'a>> {
        let mut assignment_name = None;

        for ancestor in ctx.ancestors() {
            match ancestor {
                // `X = styled` or `X.prop = styled`
                Ancestor::AssignmentExpressionRight(assignment) => {
                    assignment_name = match assignment.left() {
                        // we've got an displayName (if we need it) no need to continue
                        // However if this is an assignment expression like X = styled then we
                        // want to keep going up incase there is Y = X = styled; in this case we
                        // want to pick the outer name because react-refresh will add HMR variables
                        // like this: X = _a = styled. We could also consider only doing this if the
                        // name starts with an underscore.
                        AssignmentTarget::AssignmentTargetIdentifier(ident) => Some(ident.name),
                        AssignmentTarget::StaticMemberExpression(member) => {
                            Some(member.property.name)
                        }
                        _ => return None,
                    };
                }
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

    fn get_component_id(&mut self, ctx: &TraverseCtx<'a>) -> &'a str {
        let namespace = self.options.namespace.as_ref().map_or("", |namespace| {
            ctx.ast.allocator.alloc_concat_strs_array([namespace.as_str(), "__"])
        });

        let file_hash = self.get_file_hash(ctx);
        let id = ctx.ast.allocator.alloc_concat_strs_array([
            namespace,
            "sc-",
            file_hash,
            "-",
            itoa::Buffer::new().format(self.component_count),
        ]);
        self.component_count += 1;
        id
    }

    fn get_file_hash(&mut self, ctx: &TraverseCtx<'a>) -> &'a str {
        use rustc_hash::FxHasher;
        use std::hash::{Hash, Hasher};

        #[inline]
        fn base36_encode<'a>(mut num: u64, ctx: &TraverseCtx<'a>) -> &'a str {
            const BASE36_BYTES: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";

            num %= 36_u64.pow(6); // 36^6, to ensure the result is <= 6 characters long.

            let mut result = Vec::with_capacity(6);

            while num != 0 {
                result.push(BASE36_BYTES[(num % 36) as usize]);
                num /= 36;
            }

            ctx.ast.allocator.alloc_str(
                // SAFETY: the bytes are valid UTF-8 as they are ASCII characters.
                unsafe { std::str::from_utf8_unchecked(&result) },
            )
        }

        self.file_hash.get_or_insert_with(|| {
            let mut hasher = FxHasher::default();
            if self.ctx.source_path.is_relative() {
                self.ctx.source_path.hash(&mut hasher);
            } else {
                self.ctx.source_text.hash(&mut hasher);
            }

            base36_encode(hasher.finish(), ctx)
        })
    }

    fn get_display_name(&self, ctx: &TraverseCtx<'a>) -> Option<&'a str> {
        let component_name = Self::get_component_name(ctx);

        if self.options.file_name
            && let Some(file_stem) = self.ctx.source_path.file_stem().and_then(|stem| stem.to_str())
        {
            let block_name = if self.options.meaningless_file_names.contains(&file_stem.to_string())
            {
                self.ctx
                    .source_path
                    .parent()
                    .and_then(|parent| parent.file_name())
                    .and_then(|name| name.to_str())
                    .unwrap_or(file_stem)
            } else {
                file_stem
            };

            if let Some(component_name) = component_name {
                if block_name == component_name {
                    return Some(ctx.ast.str(component_name.as_str()));
                }
                return Some(
                    ctx.ast
                        .atom_from_strs_array([block_name, "__", component_name.as_str()])
                        .as_str(),
                );
            }
            return Some(ctx.ast.str(block_name));
        }

        component_name.map(|name| name.as_str())
    }

    /// Returns true if the given callee is a styled-components binding.
    /// Handles various forms: styled.div, styled.default, styled(...), etc.
    fn is_styled(&self, callee: &Expression<'a>, ctx: &TraverseCtx<'a>) -> bool {
        match callee.without_parentheses() {
            Expression::StaticMemberExpression(member) => {
                if let Expression::Identifier(ident) = &member.object {
                    StyledComponentsHelper::from_str(&member.property.name).is_none()
                        && Self::is_reference_of_styled(self.styled_bindings.styled, ident, ctx)
                } else if let Expression::StaticMemberExpression(static_member) = &member.object {
                    // Handle `styled.default`
                    static_member.property.name == "default"
                        && matches!(&static_member.object, Expression::Identifier(ident)
                        if Self::is_reference_of_styled(self.styled_bindings.namespace, ident, ctx))
                } else {
                    false
                }
            }
            Expression::CallExpression(call) => match &call.callee {
                Expression::Identifier(ident) => {
                    Self::is_reference_of_styled(self.styled_bindings.styled, ident, ctx)
                }
                Expression::StaticMemberExpression(member) => self.is_styled(&member.object, ctx),
                Expression::SequenceExpression(sequence) => {
                    if let Some(last) = sequence.expressions.last() {
                        match last {
                            Expression::Identifier(ident) => Self::is_reference_of_styled(
                                self.styled_bindings.styled,
                                ident,
                                ctx,
                            ),
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

    fn is_helper(&self, ident: &IdentifierReference<'a>, ctx: &TraverseCtx<'a>) -> bool {
        StyledComponentsHelper::from_str(&ident.name)
            .and_then(|helper| self.styled_bindings.helper_symbol_id(helper))
            .is_some_and(|symbol_id| {
                let reference_id = ident.reference_id();
                ctx.scoping()
                    .get_reference(reference_id)
                    .symbol_id()
                    .is_some_and(|reference_symbol_id| reference_symbol_id == symbol_id)
            })
    }

    fn is_pure_helper(&self, ident: &IdentifierReference<'a>, ctx: &TraverseCtx<'a>) -> bool {
        ident.name != "injectGlobal" && self.is_helper(ident, ctx)
    }

    fn is_reference_of_styled(
        styled_binding: Option<SymbolId>,
        ident: &IdentifierReference<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
        styled_binding.is_some_and(|styled_binding| {
            let reference_id = ident.reference_id();
            ctx.scoping()
                .get_reference(reference_id)
                .symbol_id()
                .is_some_and(|reference_symbol_id| reference_symbol_id == styled_binding)
        })
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

pub struct CssMinifier<'a> {
    ast: AstBuilder<'a>,
}

impl<'a> CssMinifier<'a> {
    const PLACEHOLDER_PREFIX: &'static str = "__PLACEHOLDER_";
    const PLACEHOLDER_SUFFIX: &'static str = "__";

    pub fn new(ast: AstBuilder<'a>) -> Self {
        Self { ast }
    }

    pub fn minify_quasis(
        quasis: &[TemplateElement<'a>],
        ast: AstBuilder<'a>,
    ) -> (Vec<Atom<'a>>, FxHashSet<usize>) {
        let minifier = Self::new(ast);

        let css = if quasis.len() == 1 {
            &quasis[0].value.raw
        } else {
            minifier.inject_unique_placeholders(quasis)
        };

        minifier.minify_css(css)
    }

    fn inject_unique_placeholders(&self, quasis: &[TemplateElement]) -> &str {
        let estimated_capacity: usize = quasis.iter().map(|s| s.value.raw.len()).sum::<usize>()
            + (quasis.len() - 1)
                * (Self::PLACEHOLDER_PREFIX.len() + Self::PLACEHOLDER_SUFFIX.len() + 2); // 2 for digits

        let mut result = StringBuilder::with_capacity_in(estimated_capacity, self.ast.allocator);

        for (index, val) in quasis.iter().enumerate() {
            result.push_str(&val.value.raw);
            if index < quasis.len() - 1 {
                result.push_str(Self::PLACEHOLDER_PREFIX);
                result.push_str(itoa::Buffer::new().format(index));
                result.push_str(Self::PLACEHOLDER_SUFFIX);
            }
        }

        result.into_str()
    }

    #[inline]
    fn try_parse_placeholder(bytes: &[u8], pos: usize) -> Option<(usize, usize)> {
        let mut i = pos + Self::PLACEHOLDER_PREFIX.len();

        let mut number = 0usize;
        let mut has_digits = false;

        while i < bytes.len() {
            let cur_byte = bytes[i];

            if !cur_byte.is_ascii_digit() {
                break;
            }

            number = number * 10 + (cur_byte - b'0') as usize;
            has_digits = true;
            i += 1;
        }

        if !has_digits ||
            // Checking suffix `__`
            !bytes[i..].starts_with(Self::PLACEHOLDER_SUFFIX.as_bytes())
        {
            return None;
        }

        Some((number, i + Self::PLACEHOLDER_SUFFIX.len()))
    }

    pub(super) fn minify_css(&self, css: &str) -> (Vec<Atom<'a>>, FxHashSet<usize>) {
        if css.trim().is_empty() {
            return (Vec::new(), FxHashSet::default());
        }

        let mut i = 0;
        let bytes = css.as_bytes();

        let mut output = Vec::new();
        let mut new_raws = Vec::new();
        let mut remaining_expression_indexes = FxHashSet::default();

        // Context state;
        let mut in_string: bool = false;
        let mut string_char: u8 = 0;
        let mut paren_depth: i32 = 0;

        while i < bytes.len() {
            let cur_byte = bytes[i];
            match cur_byte {
                // Handle string
                b'"' | b'\'' if !in_string => {
                    in_string = true;
                    string_char = cur_byte;
                }
                c if in_string && c == string_char && !Self::is_escaped(bytes, i) => {
                    in_string = false;
                }
                // Handle placeholders
                b'_' if bytes[i..].starts_with(Self::PLACEHOLDER_PREFIX.as_bytes()) => {
                    if let Some((number, new_index)) = Self::try_parse_placeholder(bytes, i) {
                        remaining_expression_indexes.insert(number);

                        new_raws.push(self.ast.atom(
                            // SAFETY: Output is all picked from the original `raw_values` and is guaranteed
                            // to be valid UTF-8.
                            unsafe { std::str::from_utf8_unchecked(&output) },
                        ));

                        // Clear output buffer, this is efficient as we reuse the same allocation.
                        output.clear();

                        i = new_index;
                        continue;
                    }
                }
                // Keep characters as-is if it's a part of a string
                _ if in_string => {}
                b'(' => {
                    paren_depth += 1;
                }
                b')' => {
                    paren_depth -= 1;
                }
                // Handle comments
                b'/' if i + 1 < bytes.len() => {
                    match bytes[i + 1] {
                        // Skip multiline comments except for `/*! ... */`
                        b'*' if i + 2 < bytes.len() && bytes[i + 2] != b'!' => {
                            i = Self::skip_multiline_comment(bytes, i);
                            // Adding a space when this is a own line block comment
                            if i < bytes.len()
                                && !bytes[i].is_ascii_whitespace()
                                && output.last().is_some_and(|&last| last != b' ')
                            {
                                output.push(b' ');
                            }
                            continue;
                        }
                        // Skip line comments, avoid `https://`
                        b'/' if paren_depth == 0 && (i == 0 || bytes[i - 1] != b':') => {
                            i = Self::skip_line_comment(bytes, i);
                            continue;
                        }
                        _ => {}
                    }
                }
                // Skip escaped newlines
                b'\\' if i + 1 < bytes.len() => {
                    let next_byte = bytes[i + 1];
                    if matches!(next_byte, b'n' | b'r') {
                        i += 2;
                        if output.last().is_some_and(|&last| last != b' ') {
                            output.push(b' ');
                        }
                        continue;
                    }
                }

                // Skip whitespace
                c if c.is_ascii_whitespace() => {
                    i += 1;
                    // Compress symbols, remove spaces around these symbols,
                    // but preserve whitespace preceding colon, to avoid joining selectors.
                    if output.last().is_some_and(|&last| {
                        !matches!(last, b' ' | b':' | b'{' | b'}' | b',' | b';')
                    }) && (i < bytes.len() && !matches!(bytes[i], b'{' | b'}' | b',' | b';'))
                    {
                        output.push(b' ');
                    }
                    continue;
                }
                _ => {}
            }

            output.push(cur_byte);
            i += 1;
        }

        // Handle remaining output
        new_raws.push(self.ast.atom(
            // SAFETY: Output is all picked from the original `raw_values` and is guaranteed
            // to be valid UTF-8.
            unsafe { std::str::from_utf8_unchecked(&output) },
        ));

        (new_raws, remaining_expression_indexes)
    }

    // Returns `true` if the character is escaped, `false` otherwise.
    fn is_escaped(bytes: &[u8], pos: usize) -> bool {
        if pos == 0 {
            return false;
        }

        let mut backslash_count = 0;
        let mut i = pos;

        while i > 0 && bytes[i - 1] == b'\\' {
            backslash_count += 1;
            i -= 1;
        }

        backslash_count % 2 == 1
    }

    #[inline]
    fn skip_multiline_comment(bytes: &[u8], start: usize) -> usize {
        let mut i = start + 2; // Skip /*

        while i + 1 < bytes.len() {
            if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                return i + 2;
            }
            i += 1;
        }

        i
    }

    #[inline]
    fn skip_line_comment(bytes: &[u8], start: usize) -> usize {
        let mut i = start;
        while i < bytes.len() && !matches!(bytes[i], b'\n' | b'\r') {
            i += 1;
        }
        i
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_allocator::Allocator;
    use oxc_ast::AstBuilder;

    fn minify_raw(input: &str) -> String {
        let allocator = Allocator::default();
        let ast = AstBuilder::new(&allocator);
        let (parts, _) = CssMinifier::new(ast).minify_css(input);

        if parts.is_empty() { String::new() } else { parts[0].to_string() }
    }

    mod strip_line_comment {
        use super::*;

        #[test]
        fn splits_line_by_potential_comment_starts() {
            let actual = minify_raw("abc def//ghi//jkl");
            debug_assert_eq!(actual, "abc def");
        }

        #[test]
        fn ignores_comment_markers_inside_strings() {
            let actual1 = minify_raw(r#"abc def"//"ghi'//'jkl//the end"#);
            debug_assert_eq!(actual1, r#"abc def"//"ghi'//'jkl"#);

            let actual2 = minify_raw(r#"abc def"//""#);
            debug_assert_eq!(actual2, r#"abc def"//""#);
        }

        #[test]
        fn ignores_comment_markers_inside_parentheses() {
            let actual = minify_raw("bla (//) bla//the end");
            debug_assert_eq!(actual, "bla (//) bla");
        }

        #[test]
        fn ignores_even_unescaped_urls() {
            let actual = minify_raw("https://test.com// comment//");
            debug_assert_eq!(actual, "https://test.com");
        }
    }

    mod minify_raw {
        use super::*;

        #[test]
        fn removes_multi_line_comments() {
            let input = "this is a/* ignore me please */test";
            let expected = "this is a test";
            let actual = minify_raw(input);
            debug_assert_eq!(actual, expected);
        }

        #[test]
        fn joins_all_lines_of_code() {
            let input = "this\nis\na/* ignore me \n please */\ntest";
            let expected = "this is a test";
            let actual = minify_raw(input);
            debug_assert_eq!(actual, expected);
        }

        #[test]
        fn removes_line_comments_filling_entire_line() {
            let input = "line one\n// remove this comment\nline two";
            let expected = "line one line two";
            let actual = minify_raw(input);
            debug_assert_eq!(actual, expected);
        }

        #[test]
        fn removes_line_comments_at_end_of_lines() {
            let input = "valid line with // a comment\nout comments";
            let expected = "valid line with out comments";
            let actual = minify_raw(input);
            debug_assert_eq!(actual, expected);
        }

        #[test]
        fn preserves_multi_line_comments_starting_with_bang() {
            let input = "this is a /*! dont ignore me please */ test/* but you can ignore me */";
            let expected = "this is a /*! dont ignore me please */ test";
            let actual = minify_raw(input);
            debug_assert_eq!(actual, expected);
        }

        #[test]
        fn returns_indices_of_removed_placeholders() {
            let allocator = Box::leak(Box::new(Allocator::default()));
            let ast = AstBuilder::new(allocator);

            // Create raw values that will generate placeholders
            let css = "this is some\ninput with __PLACEHOLDER_0__ and // ignored __PLACEHOLDER_1__";
            let (parts, remaining_indices) = CssMinifier::new(ast).minify_css(css);

            // Verify the content is properly minified
            debug_assert!(!parts.is_empty());
            let combined = parts.join("");
            debug_assert!(combined.contains("this is some input with"));
            debug_assert!(!combined.contains("// ignored"));

            debug_assert!(remaining_indices.contains(&0));
        }

        mod minify_raw_specific {
            use super::*;

            #[test]
            fn works_with_raw_escape_codes() {
                let input = "this\\nis\\na/* ignore me \\n please */\\ntest";
                let expected = "this is a test";
                let actual = minify_raw(input);
                debug_assert_eq!(actual, expected);
            }
        }

        mod compress_symbols {
            use super::*;

            #[test]
            fn removes_spaces_around_symbols() {
                let input = ";  :  {  }  ,  ;  ";
                let expected = ";:{},;";
                let actual = minify_raw(input);
                debug_assert_eq!(actual, expected);
            }

            #[test]
            fn ignores_symbols_inside_strings() {
                let input = r#";   " : " ' : ' ;"#;
                let expected = r#";" : " ' : ';"#;
                let actual = minify_raw(input);
                debug_assert_eq!(actual, expected);
            }

            #[test]
            fn preserves_whitespace_preceding_colons() {
                let input = "& :last-child { color: blue; }";
                let expected = "& :last-child{color:blue;}";
                let actual = minify_raw(input);
                debug_assert_eq!(actual, expected);
            }
        }
    }
}
