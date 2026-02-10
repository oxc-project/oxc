use std::{cmp::Ordering, sync::Arc};

use oxc_ecmascript::BoundNames;
use rustc_hash::FxHashSet;

use oxc_allocator::{Address, Allocator, GetAddress, UnstableAddress};
use oxc_ast::ast::*;
use oxc_ast_visit::{VisitMut, walk_mut};
use oxc_diagnostics::OxcDiagnostic;
use oxc_parser::Parser;
use oxc_semantic::ScopeFlags;
use oxc_span::{CompactStr, SPAN, SourceType};
use oxc_syntax::identifier::is_identifier_name;

/// Lightweight scope binding tracker that replaces full `Scoping`.
///
/// Tracks binding names in a stack of scopes during the VisitMut walk, so we can determine
/// whether an identifier is a global reference without needing pre-built scoping data.
pub struct ScopeBindingTracker<'a> {
    /// Stack of scopes, each containing bound names.
    /// The first entry is always the program/module scope.
    pub(crate) scopes: Vec<FxHashSet<&'a str>>,
    /// Indices of function scopes in the `scopes` stack (for `var` hoisting).
    pub(crate) function_scope_indices: Vec<usize>,
    /// Names declared with `declare` keyword (ambient declarations like `declare const`).
    ambient_names: FxHashSet<&'a str>,
}

impl<'a> ScopeBindingTracker<'a> {
    pub(crate) fn new() -> Self {
        Self {
            scopes: vec![FxHashSet::default()],
            function_scope_indices: vec![0],
            ambient_names: FxHashSet::default(),
        }
    }

    /// Enter a new scope.
    pub(crate) fn enter_scope(&mut self, is_function: bool) {
        let idx = self.scopes.len();
        self.scopes.push(FxHashSet::default());
        if is_function {
            self.function_scope_indices.push(idx);
        }
    }

    /// Leave the current scope.
    pub(crate) fn leave_scope(&mut self, is_function: bool) {
        self.scopes.pop();
        if is_function {
            self.function_scope_indices.pop();
        }
    }

    /// Add a binding to the current scope (`let`, `const`, function params, etc.).
    pub(crate) fn add_binding(&mut self, name: &'a str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name);
        }
    }

    /// Add a `var` binding, which hoists to the nearest function scope.
    pub(crate) fn add_var_binding(&mut self, name: &'a str) {
        if let Some(&idx) = self.function_scope_indices.last() {
            self.scopes[idx].insert(name);
        }
    }

    /// Mark a name as an ambient declaration (`declare const`).
    pub(crate) fn add_ambient(&mut self, name: &'a str) {
        self.ambient_names.insert(name);
    }

    /// Check if a name is bound in any enclosing scope.
    /// Returns `true` if the name is NOT bound (i.e., is a global reference).
    pub(crate) fn is_global(&self, name: &str) -> bool {
        !self.scopes.iter().any(|scope| scope.contains(name))
    }

    /// Check if a name is an ambient declaration (e.g., `declare const`).
    pub(crate) fn is_ambient(&self, name: &str) -> bool {
        self.ambient_names.contains(name)
    }
}

/// Configuration for [ReplaceGlobalDefines].
///
/// Due to the usage of an arena allocator, the constructor will parse once for grammatical errors,
/// and does not save the constructed expression.
///
/// The data is stored in an `Arc` so this can be shared across threads.
#[derive(Debug, Clone)]
pub struct ReplaceGlobalDefinesConfig(Arc<ReplaceGlobalDefinesConfigImpl>);

static THIS_ATOM: Atom<'static> = Atom::new_const("this");

#[derive(Debug)]
struct IdentifierDefine {
    identifier_defines: Vec<(/* key */ CompactStr, /* value */ CompactStr)>,
    /// Whether user want to replace `ThisExpression`, avoid linear scan for each `ThisExpression`
    has_this_expr_define: bool,
}
#[derive(Debug)]
struct ReplaceGlobalDefinesConfigImpl {
    identifier: IdentifierDefine,
    dot: Vec<DotDefine>,
    meta_property: Vec<MetaPropertyDefine>,
    /// extra field to avoid linear scan `meta_property` to check if it has `import.meta` every
    /// time
    /// Some(replacement): import.meta -> replacement
    /// None -> no need to replace import.meta
    import_meta: Option<CompactStr>,
}

#[derive(Debug)]
pub struct DotDefine {
    /// Member expression parts
    pub parts: Vec<CompactStr>,
    pub value: CompactStr,
}

#[derive(Debug)]
pub struct MetaPropertyDefine {
    /// only store parts after `import.meta`
    pub parts: Vec<CompactStr>,
    pub value: CompactStr,
    pub postfix_wildcard: bool,
}

impl MetaPropertyDefine {
    pub fn new(parts: Vec<CompactStr>, value: CompactStr, postfix_wildcard: bool) -> Self {
        Self { parts, value, postfix_wildcard }
    }
}

impl DotDefine {
    fn new(parts: Vec<CompactStr>, value: CompactStr) -> Self {
        Self { parts, value }
    }
}

enum IdentifierType {
    Identifier,
    DotDefines { parts: Vec<CompactStr> },
    // import.meta.a
    ImportMetaWithParts { parts: Vec<CompactStr>, postfix_wildcard: bool },
    // import.meta or import.meta.*
    ImportMeta(bool),
}

impl ReplaceGlobalDefinesConfig {
    /// # Errors
    ///
    /// * key is not an identifier
    /// * value has a syntax error
    pub fn new<S: AsRef<str>>(defines: &[(S, S)]) -> Result<Self, Vec<OxcDiagnostic>> {
        let allocator = Allocator::default();
        let mut identifier_defines = vec![];
        let mut dot_defines = vec![];
        let mut meta_properties_defines = vec![];
        let mut import_meta = None;
        let mut has_this_expr_define = false;
        for (key, value) in defines {
            let key = key.as_ref();

            let value = value.as_ref();
            Self::check_value(&allocator, value)?;

            match Self::check_key(key)? {
                IdentifierType::Identifier => {
                    has_this_expr_define |= key == "this";
                    identifier_defines.push((CompactStr::new(key), CompactStr::new(value)));
                }
                IdentifierType::DotDefines { parts } => {
                    dot_defines.push(DotDefine::new(parts, CompactStr::new(value)));
                }
                IdentifierType::ImportMetaWithParts { parts, postfix_wildcard } => {
                    meta_properties_defines.push(MetaPropertyDefine::new(
                        parts,
                        CompactStr::new(value),
                        postfix_wildcard,
                    ));
                }
                IdentifierType::ImportMeta(postfix_wildcard) => {
                    if postfix_wildcard {
                        meta_properties_defines.push(MetaPropertyDefine::new(
                            vec![],
                            CompactStr::new(value),
                            postfix_wildcard,
                        ));
                    } else {
                        import_meta = Some(CompactStr::new(value));
                    }
                }
            }
        }
        // Always move specific meta define before wildcard dot define
        // Keep other order unchanged
        // see test case replace_global_definitions_dot_with_postfix_mixed as an example
        meta_properties_defines.sort_by(|a, b| {
            if !a.postfix_wildcard && b.postfix_wildcard {
                Ordering::Less
            } else if a.postfix_wildcard && b.postfix_wildcard {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });
        Ok(Self(Arc::new(ReplaceGlobalDefinesConfigImpl {
            identifier: IdentifierDefine { identifier_defines, has_this_expr_define },
            dot: dot_defines,
            meta_property: meta_properties_defines,
            import_meta,
        })))
    }

    fn check_key(key: &str) -> Result<IdentifierType, Vec<OxcDiagnostic>> {
        let parts: Vec<&str> = key.split('.').collect();

        assert!(!parts.is_empty());

        if parts.len() == 1 {
            if !is_identifier_name(parts[0]) {
                return Err(vec![OxcDiagnostic::error(format!(
                    "The define key `{key}` is not an identifier."
                ))]);
            }
            return Ok(IdentifierType::Identifier);
        }
        let normalized_parts_len =
            if parts[parts.len() - 1] == "*" { parts.len() - 1 } else { parts.len() };
        // We can ensure now the parts.len() >= 2
        let is_import_meta = parts[0] == "import" && parts[1] == "meta";

        for part in &parts[0..normalized_parts_len] {
            if !is_identifier_name(part) {
                return Err(vec![OxcDiagnostic::error(format!(
                    "The define key `{key}` contains an invalid identifier `{part}`."
                ))]);
            }
        }
        if is_import_meta {
            match normalized_parts_len {
                2 => Ok(IdentifierType::ImportMeta(normalized_parts_len != parts.len())),
                _ => Ok(IdentifierType::ImportMetaWithParts {
                    parts: parts
                        .iter()
                        .skip(2)
                        .take(normalized_parts_len - 2)
                        .map(|s| CompactStr::new(s))
                        .collect(),
                    postfix_wildcard: normalized_parts_len != parts.len(),
                }),
            }
        // StaticMemberExpression with postfix wildcard
        } else if normalized_parts_len != parts.len() {
            Err(vec![OxcDiagnostic::error(
                "The postfix wildcard is only allowed for `import.meta`.".to_string(),
            )])
        } else {
            Ok(IdentifierType::DotDefines {
                parts: parts
                    .iter()
                    .take(normalized_parts_len)
                    .map(|s| CompactStr::new(s))
                    .collect(),
            })
        }
    }

    fn check_value(allocator: &Allocator, source_text: &str) -> Result<(), Vec<OxcDiagnostic>> {
        Parser::new(allocator, source_text, SourceType::default()).parse_expression()?;
        Ok(())
    }
}

#[must_use]
pub struct ReplaceGlobalDefinesReturn {
    pub changed: bool,
}

/// Replace Global Defines.
///
/// References:
///
/// * <https://esbuild.github.io/api/#define>
/// * <https://github.com/terser/terser?tab=readme-ov-file#conditional-compilation>
/// * <https://github.com/evanw/esbuild/blob/9c13ae1f06dfa909eb4a53882e3b7e4216a503fe/internal/config/globals.go#L852-L1014>
pub struct ReplaceGlobalDefines<'a> {
    allocator: &'a Allocator,
    config: ReplaceGlobalDefinesConfig,
    /// Since `VisitMut` does not provide a way to skip visiting a sub tree of the AstNode,
    /// Use `Option<Address>` to lock the current node when it is `Some`.
    /// During visiting sub tree, the `Lock` will always be `Some`, and we can early return, this
    /// could achieve same effect as skipping visiting sub tree.
    /// When exiting the node, reset the `Lock` to `None` to make sure not affect other
    /// transformation.
    ast_node_lock: Option<Address>,
    changed: bool,
    /// Lightweight scope binding tracker.
    scope_tracker: ScopeBindingTracker<'a>,
    /// Depth of non-arrow functions we're inside of. Used to compute scope flags for `this`
    /// replacement.
    non_arrow_function_depth: u32,
    /// Destructuring keys from the parent `VariableDeclarator` when its `id` is an
    /// `ObjectPattern`. Used to optimize object expression replacements by only keeping needed
    /// keys.
    destructuring_keys: Option<FxHashSet<CompactStr>>,
}

impl<'a> VisitMut<'a> for ReplaceGlobalDefines<'a> {
    fn enter_scope(
        &mut self,
        flags: ScopeFlags,
        _scope_id: &std::cell::Cell<Option<oxc_syntax::scope::ScopeId>>,
    ) {
        let is_function = flags.contains(ScopeFlags::Function);
        self.scope_tracker.enter_scope(is_function);
    }

    fn leave_scope(&mut self) {
        // We need to know if the scope being left is a function scope.
        // The simplest way: check if the current scope index matches the top
        // of the function_scope_indices stack.
        let current_idx = self.scope_tracker.scopes.len() - 1;
        let is_function =
            self.scope_tracker.function_scope_indices.last().is_some_and(|&i| i == current_idx);
        self.scope_tracker.leave_scope(is_function);
    }

    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        if self.ast_node_lock.is_some() {
            return;
        }
        let is_replaced = self.replace_identifier_defines(expr) || self.replace_dot_defines(expr);
        // Clear `destructuring_keys` after checking the first expression so the
        // optimization only applies to the direct init expression of a
        // destructuring `VariableDeclarator`, not to nested sub-expressions
        // (e.g. `const { a } = foo(DEFINE)` should not optimize the replacement
        // inside the call).
        self.destructuring_keys = None;
        if is_replaced {
            self.mark_as_changed();
            self.ast_node_lock = Some(expr.address());
        }
        walk_mut::walk_expression(self, expr);
        if self.ast_node_lock == Some(expr.address()) {
            self.ast_node_lock = None;
        }
    }

    fn visit_assignment_expression(&mut self, node: &mut AssignmentExpression<'a>) {
        if self.ast_node_lock.is_some() {
            return;
        }
        if self.replace_define_with_assignment_expr(node) {
            self.mark_as_changed();
            // `AssignmentExpression` is stored in a `Box`, so has a stable memory location
            self.ast_node_lock = Some(node.unstable_address());
        }
        walk_mut::walk_assignment_expression(self, node);
        // `AssignmentExpression` is stored in a `Box`, so has a stable memory location
        if self.ast_node_lock == Some(node.unstable_address()) {
            self.ast_node_lock = None;
        }
    }

    fn visit_function(&mut self, func: &mut Function<'a>, flags: ScopeFlags) {
        // Add function name to the ENCLOSING scope (not the function's own scope).
        // Function declarations bind their name in the enclosing scope.
        if let Some(id) = &func.id {
            self.scope_tracker.add_binding(id.name.as_str());
        }
        self.non_arrow_function_depth += 1;
        // walk_function will call enter_scope for the function scope, then visit params+body.
        // Params are visited inside the function scope, so they'll be added there.
        walk_mut::walk_function(self, func, flags);
        self.non_arrow_function_depth -= 1;
    }

    fn visit_class(&mut self, class: &mut Class<'a>) {
        // Add class name to the ENCLOSING scope.
        if let Some(id) = &class.id {
            self.scope_tracker.add_binding(id.name.as_str());
        }
        walk_mut::walk_class(self, class);
    }

    fn visit_formal_parameter(&mut self, param: &mut FormalParameter<'a>) {
        // Add parameter names to the current scope (function scope).
        param.pattern.bound_names(&mut |ident| {
            self.scope_tracker.add_binding(ident.name.as_str());
        });
        walk_mut::walk_formal_parameter(self, param);
    }

    fn visit_variable_declaration(&mut self, decl: &mut VariableDeclaration<'a>) {
        // Collect bindings and handle `var` hoisting.
        let is_var = decl.kind.is_var();
        let is_ambient = decl.declare;
        decl.bound_names(&mut |ident| {
            let name = ident.name.as_str();
            if is_ambient {
                self.scope_tracker.add_ambient(name);
            }
            if is_var {
                self.scope_tracker.add_var_binding(name);
            } else {
                self.scope_tracker.add_binding(name);
            }
        });
        // Still need destructuring key optimization for VariableDeclarator.
        walk_mut::walk_variable_declaration(self, decl);
    }

    fn visit_variable_declarator(&mut self, declarator: &mut VariableDeclarator<'a>) {
        // Collect destructuring keys if LHS is an ObjectPattern.
        // `visit_expression` clears `destructuring_keys` after the first expression check,
        // ensuring the optimization only applies to the direct init, not nested sub-expressions.
        if let BindingPattern::ObjectPattern(pat) = &declarator.id {
            let mut keys = FxHashSet::default();
            let mut all_static = true;
            for prop in &pat.properties {
                if let Some(key) = prop.key.name() {
                    keys.insert(CompactStr::from(key.as_ref()));
                } else {
                    all_static = false;
                    break;
                }
            }
            if all_static && !keys.is_empty() {
                self.destructuring_keys = Some(keys);
            }
        }
        walk_mut::walk_variable_declarator(self, declarator);
    }

    fn visit_catch_clause(&mut self, clause: &mut CatchClause<'a>) {
        // Catch param binds in the catch clause scope (entered by walk_catch_clause).
        walk_mut::walk_catch_clause(self, clause);
    }

    fn visit_import_declaration(&mut self, decl: &mut ImportDeclaration<'a>) {
        // Import bindings are in the module scope.
        decl.bound_names(&mut |ident| {
            self.scope_tracker.add_binding(ident.name.as_str());
        });
        walk_mut::walk_import_declaration(self, decl);
    }
}

impl<'a> ReplaceGlobalDefines<'a> {
    pub fn new(allocator: &'a Allocator, config: ReplaceGlobalDefinesConfig) -> Self {
        Self {
            allocator,
            config,
            ast_node_lock: None,
            changed: false,
            scope_tracker: ScopeBindingTracker::new(),
            non_arrow_function_depth: 0,
            destructuring_keys: None,
        }
    }

    fn mark_as_changed(&mut self) {
        self.changed = true;
    }

    pub fn build(&mut self, program: &mut Program<'a>) -> ReplaceGlobalDefinesReturn {
        self.visit_program(program);
        ReplaceGlobalDefinesReturn { changed: self.changed }
    }

    // Construct a new expression because we don't have ast clone right now.
    fn parse_value(&self, source_text: &str) -> Expression<'a> {
        // Allocate the string lazily because replacement happens rarely.
        let source_text = self.allocator.alloc_str(source_text);
        // Unwrapping here, it should already be checked by [ReplaceGlobalDefinesConfig::new].
        let mut expr = Parser::new(self.allocator, source_text, SourceType::default())
            .parse_expression()
            .unwrap();

        ClearSpans.visit_expression(&mut expr);

        expr
    }

    fn replace_identifier_defines(&self, expr: &mut Expression<'a>) -> bool {
        match expr {
            Expression::Identifier(ident) => {
                if let Some(new_expr) = self.replace_identifier_define_impl(ident) {
                    *expr = new_expr;
                    return true;
                }
            }
            Expression::ThisExpression(_)
                if self.config.0.identifier.has_this_expr_define
                    && should_replace_this_expr(self.current_scope_flags()) =>
            {
                for (key, value) in &self.config.0.identifier.identifier_defines {
                    if key.as_str() == "this" {
                        let value = value.clone();
                        let value = self.parse_value(&value);
                        *expr = value;

                        return true;
                    }
                }
            }
            _ => {}
        }
        false
    }

    fn replace_identifier_define_impl(
        &self,
        ident: &oxc_allocator::Box<'_, IdentifierReference<'_>>,
    ) -> Option<Expression<'a>> {
        let name = ident.name.as_str();
        // If the name is bound in an enclosing scope (not global), skip replacement.
        // Exception: ambient declarations (`declare const`) should still be replaced.
        if !self.scope_tracker.is_global(name) && !self.scope_tracker.is_ambient(name) {
            return None;
        }
        // This is a global variable, including ambient variants such as `declare const`.
        for (key, value) in &self.config.0.identifier.identifier_defines {
            if name == key {
                let value = value.clone();
                let value = self.parse_value(&value);
                return Some(value);
            }
        }
        None
    }

    fn replace_define_with_assignment_expr(&self, node: &mut AssignmentExpression<'a>) -> bool {
        let new_left = node
            .left
            .as_simple_assignment_target_mut()
            .and_then(|item| match item {
                SimpleAssignmentTarget::ComputedMemberExpression(computed_member_expr) => {
                    self.replace_dot_computed_member_expr(computed_member_expr)
                }
                SimpleAssignmentTarget::StaticMemberExpression(member) => {
                    self.replace_dot_static_member_expr_no_optimize(member)
                }
                SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
                    self.replace_identifier_define_impl(ident)
                }
                _ => None,
            })
            .and_then(assignment_target_from_expr);
        if let Some(new_left) = new_left {
            node.left = new_left;
            return true;
        }
        false
    }

    fn replace_dot_defines(&self, expr: &mut Expression<'a>) -> bool {
        match expr {
            Expression::ChainExpression(chain) => {
                let Some(new_expr) =
                    chain.expression.as_member_expression_mut().and_then(|item| match item {
                        MemberExpression::ComputedMemberExpression(computed_member_expr) => {
                            self.replace_dot_computed_member_expr(computed_member_expr)
                        }
                        MemberExpression::StaticMemberExpression(member) => {
                            self.replace_dot_static_member_expr(member)
                        }
                        MemberExpression::PrivateFieldExpression(_) => None,
                    })
                else {
                    return false;
                };
                *expr = new_expr;
                return true;
            }
            Expression::StaticMemberExpression(member) => {
                if let Some(new_expr) = self.replace_dot_static_member_expr(member) {
                    *expr = new_expr;
                    return true;
                }
            }
            Expression::ComputedMemberExpression(member) => {
                if let Some(new_expr) = self.replace_dot_computed_member_expr(member) {
                    *expr = new_expr;
                    return true;
                }
            }
            Expression::MetaProperty(meta_property) => {
                if let Some(replacement) = self.config.0.import_meta.clone()
                    && meta_property.meta.name == "import"
                    && meta_property.property.name == "meta"
                {
                    let value = self.parse_value(&replacement);
                    *expr = value;
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    fn replace_dot_computed_member_expr(
        &self,
        member: &ComputedMemberExpression<'a>,
    ) -> Option<Expression<'a>> {
        let scope_tracker = &self.scope_tracker;
        let scope_flags = self.current_scope_flags();
        let value = self
            .config
            .0
            .dot
            .iter()
            .find(|dot_define| {
                Self::is_dot_define(
                    scope_tracker,
                    scope_flags,
                    dot_define,
                    DotDefineMemberExpression::ComputedMemberExpression(member),
                )
            })
            .map(|dot_define| dot_define.value.clone());
        if let Some(value) = value {
            let value = self.parse_value(&value);
            return Some(value);
        }
        // TODO: meta_property_define
        None
    }

    fn replace_dot_static_member_expr(
        &self,
        member: &StaticMemberExpression<'a>,
    ) -> Option<Expression<'a>> {
        let scope_tracker = &self.scope_tracker;
        let scope_flags = self.current_scope_flags();
        let value = self
            .config
            .0
            .dot
            .iter()
            .find(|dot_define| {
                Self::is_dot_define(
                    scope_tracker,
                    scope_flags,
                    dot_define,
                    DotDefineMemberExpression::StaticMemberExpression(member),
                )
            })
            .map(|dot_define| dot_define.value.clone());
        if let Some(value) = value {
            let value = self.parse_value(&value);
            return Some(self.destructing_dot_define_optimizer(value));
        }
        let value = self
            .config
            .0
            .meta_property
            .iter()
            .find(|meta_property_define| {
                Self::is_meta_property_define(meta_property_define, member)
            })
            .map(|meta_property_define| meta_property_define.value.clone());
        if let Some(value) = value {
            let value = self.parse_value(&value);
            return Some(self.destructing_dot_define_optimizer(value));
        }
        None
    }

    /// Like `replace_dot_static_member_expr` but without the destructuring optimization.
    /// Used for assignment targets where we don't have a parent destructuring pattern.
    fn replace_dot_static_member_expr_no_optimize(
        &self,
        member: &StaticMemberExpression<'a>,
    ) -> Option<Expression<'a>> {
        let scope_tracker = &self.scope_tracker;
        let scope_flags = self.current_scope_flags();
        let value = self
            .config
            .0
            .dot
            .iter()
            .find(|dot_define| {
                Self::is_dot_define(
                    scope_tracker,
                    scope_flags,
                    dot_define,
                    DotDefineMemberExpression::StaticMemberExpression(member),
                )
            })
            .map(|dot_define| dot_define.value.clone());
        if let Some(value) = value {
            let value = self.parse_value(&value);
            return Some(value);
        }
        let value = self
            .config
            .0
            .meta_property
            .iter()
            .find(|meta_property_define| {
                Self::is_meta_property_define(meta_property_define, member)
            })
            .map(|meta_property_define| meta_property_define.value.clone());
        if let Some(value) = value {
            let value = self.parse_value(&value);
            return Some(value);
        }
        None
    }

    pub fn is_meta_property_define(
        meta_define: &MetaPropertyDefine,
        member: &StaticMemberExpression<'a>,
    ) -> bool {
        enum WildCardStatus {
            None,
            Pending,
            Matched,
        }
        if meta_define.parts.is_empty() && meta_define.postfix_wildcard {
            match &member.object {
                Expression::MetaProperty(meta) => {
                    return meta.meta.name == "import" && meta.property.name == "meta";
                }
                _ => return false,
            }
        }
        debug_assert!(!meta_define.parts.is_empty());

        let mut current_part_member_expression = Some(member);
        let mut cur_part_name: &str = &member.property.name;
        let mut is_full_match = true;
        let mut i = meta_define.parts.len() - 1;
        let mut has_matched_part = false;
        let mut wildcard_status = if meta_define.postfix_wildcard {
            WildCardStatus::Pending
        } else {
            WildCardStatus::None
        };
        loop {
            let part = &meta_define.parts[i];
            let matched = cur_part_name == part;
            if matched {
                has_matched_part = true;
            } else {
                is_full_match = false;
                // Considering import.meta.env.*
                // ```js
                // import.meta.env.test // should matched
                // import.res.meta.env // should not matched
                // ```
                // So we use has_matched_part to track if any part has matched.
                // `None` means there is no postfix wildcard defined, so any part not matched should return false
                // `Matched` means there is a postfix wildcard defined, and already matched a part, so any further
                // not matched part should return false
                if matches!(wildcard_status, WildCardStatus::None | WildCardStatus::Matched)
                    || has_matched_part
                {
                    return false;
                }
                wildcard_status = WildCardStatus::Matched;
            }

            current_part_member_expression = if let Some(member) = current_part_member_expression {
                match &member.object {
                    Expression::StaticMemberExpression(member) => {
                        cur_part_name = &member.property.name;
                        Some(member)
                    }
                    Expression::MetaProperty(_) => {
                        if meta_define.postfix_wildcard {
                            // `import.meta.env` should not match `import.meta.env.*`
                            return has_matched_part && !is_full_match;
                        }
                        return i == 0;
                    }
                    Expression::Identifier(_) => {
                        return false;
                    }
                    _ => None,
                }
            } else {
                return false;
            };

            // Config `import.meta.env.* -> 'undefined'`
            // Considering try replace `import.meta.env` to `undefined`, for the first loop the i is already
            // 0, if it did not match part name and still reach here, that means
            // current_part_member_expression is still something, and possible to match in the
            // further loop
            if i == 0 && matched {
                break;
            }

            if matched {
                i -= 1;
            }
        }

        false
    }

    /// Compute the current scope flags based on function depth tracking.
    fn current_scope_flags(&self) -> ScopeFlags {
        if self.non_arrow_function_depth > 0 { ScopeFlags::Function } else { ScopeFlags::Top }
    }

    pub(crate) fn is_dot_define<'b>(
        scope_tracker: &ScopeBindingTracker<'a>,
        scope_flags: ScopeFlags,
        dot_define: &DotDefine,
        member: DotDefineMemberExpression<'b, 'a>,
    ) -> bool {
        debug_assert!(dot_define.parts.len() > 1);
        let should_replace_this_expr = should_replace_this_expr(scope_flags);
        let Some(cur_part_name) = member.name() else {
            return false;
        };
        let mut cur_part_name: &str = cur_part_name.as_str();
        let mut current_part_member_expression = Some(member);

        for (i, part) in dot_define.parts.iter().enumerate().rev() {
            if cur_part_name != part {
                return false;
            }
            if i == 0 {
                break;
            }

            current_part_member_expression = if let Some(member) = current_part_member_expression {
                match &member.object() {
                    Expression::StaticMemberExpression(member) => {
                        cur_part_name = &member.property.name;
                        Some(DotDefineMemberExpression::StaticMemberExpression(member))
                    }
                    Expression::ComputedMemberExpression(computed_member) => {
                        static_property_name_of_computed_expr(computed_member).map(|name| {
                            cur_part_name = name.as_str();
                            DotDefineMemberExpression::ComputedMemberExpression(computed_member)
                        })
                    }
                    Expression::Identifier(ident) => {
                        if !scope_tracker.is_global(&ident.name) {
                            return false;
                        }
                        cur_part_name = &ident.name;
                        None
                    }
                    Expression::ThisExpression(_) if should_replace_this_expr => {
                        cur_part_name = THIS_ATOM.as_str();
                        None
                    }
                    Expression::MetaProperty(meta) => {
                        // Handle import.meta
                        // When we encounter a MetaProperty, we need to verify that the remaining
                        // parts match ["import", "meta"]
                        if meta.meta.name == "import" && meta.property.name == "meta" {
                            // At this point, i is the current position we're checking
                            // We need the next two parts (going backwards) to be "meta" then "import"
                            // i.e., parts[i-1] == "meta" and parts[i-2] == "import"
                            if i >= 2
                                && dot_define.parts[i - 1].as_str() == "meta"
                                && dot_define.parts[i - 2].as_str() == "import"
                            {
                                // Successfully matched import.meta at the expected position
                                // Return true if we've consumed all parts (i == 2)
                                return i == 2;
                            }
                        }
                        None
                    }
                    _ => None,
                }
            } else {
                return false;
            };
        }

        current_part_member_expression.is_none()
    }

    /// Optimize object expression replacements in destructuring patterns by only keeping needed
    /// keys.
    fn destructing_dot_define_optimizer(&self, mut expr: Expression<'a>) -> Expression<'a> {
        let Expression::ObjectExpression(obj) = &mut expr else { return expr };
        let Some(needed_keys) = &self.destructuring_keys else { return expr };

        // here we iterate the object properties twice
        // for the first time we check if all the keys are static
        // for the second time we only keep the needed keys
        // Another way to do this is mutate the objectExpr only the fly,
        // but need to save the checkpoint(to return the original Expr if there are any dynamic key exists) which is a memory clone,
        // cpu is faster than memory allocation
        let mut should_preserved_keys = Vec::with_capacity(obj.properties.len());
        for prop in &obj.properties {
            let v = match prop {
                ObjectPropertyKind::ObjectProperty(prop) => {
                    // not static key just preserve it
                    if let Some(name) = prop.key.name() {
                        needed_keys.contains(CompactStr::from(name.as_ref()).as_str())
                    } else {
                        true
                    }
                }
                // not static key
                ObjectPropertyKind::SpreadProperty(_) => true,
            };
            should_preserved_keys.push(v);
        }

        // we could ensure `should_preserved_keys` has the same length as `obj.properties`
        // the method copy from std doc https://doc.rust-lang.org/std/vec/struct.Vec.html#examples-26
        let mut iter = should_preserved_keys.iter();
        obj.properties.retain(|_| *iter.next().unwrap());
        expr
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DotDefineMemberExpression<'b, 'ast: 'b> {
    StaticMemberExpression(&'b StaticMemberExpression<'ast>),
    ComputedMemberExpression(&'b ComputedMemberExpression<'ast>),
}

impl<'b, 'a> DotDefineMemberExpression<'b, 'a> {
    fn name(&self) -> Option<Atom<'a>> {
        match self {
            DotDefineMemberExpression::StaticMemberExpression(expr) => {
                Some(expr.property.name.as_atom())
            }
            DotDefineMemberExpression::ComputedMemberExpression(expr) => {
                static_property_name_of_computed_expr(expr).copied()
            }
        }
    }

    fn object(&self) -> &'b Expression<'a> {
        match self {
            DotDefineMemberExpression::StaticMemberExpression(expr) => &expr.object,
            DotDefineMemberExpression::ComputedMemberExpression(expr) => &expr.object,
        }
    }
}

fn static_property_name_of_computed_expr<'b, 'a: 'b>(
    expr: &'b ComputedMemberExpression<'a>,
) -> Option<&'b Atom<'a>> {
    match &expr.expression {
        Expression::StringLiteral(lit) => Some(&lit.value),
        Expression::TemplateLiteral(lit) if lit.expressions.is_empty() && lit.quasis.len() == 1 => {
            Some(&lit.quasis[0].value.raw)
        }
        _ => None,
    }
}

const fn should_replace_this_expr(scope_flags: ScopeFlags) -> bool {
    !scope_flags.contains(ScopeFlags::Function) || scope_flags.contains(ScopeFlags::Arrow)
}

fn assignment_target_from_expr(expr: Expression) -> Option<AssignmentTarget> {
    match expr {
        Expression::ComputedMemberExpression(expr) => {
            Some(AssignmentTarget::ComputedMemberExpression(expr))
        }
        Expression::StaticMemberExpression(expr) => {
            Some(AssignmentTarget::StaticMemberExpression(expr))
        }
        Expression::Identifier(ident) => Some(AssignmentTarget::AssignmentTargetIdentifier(ident)),
        _ => None,
    }
}

/// Clear spans on replaced expressions for sourcemap accuracy.
struct ClearSpans;

impl VisitMut<'_> for ClearSpans {
    fn visit_span(&mut self, span: &mut Span) {
        *span = SPAN;
    }
}
