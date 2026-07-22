//! Replace global identifiers, member-expression chains, and `typeof` expressions with configured
//! constant values — the engine behind bundler `define` options (esbuild / Vite style).
//!
//! ```text
//! // define: `process.env.NODE_ENV` -> `"production"`, `__DEV__` -> `false`
//! if (process.env.NODE_ENV === "production" && !__DEV__) foo();
//! // becomes:
//! if ("production" === "production" && !false) foo();
//! ```
//!
//! The minifier then folds the now-constant condition, which is the point of `define`.
//!
//! # How matching is dispatched
//!
//! [`ReplaceGlobalDefines`] walks *every* expression in the AST, so checking each node against
//! every configured define would be `O(nodes * defines)`. We avoid that by keying defines on their
//! **trailing name**: a member expression `a.b.c` can only match a define whose last part is `c`,
//! and the visited node already carries that name (`member.property.name`). So the config buckets
//! defines into [`FxHashMap`]s keyed by that trailing segment, and each node becomes a single map
//! lookup; the full chain-walk verification only runs on the rare bucket hit. This makes the pass
//! effectively `O(nodes)`.
//!
//! Defines whose trailing name is not fixed (`import.meta.env.*`) or that apply only to one
//! operator (`typeof x`) cannot be keyed, so they fall back to small linearly-scanned lists — each
//! gated by a cheap check so ordinary code never touches them. The three storage tiers are laid out
//! on the config struct.

use std::sync::Arc;

use rustc_hash::{FxHashMap, FxHashSet};

use oxc_allocator::{Address, Allocator, ArenaBox, GetAddress, ReplaceWith, UnstableAddress};
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
    /// Identifier defines keyed by name, so each identifier reference is a single lookup instead
    /// of a scan over every define.
    identifier_defines: FxHashMap<CompactStr, CompactStr>,
    /// Whether user want to replace `ThisExpression`, avoid linear scan for each `ThisExpression`
    has_this_expr_define: bool,
}
/// Parsed, match-ready form of a [`ReplaceGlobalDefinesConfig`].
///
/// Defines are grouped into three tiers by how they are matched against AST nodes:
///
/// 1. **Keyed maps** (`identifier`, `dot`, `meta_property`) — bucketed by the trailing property
///    name, so matching a node is a single `O(1)` lookup. This is the common case and the reason
///    the whole pass is `O(nodes)` rather than `O(nodes * defines)`.
/// 2. **Linearly scanned lists** (`meta_property_wildcard`, `typeof_defines`) — defines whose
///    trailing name is variable (`import.meta.env.*`) or that only apply to a `typeof` operator, so
///    they cannot be keyed. Each scan is gated by a cheap check (is this an `import.meta` member? is
///    this a `typeof`?) so ordinary code never pays for them.
/// 3. **Special case** (`import_meta`) — the bare `import.meta` replacement.
#[derive(Debug)]
struct ReplaceGlobalDefinesConfigImpl {
    // Tier 1 — keyed by trailing name, O(1) dispatch per node.
    /// Identifier defines (e.g. `__DEV__`) keyed by name, plus whether `this` is replaced.
    identifier: IdentifierDefine,
    /// Dot defines keyed by their last part (the outermost property name accessed), e.g.
    /// `process.env.NODE_ENV` is bucketed under `NODE_ENV`. A member expression can only match a
    /// define that shares its outermost property name, so keying on it avoids scanning unrelated
    /// defines for every member expression in the file.
    dot: FxHashMap<CompactStr, Vec<DotDefine>>,
    /// Non-wildcard meta property defines, keyed by their last part (e.g. `import.meta.env.MODE`
    /// under `MODE`, `import.meta.env` under `env`). Same rationale as [`Self::dot`].
    meta_property: FxHashMap<CompactStr, Vec<MetaPropertyDefine>>,

    // Tier 2 — scanned linearly, but gated so ordinary nodes skip the scan.
    /// Wildcard meta property defines (`import.meta.env.*`, `import.meta.*`). These match a
    /// variable trailing property name and so cannot be keyed by it; they are scanned linearly,
    /// but only for `import.meta`-rooted expressions and there are normally very few of them.
    meta_property_wildcard: Vec<MetaPropertyDefine>,
    /// `typeof` defines (e.g. `typeof window` -> `"object"`). Scanned only when a `typeof` unary
    /// expression is visited, so non-`typeof` code never touches them.
    typeof_defines: Vec<TypeofDefine>,

    // Tier 3 — special case.
    /// Replacement for a bare `import.meta`. A dedicated field so we don't scan `meta_property` for
    /// `import.meta` on every meta property. `Some` -> replace, `None` -> leave as is.
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

/// What a config define key parses into. Produced by [`ReplaceGlobalDefinesConfig::check_key`] and
/// consumed by [`ReplaceGlobalDefinesConfig::new`] to route each define into the right storage tier.
enum IdentifierType {
    Identifier,
    // `key` is the last part (the bucket key); `parts` is the full member path.
    DotDefines { key: CompactStr, parts: Vec<CompactStr> },
    Typeof { parts: Vec<CompactStr> },
    // import.meta.a; `key` is the last part (used to bucket non-wildcard defines).
    ImportMetaWithParts { key: CompactStr, parts: Vec<CompactStr>, postfix_wildcard: bool },
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
        let mut identifier_defines: FxHashMap<CompactStr, CompactStr> = FxHashMap::default();
        let mut dot_defines: FxHashMap<CompactStr, Vec<DotDefine>> = FxHashMap::default();
        let mut typeof_defines = vec![];
        let mut meta_property: FxHashMap<CompactStr, Vec<MetaPropertyDefine>> =
            FxHashMap::default();
        let mut meta_property_wildcard: Vec<MetaPropertyDefine> = vec![];
        let mut import_meta = None;
        let mut has_this_expr_define = false;
        for (key, value) in defines {
            let key = key.as_ref();

            let value = value.as_ref();
            Self::check_value(&allocator, value)?;

            match Self::check_key(key)? {
                IdentifierType::Identifier => {
                    has_this_expr_define |= key == "this";
                    // Keep the first definition for a duplicate key, matching the previous
                    // first-match-wins linear scan.
                    identifier_defines
                        .entry(CompactStr::new(key))
                        .or_insert_with(|| CompactStr::new(value));
                }
                IdentifierType::DotDefines { key, parts } => {
                    dot_defines
                        .entry(key)
                        .or_default()
                        .push(DotDefine::new(parts, CompactStr::new(value)));
                }
                IdentifierType::Typeof { parts } => {
                    typeof_defines.push(TypeofDefine { parts, value: CompactStr::new(value) });
                }
                IdentifierType::ImportMetaWithParts { key, parts, postfix_wildcard } => {
                    let define =
                        MetaPropertyDefine::new(parts, CompactStr::new(value), postfix_wildcard);
                    if postfix_wildcard {
                        meta_property_wildcard.push(define);
                    } else {
                        meta_property.entry(key).or_default().push(define);
                    }
                }
                IdentifierType::ImportMeta(postfix_wildcard) => {
                    if postfix_wildcard {
                        meta_property_wildcard.push(MetaPropertyDefine::new(
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
        // No sort needed: at match time the non-wildcard `meta_property` map is always consulted
        // before the `meta_property_wildcard` list, which preserves "specific wins over wildcard".
        // See test case `dot_with_postfix_mixed`.
        Ok(Self(Arc::new(ReplaceGlobalDefinesConfigImpl {
            identifier: IdentifierDefine { identifier_defines, has_this_expr_define },
            dot: dot_defines,
            meta_property,
            meta_property_wildcard,
            typeof_defines,
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
        // The last non-wildcard part is the bucket key (the outermost property name accessed).
        // It always exists here: `normalized_parts_len >= 2` after the `parts.len() == 1` return.
        let key = CompactStr::new(parts[normalized_parts_len - 1]);
        if is_import_meta {
            match normalized_parts_len {
                2 => Ok(IdentifierType::ImportMeta(normalized_parts_len != parts.len())),
                _ => Ok(IdentifierType::ImportMetaWithParts {
                    key,
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
            Ok(IdentifierType::DotDefines { key, parts: compact_parts })
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
    /// Depth of class field initializers or static blocks where `this` is the instance/class.
    class_this_depth: u32,
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
            Self::unwrap_chain_expression_if_no_optional(expr);
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

    fn visit_property_definition(&mut self, property: &mut PropertyDefinition<'a>) {
        self.visit_decorators(&mut property.decorators);
        self.visit_property_key(&mut property.key);
        if let Some(type_annotation) = &mut property.type_annotation {
            self.visit_ts_type_annotation(type_annotation);
        }
        if let Some(value) = &mut property.value {
            self.class_this_depth += 1;
            self.visit_expression(value);
            self.class_this_depth -= 1;
        }
    }

    fn visit_static_block(&mut self, block: &mut StaticBlock<'a>) {
        self.class_this_depth += 1;
        walk_mut::walk_static_block(self, block);
        self.class_this_depth -= 1;
    }

    fn visit_accessor_property(&mut self, property: &mut AccessorProperty<'a>) {
        self.visit_decorators(&mut property.decorators);
        self.visit_property_key(&mut property.key);
        if let Some(type_annotation) = &mut property.type_annotation {
            self.visit_ts_type_annotation(type_annotation);
        }
        if let Some(value) = &mut property.value {
            self.class_this_depth += 1;
            self.visit_expression(value);
            self.class_this_depth -= 1;
        }
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
            class_this_depth: 0,
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

    // ===== Matchers =====
    // One entry point per define kind. `visit_expression` tries them in the order
    // typeof -> identifier -> dot/meta; the first one to match replaces the node.

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
                if let Some(value) = self.config.0.identifier.identifier_defines.get("this") {
                    let value = value.clone();
                    let value = self.parse_value(&value);
                    *expr = value;
                    return true;
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
            Expression::ImportMeta(_) => {
                return define.parts.len() == 2
                    && define.parts[0].as_str() == "import"
                    && define.parts[1].as_str() == "meta";
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
        let value = self.config.0.identifier.identifier_defines.get(ident.name.as_str())?.clone();
        let value = self.parse_value(&value);
        Some(value)
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
            Expression::ImportMeta(_) => {
                if let Some(replacement) = self.config.0.import_meta.clone() {
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
        // Only a static computed key (e.g. `a["b"]`) can match a define; a dynamic key never does,
        // and its outermost property name is what selects the candidate defines.
        let leaf = static_property_name_of_computed_expr(member)?;
        // TODO: meta_property_define
        let value = self.find_dot_define_value(
            leaf.as_str(),
            DotDefineMemberExpression::ComputedMemberExpression(member),
        )?;
        Some(self.parse_value(&value))
    }

    fn replace_dot_static_member_expr(
        &mut self,
        member: &StaticMemberExpression<'a>,
    ) -> Option<Expression<'a>> {
        let value = self.replace_dot_static_member_expr_impl(member)?;
        Some(self.destructing_dot_define_optimizer(value))
    }

    /// Like `replace_dot_static_member_expr` but without the destructuring optimization.
    /// Used for assignment targets where we don't have a parent destructuring pattern.
    fn replace_dot_static_member_expr_no_optimize(
        &mut self,
        member: &StaticMemberExpression<'a>,
    ) -> Option<Expression<'a>> {
        self.replace_dot_static_member_expr_impl(member)
    }

    /// Shared lookup for a static member expression: a `dot` define, then a `meta_property`
    /// define. Returns the parsed replacement expression, or `None` if nothing matches.
    fn replace_dot_static_member_expr_impl(
        &mut self,
        member: &StaticMemberExpression<'a>,
    ) -> Option<Expression<'a>> {
        let leaf = member.property.name.as_str();
        let value = self
            .find_dot_define_value(leaf, DotDefineMemberExpression::StaticMemberExpression(member))
            .or_else(|| self.find_meta_property_value(leaf, member))?;
        Some(self.parse_value(&value))
    }

    /// Find the value of a `dot` define matching `member`, whose outermost property name is `leaf`.
    /// Only defines bucketed under `leaf` can match, so this is a single map lookup instead of a
    /// scan over every define.
    fn find_dot_define_value(
        &self,
        leaf: &str,
        member: DotDefineMemberExpression<'_, 'a>,
    ) -> Option<CompactStr> {
        let bucket = self.config.0.dot.get(leaf)?;
        let scoping = self.scoping();
        let scope_flags = self.current_scope_flags();
        bucket
            .iter()
            .find(|dot_define| Self::is_dot_define(scoping, scope_flags, dot_define, member))
            .map(|dot_define| dot_define.value.clone())
    }

    /// Find the value of a meta property define matching the static member `member`, whose
    /// outermost property name is `leaf`. Non-wildcard defines are bucketed under `leaf`; wildcard
    /// defines match a variable trailing name, so they are scanned linearly but only for
    /// `import.meta`-rooted expressions.
    fn find_meta_property_value(
        &self,
        leaf: &str,
        member: &StaticMemberExpression<'a>,
    ) -> Option<CompactStr> {
        if let Some(bucket) = self.config.0.meta_property.get(leaf)
            && let Some(define) =
                bucket.iter().find(|define| Self::is_meta_property_define(define, member))
        {
            return Some(define.value.clone());
        }
        if !self.config.0.meta_property_wildcard.is_empty() && Self::is_import_meta_member(member) {
            return self
                .config
                .0
                .meta_property_wildcard
                .iter()
                .find(|define| Self::is_meta_property_define(define, member))
                .map(|define| define.value.clone());
        }
        None
    }

    /// Whether `member`'s object chain roots at `import.meta`.
    ///
    /// This is a cheap fast-reject, not a correctness check ([`Self::is_meta_property_define`]
    /// already rejects non-`import.meta` roots): a meta property define can only ever match a
    /// member expression rooted at `import.meta`, so walking the chain once here lets the caller
    /// skip the whole wildcard scan for ordinary member expressions such as `console.log` or
    /// `a.b.c`.
    fn is_import_meta_member(member: &StaticMemberExpression<'a>) -> bool {
        matches!(member.get_first_object(), Expression::ImportMeta(_))
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
                Expression::ImportMeta(_) => return true,
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
                    Expression::ImportMeta(_) => {
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

    /// Compute the current scope flags based on function and class `this` depth tracking.
    fn current_scope_flags(&self) -> ScopeFlags {
        if self.non_arrow_function_depth > 0 {
            ScopeFlags::Function
        } else if self.class_this_depth > 0 {
            ScopeFlags::ClassStaticBlock
        } else {
            ScopeFlags::Top
        }
    }

    /// If `expr` is a `ChainExpression` whose chain no longer contains any
    /// `optional: true` markers (because a define replacement removed them),
    /// unwrap it to a plain expression.
    fn unwrap_chain_expression_if_no_optional(expr: &mut Expression<'a>) {
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
        expr.replace_with(|chain_expr| {
            let Expression::ChainExpression(chain) = chain_expr else { unreachable!() };
            Expression::from(chain.unbox().expression)
        });
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
                    Expression::ImportMeta(_) => {
                        // Handle import.meta
                        // When we encounter import.meta, verify that the remaining
                        // parts match ["import", "meta"]
                        // At this point, i is the current position we're checking
                        // We need the next two parts (going backwards) to be "meta" then "import"
                        // i.e., parts[i-1] == "meta" and parts[i-2] == "import"
                        if i >= 2
                            && parts[i - 1].as_str() == "meta"
                            && parts[i - 2].as_str() == "import"
                        {
                            // Successfully matched import.meta at the expected position.
                            return i == 2;
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
    !scope_flags.contains(ScopeFlags::ClassStaticBlock)
        && (!scope_flags.contains(ScopeFlags::Function) || scope_flags.contains(ScopeFlags::Arrow))
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
