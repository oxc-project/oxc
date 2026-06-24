use std::{cmp::Ordering, sync::Arc};

use rustc_hash::FxHashSet;

use oxc_allocator::{Address, Allocator, ArenaBox, GetAddress, TakeIn, UnstableAddress};
use oxc_ast::ast::*;
use oxc_ast_visit::{VisitMut, walk_mut};
use oxc_diagnostics::{Diagnostics, OxcDiagnostic};
use oxc_parser::Parser;
use oxc_semantic::{ReferenceFlags, ScopeFlags, Scoping};
use oxc_span::{SPAN, SourceType};
use oxc_str::CompactStr;
use oxc_syntax::identifier::is_identifier_name;
use oxc_syntax::node::NodeId;
use oxc_syntax::reference::Reference;

/// Configuration for [ReplaceGlobalDefines].
///
/// Define keys may be identifiers, property chains, `typeof` expressions over identifiers or
/// property chains, and supported `import.meta` forms.
///
/// Due to the usage of an arena allocator, the constructor will parse once for grammatical errors,
/// and does not save the constructed expression.
///
/// The data is stored in an `Arc` so this can be shared across threads.
#[derive(Debug, Clone)]
pub struct ReplaceGlobalDefinesConfig(Arc<ReplaceGlobalDefinesConfigImpl>);

static THIS_STR: Str<'static> = Str::new_const("this");

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
    typeof_defines: Vec<TypeofDefine>,
    meta_property: Vec<MetaPropertyDefine>,
    /// extra field to avoid linear scan `meta_property` to check if it has `import.meta` every
    /// time
    /// Some(replacement): import.meta -> replacement
    /// None -> no need to replace import.meta
    import_meta: Option<CompactStr>,
}

#[derive(Debug)]
struct TypeofDefine {
    parts: Vec<CompactStr>,
    value: CompactStr,
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
    Typeof { parts: Vec<CompactStr> },
    // import.meta.a
    ImportMetaWithParts { parts: Vec<CompactStr>, postfix_wildcard: bool },
    // import.meta or import.meta.*
    ImportMeta(bool),
}

impl ReplaceGlobalDefinesConfig {
    /// # Errors
    ///
    /// * key is not a supported identifier, property chain, or `typeof` expression
    /// * value has a syntax error
    pub fn new<S: AsRef<str>>(defines: &[(S, S)]) -> Result<Self, Diagnostics> {
        let allocator = Allocator::default();
        let mut identifier_defines = vec![];
        let mut dot_defines = vec![];
        let mut typeof_defines = vec![];
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
                IdentifierType::Typeof { parts } => {
                    typeof_defines.push(TypeofDefine { parts, value: CompactStr::new(value) });
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
            typeof_defines,
            meta_property: meta_properties_defines,
            import_meta,
        })))
    }

    fn check_key(key: &str) -> Result<IdentifierType, Diagnostics> {
        if let Some(argument) = key.strip_prefix("typeof ") {
            let parts: Vec<&str> = argument.split('.').collect();
            let parts = Self::check_identifier_parts(key, &parts)?;
            return Ok(IdentifierType::Typeof { parts });
        }

        let parts: Vec<&str> = key.split('.').collect();

        assert!(!parts.is_empty());

        if parts.len() == 1 {
            if !is_identifier_name(parts[0]) {
                return Err(vec![OxcDiagnostic::error(format!(
                    "The define key `{key}` is not an identifier."
                ))]
                .into());
            }
            return Ok(IdentifierType::Identifier);
        }
        let normalized_parts_len =
            if parts[parts.len() - 1] == "*" { parts.len() - 1 } else { parts.len() };
        // We can ensure now the parts.len() >= 2
        let is_import_meta = parts[0] == "import" && parts[1] == "meta";

        let compact_parts = Self::check_identifier_parts(key, &parts[0..normalized_parts_len])?;
        if is_import_meta {
            match normalized_parts_len {
                2 => Ok(IdentifierType::ImportMeta(normalized_parts_len != parts.len())),
                _ => Ok(IdentifierType::ImportMetaWithParts {
                    parts: compact_parts.into_iter().skip(2).collect(),
                    postfix_wildcard: normalized_parts_len != parts.len(),
                }),
            }
        // StaticMemberExpression with postfix wildcard
        } else if normalized_parts_len != parts.len() {
            Err(vec![OxcDiagnostic::error(
                "The postfix wildcard is only allowed for `import.meta`.".to_string(),
            )]
            .into())
        } else {
            Ok(IdentifierType::DotDefines { parts: compact_parts })
        }
    }

    fn check_identifier_parts(key: &str, parts: &[&str]) -> Result<Vec<CompactStr>, Diagnostics> {
        for part in parts {
            if !is_identifier_name(part) {
                return Err(vec![OxcDiagnostic::error(format!(
                    "The define key `{key}` contains an invalid identifier `{part}`."
                ))]
                .into());
            }
        }

        Ok(parts.iter().map(|part| CompactStr::new(part)).collect())
    }

    fn check_value(allocator: &Allocator, source_text: &str) -> Result<(), Diagnostics> {
        Parser::new(allocator, source_text, SourceType::default()).parse_expression()?;
        Ok(())
    }
}

#[must_use]
pub struct ReplaceGlobalDefinesReturn {
    pub scoping: Scoping,
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
    /// Scoping data, stored during `build()`.
    scoping: Option<Scoping>,
    /// Depth of non-arrow functions we're inside of. Used to compute scope flags for `this`
    /// replacement.
    non_arrow_function_depth: u32,
    /// Destructuring keys from the parent `VariableDeclarator` when its `id` is an
    /// `ObjectPattern`. Used to optimize object expression replacements by only keeping needed
    /// keys.
    destructuring_keys: Option<FxHashSet<CompactStr>>,
}

impl<'a> VisitMut<'a> for ReplaceGlobalDefines<'a> {
    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        if self.ast_node_lock.is_some() {
            return;
        }
        let is_replaced = (!self.config.0.typeof_defines.is_empty()
            && self.replace_typeof_defines(expr))
            || self.replace_identifier_defines(expr)
            || self.replace_dot_defines(expr);
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
        // A define replacement inside a `ChainExpression` may remove the node that
        // carried `optional: true` (e.g. `process?.env[0]` with define `process.env -> {}`),
        // leaving an invalid `ChainExpression` with no optional elements.
        // Unwrap it to a plain expression to produce a valid AST.
        if matches!(expr, Expression::ChainExpression(_)) {
            self.unwrap_chain_expression_if_no_optional(expr);
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
        self.non_arrow_function_depth += 1;
        walk_mut::walk_function(self, func, flags);
        self.non_arrow_function_depth -= 1;
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
}

impl<'a> ReplaceGlobalDefines<'a> {
    pub fn new(allocator: &'a Allocator, config: ReplaceGlobalDefinesConfig) -> Self {
        Self {
            allocator,
            config,
            ast_node_lock: None,
            changed: false,
            scoping: None,
            non_arrow_function_depth: 0,
            destructuring_keys: None,
        }
    }

    fn mark_as_changed(&mut self) {
        self.changed = true;
    }

    /// # Panics
    ///
    /// Panics if scoping is not set (i.e. called outside of `build`).
    fn scoping(&self) -> &Scoping {
        self.scoping.as_ref().unwrap()
    }

    #[expect(clippy::missing_panics_doc)]
    pub fn build(
        &mut self,
        scoping: Scoping,
        program: &mut Program<'a>,
    ) -> ReplaceGlobalDefinesReturn {
        self.scoping = Some(scoping);
        self.visit_program(program);
        let scoping = self.scoping.take().unwrap();
        ReplaceGlobalDefinesReturn { scoping, changed: self.changed }
    }

    // Construct a new expression because we don't have ast clone right now.
    fn parse_value(&mut self, source_text: &str) -> Expression<'a> {
        // Allocate the string lazily because replacement happens rarely.
        let source_text = self.allocator.alloc_str(source_text);
        // Unwrapping here, it should already be checked by [ReplaceGlobalDefinesConfig::new].
        let mut expr = Parser::new(self.allocator, source_text, SourceType::default())
            .parse_expression()
            .unwrap();

        let scoping = self.scoping.as_mut().unwrap();
        UpdateReplacedExpression { scoping }.visit_expression(&mut expr);

        expr
    }

    fn replace_identifier_defines(&mut self, expr: &mut Expression<'a>) -> bool {
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

    fn replace_typeof_defines(&mut self, expr: &mut Expression<'a>) -> bool {
        let Expression::UnaryExpression(unary) = expr else {
            return false;
        };
        if unary.operator != UnaryOperator::Typeof {
            return false;
        }

        let scoping = self.scoping();
        let scope_flags = self.current_scope_flags();
        let value = self
            .config
            .0
            .typeof_defines
            .iter()
            .find(|define| Self::is_typeof_define(scoping, scope_flags, define, &unary.argument))
            .map(|define| define.value.clone());
        if let Some(value) = value {
            *expr = self.parse_value(&value);
            return true;
        }
        false
    }

    fn is_typeof_define(
        scoping: &Scoping,
        scope_flags: ScopeFlags,
        define: &TypeofDefine,
        argument: &Expression<'a>,
    ) -> bool {
        let argument = argument.without_parentheses();
        if define.parts.len() == 1 {
            return match argument {
                Expression::Identifier(ident) => {
                    ident.name == define.parts[0].as_str()
                        && Self::is_global_or_ambient_reference(scoping, ident)
                }
                Expression::ThisExpression(_) => {
                    define.parts[0].as_str() == "this" && should_replace_this_expr(scope_flags)
                }
                _ => false,
            };
        }

        let member = match argument {
            Expression::StaticMemberExpression(member) => {
                DotDefineMemberExpression::StaticMemberExpression(member)
            }
            Expression::ComputedMemberExpression(member) => {
                DotDefineMemberExpression::ComputedMemberExpression(member)
            }
            Expression::ChainExpression(chain) => {
                let Some(member) = chain.expression.as_member_expression() else {
                    return false;
                };
                match member {
                    MemberExpression::StaticMemberExpression(member) => {
                        DotDefineMemberExpression::StaticMemberExpression(member)
                    }
                    MemberExpression::ComputedMemberExpression(member) => {
                        DotDefineMemberExpression::ComputedMemberExpression(member)
                    }
                    MemberExpression::PrivateFieldExpression(_) => return false,
                }
            }
            Expression::MetaProperty(meta) => {
                return define.parts.len() == 2
                    && define.parts[0].as_str() == "import"
                    && define.parts[1].as_str() == "meta"
                    && meta.meta.name == "import"
                    && meta.property.name == "meta";
            }
            _ => return false,
        };
        Self::is_dot_define_parts(scoping, scope_flags, &define.parts, member)
    }

    fn replace_identifier_define_impl(
        &mut self,
        ident: &ArenaBox<'_, IdentifierReference<'_>>,
    ) -> Option<Expression<'a>> {
        if !Self::is_global_or_ambient_reference(self.scoping(), ident) {
            return None;
        }
        for (key, value) in &self.config.0.identifier.identifier_defines {
            if ident.name.as_str() == key {
                let value = value.clone();
                let value = self.parse_value(&value);
                return Some(value);
            }
        }
        None
    }

    fn replace_define_with_assignment_expr(&mut self, node: &mut AssignmentExpression<'a>) -> bool {
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

    fn replace_dot_defines(&mut self, expr: &mut Expression<'a>) -> bool {
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
        &mut self,
        member: &ComputedMemberExpression<'a>,
    ) -> Option<Expression<'a>> {
        let scoping = self.scoping();
        let scope_flags = self.current_scope_flags();
        let value = self
            .config
            .0
            .dot
            .iter()
            .find(|dot_define| {
                Self::is_dot_define(
                    scoping,
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
        &mut self,
        member: &StaticMemberExpression<'a>,
    ) -> Option<Expression<'a>> {
        let scoping = self.scoping();
        let scope_flags = self.current_scope_flags();
        let value = self
            .config
            .0
            .dot
            .iter()
            .find(|dot_define| {
                Self::is_dot_define(
                    scoping,
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
        &mut self,
        member: &StaticMemberExpression<'a>,
    ) -> Option<Expression<'a>> {
        let scoping = self.scoping();
        let scope_flags = self.current_scope_flags();
        let value = self
            .config
            .0
            .dot
            .iter()
            .find(|dot_define| {
                Self::is_dot_define(
                    scoping,
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

    /// If `expr` is a `ChainExpression` whose chain no longer contains any
    /// `optional: true` markers (because a define replacement removed them),
    /// unwrap it to a plain expression.
    fn unwrap_chain_expression_if_no_optional(&self, expr: &mut Expression<'a>) {
        let Expression::ChainExpression(chain) = &*expr else { return };

        // Check the chain element's optional flag and get the first object/callee to walk.
        let (optional, mut current) = match &chain.expression {
            ChainElement::CallExpression(c) => (c.optional, Some(&c.callee)),
            ChainElement::TSNonNullExpression(ts) => (false, Some(&ts.expression)),
            _ => match chain.expression.as_member_expression() {
                Some(m) => (m.optional(), Some(m.object())),
                None => return,
            },
        };
        if optional {
            return;
        }

        // Walk down the object/callee chain. If any node has `optional: true`, keep the chain.
        while let Some(e) = current {
            match e {
                Expression::StaticMemberExpression(m) => {
                    if m.optional {
                        return;
                    }
                    current = Some(&m.object);
                }
                Expression::ComputedMemberExpression(m) => {
                    if m.optional {
                        return;
                    }
                    current = Some(&m.object);
                }
                Expression::PrivateFieldExpression(m) => {
                    if m.optional {
                        return;
                    }
                    current = Some(&m.object);
                }
                Expression::CallExpression(c) => {
                    if c.optional {
                        return;
                    }
                    current = Some(&c.callee);
                }
                _ => break,
            }
        }

        // No optional markers remain — unwrap the chain to a plain expression.
        let chain_expr = expr.take_in(&self.allocator);
        let Expression::ChainExpression(chain) = chain_expr else { unreachable!() };
        *expr = Expression::from(chain.unbox().expression);
    }

    pub fn is_dot_define<'b>(
        scoping: &Scoping,
        scope_flags: ScopeFlags,
        dot_define: &DotDefine,
        member: DotDefineMemberExpression<'b, 'a>,
    ) -> bool {
        Self::is_dot_define_parts(scoping, scope_flags, &dot_define.parts, member)
    }

    fn is_dot_define_parts<'b>(
        scoping: &Scoping,
        scope_flags: ScopeFlags,
        parts: &[CompactStr],
        member: DotDefineMemberExpression<'b, 'a>,
    ) -> bool {
        debug_assert!(parts.len() > 1);
        let should_replace_this_expr = should_replace_this_expr(scope_flags);
        let Some(cur_part_name) = member.name() else {
            return false;
        };
        let mut cur_part_name: &str = cur_part_name.as_str();
        let mut current_part_member_expression = Some(member);

        for (i, part) in parts.iter().enumerate().rev() {
            if cur_part_name != part {
                return false;
            }
            if i == 0 {
                break;
            }

            current_part_member_expression = if let Some(member) = current_part_member_expression {
                match member.object().without_parentheses() {
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
                        if !Self::is_global_or_ambient_reference(scoping, ident) {
                            return false;
                        }
                        cur_part_name = &ident.name;
                        None
                    }
                    Expression::ThisExpression(_) if should_replace_this_expr => {
                        cur_part_name = THIS_STR.as_str();
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
                                && parts[i - 1].as_str() == "meta"
                                && parts[i - 2].as_str() == "import"
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

    /// Return whether an identifier reference should be treated like a global define root.
    ///
    /// Unresolved references are globals, so `foo.bar` can be replaced by a `foo.bar` define.
    /// Ambient declarations are also replaceable because they are TypeScript-only and do not
    /// create runtime bindings:
    ///
    /// ```ts
    /// declare let self: ServiceWorkerGlobalScope;
    /// precacheAndRoute(self.__WB_MANIFEST); // replace `self.__WB_MANIFEST`
    /// ```
    ///
    /// Runtime bindings still shadow define roots:
    ///
    /// ```ts
    /// let self;
    /// precacheAndRoute(self.__WB_MANIFEST); // do not replace
    /// ```
    fn is_global_or_ambient_reference(scoping: &Scoping, ident: &IdentifierReference<'_>) -> bool {
        scoping
            .get_reference(ident.reference_id())
            .symbol_id()
            .is_none_or(|symbol_id| scoping.symbol_flags(symbol_id).is_ambient())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DotDefineMemberExpression<'b, 'ast: 'b> {
    StaticMemberExpression(&'b StaticMemberExpression<'ast>),
    ComputedMemberExpression(&'b ComputedMemberExpression<'ast>),
}

impl<'b, 'a> DotDefineMemberExpression<'b, 'a> {
    fn name(&self) -> Option<Str<'a>> {
        match self {
            DotDefineMemberExpression::StaticMemberExpression(expr) => {
                Some(expr.property.name.as_arena_str())
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
) -> Option<&'b Str<'a>> {
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

/// Update the replaced expression:
/// * change spans to empty spans for sourcemap
/// * assign reference id in root scope
struct UpdateReplacedExpression<'a> {
    scoping: &'a mut Scoping,
}

impl VisitMut<'_> for UpdateReplacedExpression<'_> {
    fn visit_identifier_reference(&mut self, ident: &mut IdentifierReference<'_>) {
        let reference =
            Reference::new(NodeId::DUMMY, self.scoping.root_scope_id(), ReferenceFlags::Read);
        let reference_id = self.scoping.create_reference(reference);
        self.scoping.add_root_unresolved_reference(ident.name, reference_id);
        ident.set_reference_id(reference_id);
        walk_mut::walk_identifier_reference(self, ident);
    }

    fn visit_span(&mut self, span: &mut Span) {
        *span = SPAN;
    }
}
