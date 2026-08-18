//! Property-name mangling.
//!
//! The flow has three steps:
//!
//! 1. Collect property names without changing the AST.
//! 2. Assign one output name to each selected property name.
//! 3. Rewrite each program once.
//!
//! Keeping these steps separate lets bundlers share one mapping across programs.

use oxc_allocator::{Allocator, TakeIn};
use oxc_ast::{ast::*, builder::AstBuilder};
use oxc_ast_visit::{Visit, VisitMut, walk, walk_mut};
use oxc_ecmascript::StringToNumber;
use oxc_mangler::base54;
use oxc_span::Span;
use oxc_str::{CompactStr, Ident, Str};
use oxc_syntax::{identifier::is_identifier_name, number::ToJsString};
use rustc_hash::{FxHashMap, FxHashSet};

/// A property-mangle cache. `Some(name)` pins the output name and `None` keeps the property.
///
/// Pinned output names are authoritative and may intentionally be shared by multiple input
/// names. Automatically generated names never share an output or collide with a cache key or
/// pinned output.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ManglePropertyCache(FxHashMap<CompactStr, Option<CompactStr>>);

impl ManglePropertyCache {
    /// Add a cache entry after validating that its target is safe in every rewritten position.
    ///
    /// # Errors
    ///
    /// Returns an error when `target` is not an `IdentifierName` or is a hard-reserved property
    /// name.
    pub fn insert(
        &mut self,
        original: CompactStr,
        target: Option<CompactStr>,
    ) -> Result<(), InvalidManglePropertyCacheTarget> {
        if let Some(target) = &target
            && !is_valid_property_mangle_cache_target(target)
        {
            return Err(InvalidManglePropertyCacheTarget { original, target: target.clone() });
        }
        self.0.insert(original, target);
        Ok(())
    }

    /// Return the pinned target or reservation for an original property name.
    pub fn get(&self, original: &str) -> Option<&Option<CompactStr>> {
        self.0.get(original)
    }

    /// Iterate over original property names and their pinned targets or reservations.
    pub fn iter(&self) -> impl Iterator<Item = (&CompactStr, &Option<CompactStr>)> {
        self.0.iter()
    }

    /// Return whether the cache contains no mappings or reservations.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn insert_generated(&mut self, original: CompactStr, target: Option<CompactStr>) {
        self.0.insert(original, target);
    }
}

impl IntoIterator for ManglePropertyCache {
    type Item = (CompactStr, Option<CompactStr>);
    type IntoIter = <FxHashMap<CompactStr, Option<CompactStr>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// An invalid property-mangle cache target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidManglePropertyCacheTarget {
    pub original: CompactStr,
    pub target: CompactStr,
}

impl std::fmt::Display for InvalidManglePropertyCacheTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid property-mangle cache target for '{}': '{}' must be an IdentifierName other than '__proto__', 'constructor', or 'prototype'",
            self.original, self.target
        )
    }
}

impl std::error::Error for InvalidManglePropertyCacheTarget {}

/// A single-application mapping from original property names to their final names.
///
/// Applying a mapping more than once is not generally idempotent: a cache target may also be
/// another cache key or source candidate. Each program must therefore be rewritten exactly once.
pub type PropertyMapping = FxHashMap<CompactStr, CompactStr>;

/// Options for opt-in property-name mangling.
#[derive(Debug, Clone)]
pub struct ManglePropertiesOptions {
    /// Property names must match this expression to be mangled.
    pub include: lazy_regex::Regex,
    /// Matching property names are excluded after `include` is evaluated.
    pub exclude: Option<lazy_regex::Regex>,
    /// Exact property names that must remain unchanged.
    pub reserved: FxHashSet<CompactStr>,
    /// Also mangle quoted property occurrences.
    pub mangle_quoted: bool,
    /// Generate readable property names instead of base54 names.
    pub debug: bool,
    /// Stable input mappings and explicit `false`-equivalent reservations.
    pub cache: ManglePropertyCache,
}

impl ManglePropertiesOptions {
    pub fn new(include: lazy_regex::Regex) -> Self {
        Self {
            include,
            exclude: None,
            reserved: FxHashSet::default(),
            mangle_quoted: false,
            debug: false,
            cache: ManglePropertyCache::default(),
        }
    }

    /// Compile an `include` pattern using Rust's regex syntax.
    ///
    /// # Errors
    ///
    /// Returns the regex parser's message when `include` is invalid.
    pub fn from_pattern(include: &str) -> Result<Self, String> {
        lazy_regex::Regex::new(include).map(Self::new).map_err(|error| error.to_string())
    }
}

/// Returns whether a user-provided cache target can be emitted in every property position.
pub fn is_valid_property_mangle_cache_target(name: &str) -> bool {
    !is_hard_reserved(name) && is_identifier_name(name)
}

fn is_hard_reserved(name: &str) -> bool {
    matches!(name, "__proto__" | "constructor" | "prototype")
}

fn numeric_key_string(value: f64) -> String {
    if value == 0.0 { "0".to_string() } else { value.to_js_string() }
}

fn is_canonical_numeric_string(name: &str) -> bool {
    let value = name.string_to_number();
    if value.is_nan() {
        return name == "NaN";
    }
    numeric_key_string(value) == name
}

fn key_annotated_spans(program: &Program<'_>) -> FxHashSet<u32> {
    program
        .comments
        .iter()
        .filter(|comment| comment.is_property_key_annotation())
        .map(|comment| comment.attached_to)
        .collect()
}

/// Data gathered by the read-only collection phase.
#[derive(Debug, Default)]
struct PropertyCollectState {
    /// Number of candidate occurrences before eligibility filtering.
    frequencies: FxHashMap<CompactStr, u32>,
    /// Spellings that remain in the output and therefore cannot be automatic output names.
    occupied: FxHashSet<CompactStr>,
}

/// Property candidates and occupied names collected from one program.
///
/// This opaque value lets parallel callers collect with a shared options reference, then merge
/// the results into one [`PropertyMangler`] without cloning caches or reserved-name sets per
/// program.
#[derive(Debug)]
pub struct PropertyMangleCollection(PropertyCollectState);

impl PropertyMangleCollection {
    /// Collect property candidates from one program without mutating its AST.
    pub fn from_program(options: &ManglePropertiesOptions, program: &Program<'_>) -> Self {
        let key_annotated = key_annotated_spans(program);
        Self(collect(options, program, &key_annotated))
    }
}

struct PropertyCollector<'o> {
    options: &'o ManglePropertiesOptions,
    key_annotated: &'o FxHashSet<u32>,
    state: PropertyCollectState,
}

fn is_eligible(options: &ManglePropertiesOptions, name: &str) -> bool {
    if is_hard_reserved(name)
        || is_canonical_numeric_string(name)
        || options.reserved.contains(name)
        || !options.include.is_match(name)
        || options.exclude.as_ref().is_some_and(|regex| regex.is_match(name))
    {
        return false;
    }
    if let Some(cached) = options.cache.get(name) {
        return cached.is_some();
    }
    true
}

impl<'o> PropertyCollector<'o> {
    fn new(options: &'o ManglePropertiesOptions, key_annotated: &'o FxHashSet<u32>) -> Self {
        Self { options, key_annotated, state: PropertyCollectState::default() }
    }

    fn candidate(&mut self, name: &str) {
        // Avoid constructing an owned key for repeated occurrences.
        if let Some(count) = self.state.frequencies.get_mut(name) {
            *count = count.saturating_add(1);
            return;
        }
        let count = self.state.frequencies.entry(CompactStr::from(name)).or_default();
        *count = count.saturating_add(1);
    }

    fn quoted(&mut self, name: &str) {
        if self.options.mangle_quoted {
            self.candidate(name);
        } else {
            self.occupy(name);
        }
    }

    fn occupy(&mut self, name: &str) {
        self.state.occupied.insert(CompactStr::from(name));
    }

    fn special_literal(&self, span: Span) -> bool {
        self.key_annotated.contains(&span.start)
    }

    fn observe_literal(&mut self, span: Span, name: &str) {
        if self.special_literal(span) {
            self.candidate(name);
        } else {
            // Compression can propagate or fold an arbitrary string into a property key after
            // property names have already been assigned. Keep every literal spelling out of the
            // generated namespace so that later materialization cannot merge distinct keys.
            self.occupy(name);
        }
    }

    fn classify_key_expression(&mut self, expression: &Expression<'_>) {
        match expression.get_inner_expression() {
            Expression::StringLiteral(literal) if self.special_literal(literal.span) => {}
            Expression::StringLiteral(literal) => self.quoted(literal.value.as_str()),
            Expression::TemplateLiteral(template)
                if template.expressions.is_empty() && self.special_literal(template.span) => {}
            Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
                if let [quasi] = template.quasis.as_slice()
                    && let Some(cooked) = quasi.value.cooked
                {
                    self.quoted(cooked.as_str());
                }
            }
            Expression::ConditionalExpression(expression) => {
                self.classify_key_expression(&expression.consequent);
                self.classify_key_expression(&expression.alternate);
            }
            Expression::SequenceExpression(expression) => {
                if let Some(last) = expression.expressions.last() {
                    self.classify_key_expression(last);
                }
            }
            Expression::NumericLiteral(literal) => self.occupy(&numeric_key_string(literal.value)),
            _ => {}
        }
    }

    fn classify_property_key(&mut self, key: &PropertyKey<'_>) {
        match key {
            PropertyKey::StaticIdentifier(identifier) => self.candidate(identifier.name.as_str()),
            PropertyKey::PrivateIdentifier(_) => {}
            PropertyKey::NumericLiteral(literal) => {
                self.occupy(&numeric_key_string(literal.value));
            }
            key => {
                if let Some(expression) = key.as_expression() {
                    self.classify_key_expression(expression);
                }
            }
        }
    }
}

impl<'a> Visit<'a> for PropertyCollector<'_> {
    fn visit_directive(&mut self, directive: &Directive<'a>) {
        // Directives are not property-name positions.
        self.occupy(directive.expression.value.as_str());
    }

    fn visit_ts_type(&mut self, _ty: &TSType<'a>) {}

    fn visit_ts_property_signature(&mut self, _signature: &TSPropertySignature<'a>) {}

    fn visit_ts_method_signature(&mut self, _signature: &TSMethodSignature<'a>) {}

    fn visit_static_member_expression(&mut self, expression: &StaticMemberExpression<'a>) {
        self.candidate(expression.property.name.as_str());
        walk::walk_static_member_expression(self, expression);
    }

    fn visit_computed_member_expression(&mut self, expression: &ComputedMemberExpression<'a>) {
        self.classify_key_expression(&expression.expression);
        walk::walk_computed_member_expression(self, expression);
    }

    fn visit_property_key(&mut self, key: &PropertyKey<'a>) {
        self.classify_property_key(key);
        walk::walk_property_key(self, key);
    }

    fn visit_assignment_target_property_identifier(
        &mut self,
        property: &AssignmentTargetPropertyIdentifier<'a>,
    ) {
        self.candidate(property.binding.name.as_str());
        walk::walk_assignment_target_property_identifier(self, property);
    }

    fn visit_jsx_attribute_name(&mut self, name: &JSXAttributeName<'a>) {
        match name {
            JSXAttributeName::Identifier(identifier) => self.candidate(identifier.name.as_str()),
            JSXAttributeName::NamespacedName(name) => {
                self.occupy(name.namespace.name.as_str());
                self.occupy(name.name.name.as_str());
            }
        }
        walk::walk_jsx_attribute_name(self, name);
    }

    fn visit_jsx_member_expression(&mut self, expression: &JSXMemberExpression<'a>) {
        self.candidate(expression.property.name.as_str());
        walk::walk_jsx_member_expression(self, expression);
    }

    fn visit_binary_expression(&mut self, expression: &BinaryExpression<'a>) {
        if expression.operator == BinaryOperator::In {
            self.classify_key_expression(&expression.left);
        }
        walk::walk_binary_expression(self, expression);
    }

    fn visit_string_literal(&mut self, literal: &StringLiteral<'a>) {
        self.observe_literal(literal.span, literal.value.as_str());
    }

    fn visit_template_literal(&mut self, template: &TemplateLiteral<'a>) {
        if template.expressions.is_empty()
            && let [quasi] = template.quasis.as_slice()
            && let Some(cooked) = quasi.value.cooked
        {
            self.observe_literal(template.span, cooked.as_str());
        }
        walk::walk_template_literal(self, template);
    }
}

fn collect(
    options: &ManglePropertiesOptions,
    program: &Program<'_>,
    key_annotated: &FxHashSet<u32>,
) -> PropertyCollectState {
    let mut collector = PropertyCollector::new(options, key_annotated);
    collector.visit_program(program);
    collector.state
}

fn debug_name(original: &str, attempt: u32) -> CompactStr {
    if is_identifier_name(original) {
        if attempt == 0 {
            CompactStr::from(format!("_${original}$_"))
        } else {
            CompactStr::from(format!("_${original}${attempt}$_"))
        }
    } else {
        CompactStr::from(format!("_$property{attempt}$_"))
    }
}

fn assign(
    options: &ManglePropertiesOptions,
    state: &PropertyCollectState,
) -> (PropertyMapping, ManglePropertyCache) {
    // Eligibility depends only on the name, so evaluate it once after collection.
    let mut names: Vec<_> =
        state.frequencies.iter().filter(|(name, _)| is_eligible(options, name.as_str())).collect();
    names.sort_unstable_by(|(name_a, count_a), (name_b, count_b)| {
        count_b.cmp(count_a).then_with(|| name_a.as_str().cmp(name_b.as_str()))
    });

    // Generated names must avoid collected names, reservations, cache keys, and pinned targets.
    let mut occupied = state.occupied.clone();
    occupied.extend(state.frequencies.keys().cloned());
    occupied.extend(options.reserved.iter().cloned());
    occupied.extend(options.cache.0.keys().cloned());
    occupied.extend(options.cache.0.values().flatten().cloned());

    let mut mapping = PropertyMapping::default();
    let mut cache = options.cache.clone();
    let mut counter = 0u32;

    for (original, _) in names {
        if let Some(Some(target)) = options.cache.get(original.as_str()) {
            mapping.insert(original.clone(), target.clone());
            continue;
        }

        let mut debug_attempt = 0u32;
        let target = loop {
            let candidate = if options.debug {
                let candidate = debug_name(original.as_str(), debug_attempt);
                debug_attempt =
                    debug_attempt.checked_add(1).expect("debug property name space exhausted");
                candidate
            } else {
                let candidate = CompactStr::from(base54(counter).as_str());
                counter = counter.checked_add(1).expect("property name space exhausted");
                candidate
            };
            if !occupied.contains(candidate.as_str()) && !is_hard_reserved(candidate.as_str()) {
                break candidate;
            }
        };
        occupied.insert(target.clone());
        cache.insert_generated(original.clone(), Some(target.clone()));
        mapping.insert(original.clone(), target);
    }

    (mapping, cache)
}

// Property keys can share syntax with local bindings. Expand shorthand when needed so only the
// property key changes.
struct PropertyRewriter<'a, 'm> {
    mapping: &'m PropertyMapping,
    mangle_quoted: bool,
    key_annotated: &'m FxHashSet<u32>,
    ast: AstBuilder<'a>,
}

impl<'a> PropertyRewriter<'a, '_> {
    fn special_literal(&self, span: Span) -> bool {
        self.key_annotated.contains(&span.start)
    }

    fn should_rewrite_literal(&self, span: Span) -> bool {
        self.key_annotated.contains(&span.start) || self.mangle_quoted
    }

    fn target(&self, original: &str) -> Option<&CompactStr> {
        self.mapping.get(original)
    }

    fn rename_string_literal(&self, literal: &mut StringLiteral<'a>) {
        if !self.should_rewrite_literal(literal.span) {
            return;
        }
        if let Some(target) = self.target(literal.value.as_str()) {
            literal.value = Str::from_str_in(target.as_str(), &self.ast);
            literal.raw = None;
        }
    }

    fn rename_template_literal(&self, template: &mut TemplateLiteral<'a>) {
        if !self.should_rewrite_literal(template.span) || !template.expressions.is_empty() {
            return;
        }
        if let [quasi] = template.quasis.as_mut_slice()
            && let Some(cooked) = quasi.value.cooked
            && let Some(target) = self.target(cooked.as_str())
        {
            let target = Str::from_str_in(target.as_str(), &self.ast);
            quasi.value.cooked = Some(target);
            quasi.value.raw = target;
        }
    }

    fn rename_key_expression(&self, expression: &mut Expression<'a>) {
        match expression.get_inner_expression_mut() {
            // Annotated literals were already handled during traversal.
            Expression::StringLiteral(literal) if !self.special_literal(literal.span) => {
                self.rename_string_literal(literal);
            }
            Expression::TemplateLiteral(template) if !self.special_literal(template.span) => {
                self.rename_template_literal(template);
            }
            Expression::ConditionalExpression(expression) => {
                self.rename_key_expression(&mut expression.consequent);
                self.rename_key_expression(&mut expression.alternate);
            }
            Expression::SequenceExpression(expression) => {
                if let Some(last) = expression.expressions.last_mut() {
                    self.rename_key_expression(last);
                }
            }
            _ => {}
        }
    }

    fn direct_string_key(key: &PropertyKey<'a>) -> Option<(CompactStr, Span)> {
        if let PropertyKey::StringLiteral(literal) = key {
            Some((CompactStr::from(literal.value.as_str()), literal.span))
        } else {
            None
        }
    }

    fn rewrite_key_position(
        &self,
        key: &mut PropertyKey<'a>,
        computed: &mut bool,
        original_string: Option<(CompactStr, Span)>,
    ) {
        if let Some((original, span)) = original_string
            && self.should_rewrite_literal(span)
            && let Some(target) = self.target(original.as_str())
        {
            *key = PropertyKey::StaticIdentifier(IdentifierName::boxed(
                span,
                Ident::from_str_in(target.as_str(), &self.ast),
                &self.ast,
            ));
            *computed = false;
            return;
        }
        if *computed && let Some(expression) = key.as_expression_mut() {
            self.rename_key_expression(expression);
        }
    }

    fn rename_static_key(&self, key: &mut PropertyKey<'a>) {
        if let PropertyKey::StaticIdentifier(identifier) = key
            && let Some(target) = self.target(identifier.name.as_str())
        {
            identifier.name = Ident::from_str_in(target.as_str(), &self.ast);
        }
    }
}

impl<'a> VisitMut<'a> for PropertyRewriter<'a, '_> {
    // Codegen emits the directive's separate raw value, so leave directives unchanged.
    fn visit_directive(&mut self, _directive: &mut Directive<'a>) {}

    fn visit_ts_type(&mut self, _ty: &mut TSType<'a>) {}

    fn visit_ts_property_signature(&mut self, _signature: &mut TSPropertySignature<'a>) {}

    fn visit_ts_method_signature(&mut self, _signature: &mut TSMethodSignature<'a>) {}

    fn visit_static_member_expression(&mut self, expression: &mut StaticMemberExpression<'a>) {
        let original = CompactStr::from(expression.property.name.as_str());
        walk_mut::walk_static_member_expression(self, expression);
        if let Some(target) = self.target(original.as_str()) {
            expression.property.name = Ident::from_str_in(target.as_str(), &self.ast);
        }
    }

    fn visit_expression(&mut self, expression: &mut Expression<'a>) {
        if let Expression::ComputedMemberExpression(member) = expression
            && let Expression::StringLiteral(literal) = &member.expression
        {
            let original = CompactStr::from(literal.value.as_str());
            let property_span = literal.span;
            if self.should_rewrite_literal(property_span)
                && let Some(target) = self.target(original.as_str())
            {
                let property = IdentifierName::new(
                    property_span,
                    Ident::from_str_in(target.as_str(), &self.ast),
                    &self.ast,
                );
                let replacement = StaticMemberExpression::boxed(
                    member.span,
                    member.object.take_in(&self.ast),
                    property,
                    member.optional,
                    &self.ast,
                );
                *expression = Expression::StaticMemberExpression(replacement);
                if let Expression::StaticMemberExpression(member) = expression {
                    self.visit_expression(&mut member.object);
                }
                return;
            }
        }
        walk_mut::walk_expression(self, expression);
    }

    fn visit_computed_member_expression(&mut self, expression: &mut ComputedMemberExpression<'a>) {
        walk_mut::walk_computed_member_expression(self, expression);
        self.rename_key_expression(&mut expression.expression);
    }

    fn visit_property_key(&mut self, key: &mut PropertyKey<'a>) {
        self.rename_static_key(key);
        walk_mut::walk_property_key(self, key);
    }

    fn visit_object_property(&mut self, property: &mut ObjectProperty<'a>) {
        let original_string = Self::direct_string_key(&property.key);
        let expands_shorthand = property.shorthand
            && matches!(&property.key, PropertyKey::StaticIdentifier(identifier) if self.target(identifier.name.as_str()).is_some());
        walk_mut::walk_object_property(self, property);
        if expands_shorthand {
            property.shorthand = false;
        }
        self.rewrite_key_position(&mut property.key, &mut property.computed, original_string);
    }

    fn visit_binding_property(&mut self, property: &mut BindingProperty<'a>) {
        let original_string = Self::direct_string_key(&property.key);
        let expands_shorthand = property.shorthand
            && matches!(&property.key, PropertyKey::StaticIdentifier(identifier) if self.target(identifier.name.as_str()).is_some());
        walk_mut::walk_binding_property(self, property);
        if expands_shorthand {
            property.shorthand = false;
        }
        self.rewrite_key_position(&mut property.key, &mut property.computed, original_string);
    }

    fn visit_assignment_target_property(&mut self, property: &mut AssignmentTargetProperty<'a>) {
        if let AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(identifier) = property {
            let original = CompactStr::from(identifier.binding.name.as_str());
            let target = self.target(original.as_str()).cloned();
            walk_mut::walk_assignment_target_property(self, property);
            if let Some(target) = target
                && let AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(identifier) =
                    property
            {
                let span = identifier.span;
                let binding_span = identifier.binding.span;
                let binding_name = identifier.binding.name;
                let reference_id = identifier.binding.reference_id.get();
                let init = identifier.init.take();
                let assignment_target = reference_id.map_or_else(
                    || {
                        AssignmentTarget::new_assignment_target_identifier(
                            binding_span,
                            binding_name,
                            &self.ast,
                        )
                    },
                    |reference_id| {
                        AssignmentTarget::new_assignment_target_identifier_with_reference_id(
                            binding_span,
                            binding_name,
                            reference_id,
                            &self.ast,
                        )
                    },
                );
                let binding = if let Some(init) = init {
                    AssignmentTargetMaybeDefault::new_assignment_target_with_default(
                        span,
                        assignment_target,
                        init,
                        &self.ast,
                    )
                } else {
                    match assignment_target {
                        AssignmentTarget::AssignmentTargetIdentifier(identifier) => {
                            AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(identifier)
                        }
                        _ => unreachable!(),
                    }
                };
                let name = PropertyKey::StaticIdentifier(IdentifierName::boxed(
                    binding_span,
                    Ident::from_str_in(target.as_str(), &self.ast),
                    &self.ast,
                ));
                *property = AssignmentTargetProperty::new_assignment_target_property_property(
                    span, name, binding, false, &self.ast,
                );
            }
            return;
        }
        walk_mut::walk_assignment_target_property(self, property);
    }

    fn visit_assignment_target_property_property(
        &mut self,
        property: &mut AssignmentTargetPropertyProperty<'a>,
    ) {
        let original_string = Self::direct_string_key(&property.name);
        walk_mut::walk_assignment_target_property_property(self, property);
        self.rewrite_key_position(&mut property.name, &mut property.computed, original_string);
    }

    fn visit_property_definition(&mut self, property: &mut PropertyDefinition<'a>) {
        let original_string = Self::direct_string_key(&property.key);
        walk_mut::walk_property_definition(self, property);
        self.rewrite_key_position(&mut property.key, &mut property.computed, original_string);
    }

    fn visit_accessor_property(&mut self, property: &mut AccessorProperty<'a>) {
        let original_string = Self::direct_string_key(&property.key);
        walk_mut::walk_accessor_property(self, property);
        self.rewrite_key_position(&mut property.key, &mut property.computed, original_string);
    }

    fn visit_method_definition(&mut self, method: &mut MethodDefinition<'a>) {
        let original_string = Self::direct_string_key(&method.key);
        walk_mut::walk_method_definition(self, method);
        self.rewrite_key_position(&mut method.key, &mut method.computed, original_string);
    }

    fn visit_binary_expression(&mut self, expression: &mut BinaryExpression<'a>) {
        walk_mut::walk_binary_expression(self, expression);
        if expression.operator == BinaryOperator::In {
            self.rename_key_expression(&mut expression.left);
        }
    }

    fn visit_jsx_attribute_name(&mut self, name: &mut JSXAttributeName<'a>) {
        if let JSXAttributeName::Identifier(identifier) = name {
            let original = CompactStr::from(identifier.name.as_str());
            walk_mut::walk_jsx_attribute_name(self, name);
            if let JSXAttributeName::Identifier(identifier) = name
                && let Some(target) = self.target(original.as_str())
            {
                identifier.name = Str::from_str_in(target.as_str(), &self.ast);
            }
        } else {
            walk_mut::walk_jsx_attribute_name(self, name);
        }
    }

    fn visit_jsx_member_expression(&mut self, expression: &mut JSXMemberExpression<'a>) {
        let original = CompactStr::from(expression.property.name.as_str());
        walk_mut::walk_jsx_member_expression(self, expression);
        if let Some(target) = self.target(original.as_str()) {
            expression.property.name = Str::from_str_in(target.as_str(), &self.ast);
        }
    }

    fn visit_string_literal(&mut self, literal: &mut StringLiteral<'a>) {
        if self.key_annotated.contains(&literal.span.start) {
            self.rename_string_literal(literal);
        }
    }

    fn visit_template_literal(&mut self, template: &mut TemplateLiteral<'a>) {
        if self.key_annotated.contains(&template.span.start) {
            self.rename_template_literal(template);
        }
        walk_mut::walk_template_literal(self, template);
    }
}

/// Three-phase property-name mangler.
pub struct PropertyMangler {
    options: ManglePropertiesOptions,
    state: PropertyCollectState,
    mapping: PropertyMapping,
    cache: ManglePropertyCache,
}

impl PropertyMangler {
    pub fn new(options: ManglePropertiesOptions) -> Self {
        Self {
            cache: ManglePropertyCache::default(),
            options,
            state: PropertyCollectState::default(),
            mapping: PropertyMapping::default(),
        }
    }

    /// Collect property occurrences and occupied spellings without mutating the AST.
    pub fn collect(&mut self, program: &Program<'_>) {
        let collection = PropertyMangleCollection::from_program(&self.options, program);
        self.merge_collected(collection);
    }

    /// Merge candidates collected independently with equivalent options.
    ///
    /// This allows callers to collect programs in parallel, merge the results, and assign one
    /// mapping.
    pub fn merge_collected(&mut self, collection: PropertyMangleCollection) {
        for (name, frequency) in collection.0.frequencies {
            let total = self.state.frequencies.entry(name).or_default();
            *total = total.saturating_add(frequency);
        }
        self.state.occupied.extend(collection.0.occupied);
    }

    /// Assign deterministic names by descending occurrence frequency and lexical tie-break.
    pub fn assign(&mut self) -> &PropertyMapping {
        let (mapping, cache) = assign(&self.options, &self.state);
        self.mapping = mapping;
        self.cache = cache;
        &self.mapping
    }

    /// Rewrite every selected occurrence once.
    ///
    /// Do not apply this mapping to already-rewritten code. Cache targets are allowed to equal
    /// other cache keys, so a second application can incorrectly follow a mapping chain.
    pub fn rewrite<'a>(&self, program: &mut Program<'a>, allocator: &'a Allocator) {
        if self.mapping.is_empty() {
            return;
        }
        let key_annotated = key_annotated_spans(program);
        let mut rewriter = PropertyRewriter {
            mapping: &self.mapping,
            mangle_quoted: self.options.mangle_quoted,
            key_annotated: &key_annotated,
            ast: AstBuilder::new(allocator),
        };
        rewriter.visit_program(program);
    }

    pub fn mapping(&self) -> &PropertyMapping {
        &self.mapping
    }

    pub fn cache(&self) -> &ManglePropertyCache {
        &self.cache
    }

    pub fn into_cache(self) -> ManglePropertyCache {
        self.cache
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(pattern: &str) -> ManglePropertiesOptions {
        ManglePropertiesOptions::new(lazy_regex::Regex::new(pattern).unwrap())
    }

    #[test]
    fn assignment_prefers_frequency_then_name() {
        let options = options("^_");
        let state = PropertyCollectState {
            frequencies: FxHashMap::from_iter([
                (CompactStr::from("_rare"), 1),
                (CompactStr::from("_often"), 3),
                (CompactStr::from("_also_often"), 3),
            ]),
            occupied: FxHashSet::default(),
        };
        let (mapping, _) = assign(&options, &state);
        assert_eq!(mapping["_also_often"].as_str(), "e");
        assert_eq!(mapping["_often"].as_str(), "t");
        assert_eq!(mapping["_rare"].as_str(), "n");
    }

    #[test]
    fn assignment_honors_cache_and_allows_duplicate_targets() {
        let mut options = options("^_");
        options.cache.insert("_a".into(), Some("A".into())).unwrap();
        options.cache.insert("_b".into(), Some("A".into())).unwrap();
        options.cache.insert("_keep".into(), None).unwrap();
        let state = PropertyCollectState {
            frequencies: FxHashMap::from_iter([
                (CompactStr::from("_a"), 1),
                (CompactStr::from("_b"), 1),
                (CompactStr::from("_auto"), 1),
            ]),
            occupied: FxHashSet::from_iter([CompactStr::from("_keep")]),
        };
        let (mapping, cache) = assign(&options, &state);
        assert_eq!(mapping["_a"].as_str(), "A");
        assert_eq!(mapping["_b"].as_str(), "A");
        assert_ne!(mapping["_auto"].as_str(), "A");
        assert_eq!(cache.get("_keep"), Some(&None));
    }

    #[test]
    fn assignment_filters_ineligible_collected_names() {
        let mut options = options("^(?:_|1$|constructor$)");
        options.exclude = Some(lazy_regex::Regex::new("^_excluded(?:One|Two)$").unwrap());
        options.reserved.insert("_reserved".into());
        options.cache.insert("_cached".into(), None).unwrap();
        let state = PropertyCollectState {
            frequencies: FxHashMap::from_iter([
                (CompactStr::from("_mangle"), 1),
                (CompactStr::from("_excludedOne"), 10),
                (CompactStr::from("_reserved"), 10),
                (CompactStr::from("_cached"), 10),
                (CompactStr::from("public"), 10),
                (CompactStr::from("1"), 10),
                (CompactStr::from("constructor"), 10),
            ]),
            occupied: FxHashSet::default(),
        };
        let (mapping, cache) = assign(&options, &state);
        assert_eq!(mapping.len(), 1);
        assert!(mapping.contains_key("_mangle"));
        assert_eq!(cache.get("_cached"), Some(&None));
    }

    #[test]
    fn hard_reservations_are_exact() {
        assert!(is_hard_reserved("__proto__"));
        assert!(is_hard_reserved("constructor"));
        assert!(is_hard_reserved("prototype"));
        assert!(!is_hard_reserved("toString"));
        assert!(!is_hard_reserved("then"));
    }

    #[test]
    fn cache_targets_are_identifier_names() {
        assert!(is_valid_property_mangle_cache_target("valid$name"));
        assert!(!is_valid_property_mangle_cache_target("not-valid"));
        assert!(!is_valid_property_mangle_cache_target("__proto__"));
        assert!(!is_valid_property_mangle_cache_target("constructor"));
        assert!(!is_valid_property_mangle_cache_target("prototype"));
    }

    #[test]
    fn invalid_cache_targets_cannot_enter_the_rust_engine() {
        let mut cache = ManglePropertyCache::default();
        for target in ["a-b", "__proto__", "constructor", "prototype", ""] {
            let error = cache.insert("_field".into(), Some(target.into())).unwrap_err();
            assert_eq!(error.original.as_str(), "_field");
            assert_eq!(error.target.as_str(), target);
            assert!(cache.is_empty());
        }
        cache.insert("_field".into(), Some("valid$name".into())).unwrap();
        assert_eq!(cache.get("_field").and_then(Option::as_deref), Some("valid$name"));
    }
}
