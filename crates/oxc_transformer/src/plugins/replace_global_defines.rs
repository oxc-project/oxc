use std::{cmp::Ordering, sync::Arc};

use lazy_static::lazy_static;
use oxc_allocator::{Address, Allocator, GetAddress};
use oxc_ast::{ast::*, VisitMut};
use oxc_diagnostics::OxcDiagnostic;
use oxc_parser::Parser;
use oxc_semantic::{IsGlobalReference, ScopeFlags, ScopeTree, SymbolTable};
use oxc_span::{CompactStr, SourceType, SPAN};
use oxc_syntax::identifier::is_identifier_name;
use oxc_traverse::{traverse_mut, Ancestor, Traverse, TraverseCtx};
use rustc_hash::FxHashSet;

/// Configuration for [ReplaceGlobalDefines].
///
/// Due to the usage of an arena allocator, the constructor will parse once for grammatical errors,
/// and does not save the constructed expression.
///
/// The data is stored in an `Arc` so this can be shared across threads.
#[derive(Debug, Clone)]
pub struct ReplaceGlobalDefinesConfig(Arc<ReplaceGlobalDefinesConfigImpl>);

lazy_static! {
    static ref THIS_ATOM: Atom<'static> = Atom::from("this");
}

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
    pub symbols: SymbolTable,
    pub scopes: ScopeTree,
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
    /// Since `Traverse` did not provide a way to skipping visiting sub tree of the AstNode,
    /// Use `Option<Address>` to lock the current node when it is `Some`.
    /// during visiting sub tree, the `Lock` will always be `Some`, and we can early return, this
    /// could acheieve same effect as skipping visiting sub tree.
    /// When `exit` the node, reset the `Lock` to `None` to make sure not affect other
    /// transformation.
    ast_node_lock: Option<Address>,
}

impl<'a> Traverse<'a> for ReplaceGlobalDefines<'a> {
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.ast_node_lock.is_some() {
            return;
        }
        let is_replaced =
            self.replace_identifier_defines(expr, ctx) || self.replace_dot_defines(expr, ctx);
        if is_replaced {
            self.ast_node_lock = Some(expr.address());
        }
    }

    fn exit_expression(&mut self, node: &mut Expression<'a>, _ctx: &mut TraverseCtx<'a>) {
        if self.ast_node_lock == Some(node.address()) {
            self.ast_node_lock = None;
        }
    }

    fn enter_assignment_expression(
        &mut self,
        node: &mut AssignmentExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.ast_node_lock.is_some() {
            return;
        }
        if self.replace_define_with_assignment_expr(node, ctx) {
            // `AssignmentExpression` is stored in a `Box`, so we can use `from_ptr` to get
            // the stable address
            self.ast_node_lock = Some(Address::from_ptr(node));
        }
    }

    fn exit_assignment_expression(
        &mut self,
        node: &mut AssignmentExpression<'a>,
        _: &mut TraverseCtx<'a>,
    ) {
        if self.ast_node_lock == Some(Address::from_ptr(node)) {
            self.ast_node_lock = None;
        }
    }
}

impl<'a> ReplaceGlobalDefines<'a> {
    pub fn new(allocator: &'a Allocator, config: ReplaceGlobalDefinesConfig) -> Self {
        Self { allocator, config, ast_node_lock: None }
    }

    pub fn build(
        &mut self,
        symbols: SymbolTable,
        scopes: ScopeTree,
        program: &mut Program<'a>,
    ) -> ReplaceGlobalDefinesReturn {
        let (symbols, scopes) = traverse_mut(self, self.allocator, program, symbols, scopes);
        ReplaceGlobalDefinesReturn { symbols, scopes }
    }

    // Construct a new expression because we don't have ast clone right now.
    fn parse_value(&self, source_text: &str) -> Expression<'a> {
        // Allocate the string lazily because replacement happens rarely.
        let source_text = self.allocator.alloc_str(source_text);
        // Unwrapping here, it should already be checked by [ReplaceGlobalDefinesConfig::new].
        let mut expr = Parser::new(self.allocator, source_text, SourceType::default())
            .parse_expression()
            .unwrap();

        RemoveSpans.visit_expression(&mut expr);

        expr
    }

    fn replace_identifier_defines(
        &self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> bool {
        match expr {
            Expression::Identifier(ident) => {
                if let Some(new_expr) = self.replace_identifier_define_impl(ident, ctx) {
                    *expr = new_expr;
                    return true;
                }
            }
            Expression::ThisExpression(_)
                if self.config.0.identifier.has_this_expr_define
                    && should_replace_this_expr(ctx.current_scope_flags()) =>
            {
                for (key, value) in &self.config.0.identifier.identifier_defines {
                    if key.as_str() == "this" {
                        let value = self.parse_value(value);
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
        ident: &mut oxc_allocator::Box<'_, IdentifierReference<'_>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        if !ident.is_global_reference(ctx.symbols()) {
            return None;
        }
        for (key, value) in &self.config.0.identifier.identifier_defines {
            if ident.name.as_str() == key {
                let value = self.parse_value(value);

                return Some(value);
            }
        }
        None
    }

    fn replace_define_with_assignment_expr(
        &mut self,
        node: &mut AssignmentExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> bool {
        let new_left = node
            .left
            .as_simple_assignment_target_mut()
            .and_then(|item| match item {
                SimpleAssignmentTarget::ComputedMemberExpression(computed_member_expr) => {
                    self.replace_dot_computed_member_expr(ctx, computed_member_expr)
                }
                SimpleAssignmentTarget::StaticMemberExpression(member) => {
                    self.replace_dot_static_member_expr(ctx, member)
                }
                SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
                    self.replace_identifier_define_impl(ident, ctx)
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

    fn replace_dot_defines(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> bool {
        match expr {
            Expression::ChainExpression(chain) => {
                let Some(new_expr) =
                    chain.expression.as_member_expression_mut().and_then(|item| match item {
                        MemberExpression::ComputedMemberExpression(computed_member_expr) => {
                            self.replace_dot_computed_member_expr(ctx, computed_member_expr)
                        }
                        MemberExpression::StaticMemberExpression(member) => {
                            self.replace_dot_static_member_expr(ctx, member)
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
                if let Some(new_expr) = self.replace_dot_static_member_expr(ctx, member) {
                    *expr = new_expr;
                    return true;
                }
            }
            Expression::ComputedMemberExpression(member) => {
                if let Some(new_expr) = self.replace_dot_computed_member_expr(ctx, member) {
                    *expr = new_expr;
                    return true;
                }
            }
            Expression::MetaProperty(meta_property) => {
                if let Some(replacement) = &self.config.0.import_meta {
                    if meta_property.meta.name == "import" && meta_property.property.name == "meta"
                    {
                        let value = self.parse_value(replacement);
                        *expr = value;
                        return true;
                    }
                }
            }
            _ => {}
        }
        false
    }

    fn replace_dot_computed_member_expr(
        &mut self,
        ctx: &mut TraverseCtx<'a>,
        member: &mut ComputedMemberExpression<'a>,
    ) -> Option<Expression<'a>> {
        for dot_define in &self.config.0.dot {
            if Self::is_dot_define(
                ctx,
                dot_define,
                DotDefineMemberExpression::ComputedMemberExpression(member),
            ) {
                let value = self.parse_value(&dot_define.value);
                return Some(value);
            }
        }
        // TODO: meta_property_define
        None
    }

    fn replace_dot_static_member_expr(
        &mut self,
        ctx: &mut TraverseCtx<'a>,
        member: &mut StaticMemberExpression<'a>,
    ) -> Option<Expression<'a>> {
        for dot_define in &self.config.0.dot {
            if Self::is_dot_define(
                ctx,
                dot_define,
                DotDefineMemberExpression::StaticMemberExpression(member),
            ) {
                let value = self.parse_value(&dot_define.value);
                return Some(destructing_dot_define_optimizer(value, ctx));
            }
        }
        for meta_property_define in &self.config.0.meta_property {
            if Self::is_meta_property_define(meta_property_define, member) {
                let value = self.parse_value(&meta_property_define.value);
                return Some(destructing_dot_define_optimizer(value, ctx));
            }
        }
        None
    }

    pub fn is_meta_property_define(
        meta_define: &MetaPropertyDefine,
        member: &StaticMemberExpression<'a>,
    ) -> bool {
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
        let mut cur_part_name = &member.property.name;
        let mut is_full_match = true;
        let mut i = meta_define.parts.len() - 1;
        let mut has_matched_part = false;
        loop {
            let part = &meta_define.parts[i];
            let matched = cur_part_name.as_str() == part;
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

                if !meta_define.postfix_wildcard || has_matched_part {
                    return false;
                }
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
                        return true;
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

    pub fn is_dot_define<'b>(
        ctx: &mut TraverseCtx<'a>,
        dot_define: &DotDefine,
        member: DotDefineMemberExpression<'b, 'a>,
    ) -> bool {
        debug_assert!(dot_define.parts.len() > 1);
        let should_replace_this_expr = should_replace_this_expr(ctx.current_scope_flags());
        let Some(mut cur_part_name) = member.name() else {
            return false;
        };
        let mut current_part_member_expression = Some(member);

        for (i, part) in dot_define.parts.iter().enumerate().rev() {
            if cur_part_name.as_str() != part {
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
                            cur_part_name = name;
                            DotDefineMemberExpression::ComputedMemberExpression(computed_member)
                        })
                    }
                    Expression::Identifier(ident) => {
                        if !ident.is_global_reference(ctx.symbols()) {
                            return false;
                        }
                        cur_part_name = &ident.name;
                        None
                    }
                    Expression::ThisExpression(_) if should_replace_this_expr => {
                        cur_part_name = &THIS_ATOM;
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
}

#[derive(Debug, Clone, Copy)]
pub enum DotDefineMemberExpression<'b, 'ast: 'b> {
    StaticMemberExpression(&'b StaticMemberExpression<'ast>),
    ComputedMemberExpression(&'b ComputedMemberExpression<'ast>),
}

impl<'b, 'a> DotDefineMemberExpression<'b, 'a> {
    fn name(&self) -> Option<&'b Atom<'a>> {
        match self {
            DotDefineMemberExpression::StaticMemberExpression(expr) => Some(&expr.property.name),
            DotDefineMemberExpression::ComputedMemberExpression(expr) => {
                static_property_name_of_computed_expr(expr)
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

fn destructing_dot_define_optimizer<'ast>(
    mut expr: Expression<'ast>,
    ctx: &mut TraverseCtx<'ast>,
) -> Expression<'ast> {
    let Expression::ObjectExpression(obj) = &mut expr else { return expr };
    let parent = ctx.parent();
    let destruct_obj_pat = match parent {
        Ancestor::VariableDeclaratorInit(declarator) => match &declarator.id().kind {
            BindingPatternKind::ObjectPattern(pat) => pat,
            _ => return expr,
        },
        _ => {
            return expr;
        }
    };
    let mut needed_keys = FxHashSet::default();
    for prop in &destruct_obj_pat.properties {
        match prop.key.name() {
            Some(key) => {
                needed_keys.insert(key);
            }
            // if there exists a none static key, we can't optimize
            None => {
                return expr;
            }
        }
    }

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
                    needed_keys.contains(&name)
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

struct RemoveSpans;

impl VisitMut<'_> for RemoveSpans {
    fn visit_span(&mut self, span: &mut Span) {
        *span = SPAN;
    }
}
