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

use std::{
    borrow::Cow,
    hash::{Hash, Hasher},
    iter::once,
};

use rustc_hash::{FxHashSet, FxHasher};
use serde::Deserialize;

use oxc_allocator::{TakeIn, Vec as ArenaVec};
use oxc_ast::{AstBuilder, NONE, ast::*};
use oxc_data_structures::inline_string::InlineString;
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
    Keyframes = 2,
    UseTheme = 3,
    WithTheme = 4,
    InjectGlobal = 5,
}

impl StyledComponentsHelper {
    /// Convert string to [`StyledComponentsHelper`].
    fn from_str(name: &str) -> Option<Self> {
        if name == "injectGlobal" { Some(Self::InjectGlobal) } else { Self::pure_from_str(name) }
    }

    /// Convert string to [`StyledComponentsHelper`], excluding `injectGlobal`.
    fn pure_from_str(name: &str) -> Option<Self> {
        match name {
            "createGlobalStyle" => Some(Self::CreateGlobalStyle),
            "css" => Some(Self::Css),
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
    component_id_prefix: Option<String>,
    /// Filename or directory name is used for `displayName`
    block_name: Option<Atom<'a>>,
}

impl<'a, 'ctx> StyledComponents<'a, 'ctx> {
    pub fn new(options: StyledComponentsOptions, ctx: &'ctx TransformCtx<'a>) -> Self {
        Self {
            options,
            ctx,
            styled_bindings: StyledComponentsBinding::default(),
            component_id_prefix: None,
            component_count: 0,
            block_name: None,
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

    #[inline] // Because it's a hot path, and most `Expression`s are not `TaggedTemplateExpression`s
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
                Ancestor::CallExpressionCallee(_) | Ancestor::StaticMemberExpressionObject(_) | Ancestor::ComputedMemberExpressionObject(_)
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

    /// Handles the pure annotation for pure helper calls
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

        // Update the quasis with the new raw values.
        for (new_raw, quasis) in new_raws.into_iter().zip(quasis.iter_mut()) {
            quasis.value.raw = new_raw;
        }

        // Keep expressions that are still present after minification.
        if expressions.len() != remained_expression_indices.len() {
            let mut i = 0;
            expressions.retain(|_| {
                let keep = remained_expression_indices.contains(&i);
                i += 1;
                keep
            });

            // SAFETY:
            // This is safe because template literal has ensured that `quasis` always
            // has one more element than `expressions`, and the `CSSMinifier` guarantees that
            // once a expression is removed, the corresponding quasi will also be removed.
            // Therefore, the length of `quasis` will always be one more than `expressions`.
            // So we can safely set the length of `quasis` to `expressions.len() + 1`.
            unsafe {
                // Set the length of `quasis` to `expressions.len() + 1` to truncate the quasis that
                // have been minified out.
                quasis.set_len(expressions.len() + 1);
            }
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

    /// Add `displayName` and `componentId` to `withConfig({})`
    ///
    /// If the call doesn't exist, then will create a new `withConfig` call expression
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

    /// Collects import bindings which imports from `styled-components`
    fn collect_styled_bindings(&mut self, program: &Program<'a>, _ctx: &mut TraverseCtx<'a>) {
        for statement in &program.body {
            let Statement::ImportDeclaration(import) = &statement else { continue };
            let Some(specifiers) = &import.specifiers else { continue };
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
                                    self.styled_bindings.set_helper_symbol_id(helper, symbol_id);
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

    /// Traverses the expression tree to find the `withConfig` call.
    fn get_with_config<'b>(expr: &'b mut Expression<'a>) -> Option<&'b mut CallExpression<'a>> {
        let mut current = expr;
        loop {
            match current {
                Expression::CallExpression(call) => {
                    if let Expression::StaticMemberExpression(member) = &call.callee {
                        if member.property.name == "withConfig" {
                            return Some(call);
                        }
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
            let value = self.get_component_id(ctx);
            properties.push(Self::create_object_property("componentId", value, ctx));
        }
    }

    // Infers the component name from the parent variable declarator, assignment expression,
    // or object property.
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

    /// `<namespace__>sc-<file_hash>-<component_count>`
    fn get_component_id(&mut self, ctx: &TraverseCtx<'a>) -> Atom<'a> {
        // Cache `<namespace__>sc-<file_hash>-` part as it's the same each time
        let prefix = if let Some(prefix) = self.component_id_prefix.as_deref() {
            prefix
        } else {
            const HASH_LEN: usize = 6;
            const PREFIX_LEN: usize = "sc-".len() + HASH_LEN + "-".len();
            const NAMESPACED_PREFIX_LEN: usize = "__".len() + PREFIX_LEN;

            let mut prefix = if let Some(namespace) = &self.options.namespace {
                let mut prefix = String::with_capacity(namespace.len() + NAMESPACED_PREFIX_LEN);
                prefix.extend([namespace, "__"]);
                prefix
            } else {
                String::with_capacity(PREFIX_LEN)
            };

            prefix.extend(["sc-", self.get_file_hash().as_str(), "-"]);

            self.component_id_prefix = Some(prefix);
            self.component_id_prefix.as_deref().unwrap()
        };

        // Add component count to end
        let mut buffer = itoa::Buffer::new();
        let count = buffer.format(self.component_count);
        self.component_count += 1;
        ctx.ast.atom_from_strs_array([prefix, count])
    }

    /// Generates a unique file hash based on the source path or source code.
    fn get_file_hash(&self) -> InlineString<7, u8> {
        #[inline]
        fn base36_encode(mut num: u64) -> InlineString<7, u8> {
            const BASE36_BYTES: &[u8; 36] = b"abcdefghijklmnopqrstuvwxyz0123456789";

            num %= 36_u64.pow(6); // 36^6, to ensure the result is <= 6 characters long.

            let mut str = InlineString::new();
            while num != 0 {
                // SAFETY: `num < 36.pow(6)` to start with, is and divided by 36 on each turn of loop,
                // so we cannot push more than 6 bytes. Capacity of `InlineString` is 7.
                // All bytes in `BASE36_BYTES` are ASCII.
                unsafe { str.push_unchecked(BASE36_BYTES[(num % 36) as usize]) };
                num /= 36;
            }
            str
        }

        let mut hasher = FxHasher::default();
        if self.ctx.source_path.is_absolute() {
            self.ctx.source_path.hash(&mut hasher);
        } else {
            self.ctx.source_text.hash(&mut hasher);
        }

        base36_encode(hasher.finish())
    }

    /// Returns the block name based on the file stem or parent directory name.
    fn get_block_name(&mut self, ctx: &TraverseCtx<'a>) -> Option<Atom<'a>> {
        if !self.options.file_name {
            return None;
        }

        let file_stem = self.ctx.source_path.file_stem().and_then(|stem| stem.to_str())?;

        Some(*self.block_name.get_or_insert_with(|| {
            // Should be a name, but if the file stem is in the meaningless file names list,
            // we will use the parent directory name instead.
            let block_name =
                if self.options.meaningless_file_names.iter().any(|name| name == file_stem) {
                    self.ctx
                        .source_path
                        .parent()
                        .and_then(|parent| parent.file_name())
                        .and_then(|name| name.to_str())
                        .unwrap_or(file_stem)
                } else {
                    file_stem
                };

            ctx.ast.atom(block_name)
        }))
    }

    /// Returns the display name which infers the component name or gets from the file name.
    fn get_display_name(&mut self, ctx: &TraverseCtx<'a>) -> Atom<'a> {
        let component_name = Self::get_component_name(ctx);

        let Some(block_name) = self.get_block_name(ctx) else {
            return component_name.unwrap_or(Atom::from(""));
        };

        if let Some(component_name) = component_name {
            if block_name == component_name {
                component_name
            } else {
                ctx.ast.atom_from_strs_array([&block_name, "__", &component_name])
            }
        } else {
            block_name
        }
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

    /// Checks if the identifier is a helper function of styled-components
    fn is_helper(&self, ident: &IdentifierReference<'a>, ctx: &TraverseCtx<'a>) -> bool {
        StyledComponentsHelper::from_str(&ident.name)
            .is_some_and(|helper| self.is_specific_helper(ident, helper, ctx))
    }

    /// Checks if the identifier is a pure helper function of styled-components
    fn is_pure_helper(&self, ident: &IdentifierReference<'a>, ctx: &TraverseCtx<'a>) -> bool {
        StyledComponentsHelper::pure_from_str(&ident.name)
            .is_some_and(|helper| self.is_specific_helper(ident, helper, ctx))
    }

    fn is_specific_helper(
        &self,
        ident: &IdentifierReference<'a>,
        helper: StyledComponentsHelper,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
        self.styled_bindings.helper_symbol_id(helper).is_some_and(|symbol_id| {
            let reference_id = ident.reference_id();
            ctx.scoping()
                .get_reference(reference_id)
                .symbol_id()
                .is_some_and(|reference_symbol_id| reference_symbol_id == symbol_id)
        })
    }

    /// Checks if the identifier is a reference to a styled binding.
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

    /// `{ key: value }`
    //     ^^^^^^^^^^
    fn create_object_property(
        key: &'static str,
        value: Atom<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> ObjectPropertyKind<'a> {
        let key = ctx.ast.property_key_static_identifier(SPAN, key);
        let value = ctx.ast.expression_string_literal(SPAN, value, None);
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

/// A CSS minifier that is specifically designed to minify CSS code within
/// styled-components template literals.
pub struct CssMinifier<'a> {
    ast: AstBuilder<'a>,
}

impl<'a> CssMinifier<'a> {
    const PLACEHOLDER_PREFIX: &'static str = "__PLACEHOLDER_";
    const PLACEHOLDER_SUFFIX: &'static str = "__";

    pub fn new(ast: AstBuilder<'a>) -> Self {
        Self { ast }
    }

    /// Minifies the CSS code within a series of template literal `quasis`.
    ///
    /// This function takes an array of `TemplateElement` (quasis) and
    /// first injects unique placeholders for the expressions between them.
    /// Then, passes the processed CSS string to the [CssMinifier::minify_css].
    ///
    /// ### Returns:
    ///
    /// A tuple containing:
    /// First: A vector of `Atom` containing the minified CSS strings.
    /// Second: A set of indices representing which expressions were kept.
    pub fn minify_quasis(
        quasis: &[TemplateElement<'a>],
        ast: AstBuilder<'a>,
    ) -> (Vec<Atom<'a>>, FxHashSet<usize>) {
        let minifier = Self::new(ast);

        let css = if quasis.len() == 1 {
            Cow::Borrowed(quasis[0].value.raw.as_str())
        } else {
            Cow::Owned(Self::inject_unique_placeholders(quasis))
        };

        minifier.minify_css(&css)
    }

    /// Injects unique placeholders into a series of `quasis` to represent expressions.
    ///
    /// This is a key step for minification. By replacing expressions with placeholders,
    /// we can treat the entire template literal as a single block of CSS, which simplifies
    /// the minification process. The placeholders are designed to be unique and unlikely
    /// to appear in regular CSS.
    ///
    /// # Example
    ///
    /// Given quasis from `` `width: ${width}px; color: ${color};` ``, which are
    /// `["width: ", "px; color: ", ";"]`, and expressions `[width, color]`,
    /// this function will produce the string:
    /// `"width: __PLACEHOLDER_0__px; color: __PLACEHOLDER_1__;"`
    fn inject_unique_placeholders(quasis: &[TemplateElement]) -> String {
        let estimated_capacity: usize = quasis.iter().map(|s| s.value.raw.len()).sum::<usize>()
            + (quasis.len() - 1)
                * (Self::PLACEHOLDER_PREFIX.len() + Self::PLACEHOLDER_SUFFIX.len() + 2); // 2 for digits

        let mut result = String::with_capacity(estimated_capacity);

        for (index, val) in quasis.iter().enumerate() {
            result.push_str(&val.value.raw);
            if index < quasis.len() - 1 {
                result.extend([
                    Self::PLACEHOLDER_PREFIX,
                    itoa::Buffer::new().format(index),
                    Self::PLACEHOLDER_SUFFIX,
                ]);
            }
        }

        result
    }

    /// Tries to parse a placeholder like `__PLACEHOLDER_0__` from a byte slice.
    ///
    /// This function is used by the minifier to detect where expressions were
    /// in the original template literal.
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

    /// The core CSS minification logic.
    ///
    /// This function iterates through the CSS string (as bytes) and applies minification
    /// rules. It handles strings, comments, and whitespace, and splits the output
    /// by the placeholders it finds.
    ///
    /// ### Steps:
    ///
    /// 1. It initializes state variables to track whether it's inside a string,
    ///    the current string character, and the depth of parentheses.
    /// 2. It iterates through each byte of the CSS string:
    ///   - If it encounters a string quote (`"` or `'`), it toggles the `in_string` state.
    ///   - If it finds a placeholder (starting with `_` and followed by
    ///     [CssMinifier::PLACEHOLDER_PREFIX]), it tries to parse the placeholder and
    ///     keeps track of its index.
    ///   - It skips comments (both block and line comments) and whitespace, compressing them
    ///     as needed.
    ///   - It handles escaped newlines by replacing them with a space.
    /// 3. It collects the minified CSS parts into `new_raws` and tracks which
    ///    expressions were kept in a `remaining_expression_indexes`.
    /// 4. Finally, it returns the minified CSS parts and the set of kept expression indices.
    ///
    /// The step 2 is the key part of the minification process, it tries to split the CSS which
    /// joins by the placeholders that is injected by [CssMinifier::inject_unique_placeholders].
    ///
    /// ### Example:
    ///
    /// For a case like:
    /// ```js
    /// styled.div`
    ///   width: ${width}px;
    ///   color: red; // color: ${color};
    ///   height: 100px;
    /// `
    /// // quasis: ["width: ", "px;\n   color: red; // color:", ";\n   height: 100px;"]
    /// // expressions: [width, color]
    /// ```
    ///
    /// We will receive a CSS string which produced by [CssMinifier::inject_unique_placeholders] like:
    /// ```js
    /// "width: __PLACEHOLDER_0__px;
    /// // color: __PLACEHOLDER_1__;
    /// height: 100px;"
    /// ```
    ///
    /// The result of quasis and expressions will be:
    ///
    /// ```rust
    /// (vec!["width:", "px;color:red;height:100px;"], { 0 })
    /// ```
    ///
    /// Only the first expression (`width`) is kept, and the second one (`color`) is removed
    /// because it was inside a comment.
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
                // This is where we detect the placeholders we injected earlier.
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
                        // Skip line comments, but be careful not to break URLs like `https://...`
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

                // Skip and compress whitespace.
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

        // Add any remaining text after no more placeholders.
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

    /// Skips a multiline comment `/* ... */`.
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

    /// Skips a line comment `// ...`
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
            let allocator = Allocator::default();
            let ast = AstBuilder::new(&allocator);

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
