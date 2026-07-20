//! Property-name mangling.
//!
//! Property mangling is deliberately separate from identifier mangling. The engine runs in
//! three phases so callers that process multiple programs can collect globally, assign one
//! deterministic mapping, and then rewrite each program exactly once.

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
    /// Returns an error when `target` is not an `IdentifierName` or is `"__proto__"`.
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

    fn insert_generated(&mut self, original: CompactStr, target: Option<CompactStr>) {
        self.0.insert(original, target);
    }
}

impl std::ops::Deref for ManglePropertyCache {
    type Target = FxHashMap<CompactStr, Option<CompactStr>>;

    fn deref(&self) -> &Self::Target {
        &self.0
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
            "invalid property-mangle cache target for '{}': '{}' must be an IdentifierName other than '__proto__'",
            self.original, self.target
        )
    }
}

impl std::error::Error for InvalidManglePropertyCacheTarget {}

/// A mapping from original property names to their final names.
pub type PropertyMapping = FxHashMap<CompactStr, CompactStr>;

/// The syntactic class of a property key before a transform converted it into a string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyKeyOrigin {
    /// The string came from an identifier-like key such as `obj.foo`.
    Unquoted,
    /// The string came from a quoted key such as `obj["foo"]`.
    Quoted,
}

/// Transformer provenance for strings derived from property keys.
///
/// Spans are only meaningful within the `Program` that produced them. Callers must not carry
/// this map across a print-and-reparse boundary.
pub type PropertyKeyProvenance = FxHashMap<Span, PropertyKeyOrigin>;

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
    name != "__proto__" && is_identifier_name(name)
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

fn is_key_annotation(comment: &Comment, source_text: &str) -> bool {
    let text = comment.content_span().source_text(source_text).trim();
    matches!(text.strip_prefix(['@', '#']), Some("__KEY__"))
}

fn key_annotated_spans(program: &Program<'_>) -> FxHashSet<u32> {
    program
        .comments
        .iter()
        .filter(|comment| {
            comment.position == CommentPosition::Leading
                && is_key_annotation(comment, program.source_text)
        })
        .map(|comment| comment.attached_to)
        .collect()
}

/// Data gathered by the read-only collection phase.
#[derive(Debug, Default, Clone)]
pub struct PropertyCollectState {
    /// Number of rewriteable occurrences for each property name.
    pub frequencies: FxHashMap<CompactStr, u32>,
    /// Spellings that remain in the output and therefore cannot be automatic output names.
    pub occupied: FxHashSet<CompactStr>,
}

struct PropertyCollector<'o> {
    options: &'o ManglePropertiesOptions,
    provenance: Option<&'o PropertyKeyProvenance>,
    key_annotated: &'o FxHashSet<u32>,
    state: PropertyCollectState,
}

impl<'o> PropertyCollector<'o> {
    fn new(
        options: &'o ManglePropertiesOptions,
        provenance: Option<&'o PropertyKeyProvenance>,
        key_annotated: &'o FxHashSet<u32>,
    ) -> Self {
        Self { options, provenance, key_annotated, state: PropertyCollectState::default() }
    }

    fn is_eligible(&self, name: &str) -> bool {
        if is_hard_reserved(name)
            || is_canonical_numeric_string(name)
            || self.options.reserved.contains(name)
            || !self.options.include.is_match(name)
            || self.options.exclude.as_ref().is_some_and(|regex| regex.is_match(name))
        {
            return false;
        }
        if let Some(cached) = self.options.cache.get(name) {
            return cached.is_some();
        }
        true
    }

    fn candidate(&mut self, name: &str) {
        if self.is_eligible(name) {
            let count = self.state.frequencies.entry(CompactStr::from(name)).or_default();
            *count = count.saturating_add(1);
        } else {
            self.occupy(name);
        }
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
            || self.provenance.is_some_and(|origins| origins.contains_key(&span))
    }

    fn classify_literal(&mut self, span: Span, name: &str) {
        if self.key_annotated.contains(&span.start) {
            self.candidate(name);
            return;
        }
        match self.provenance.and_then(|origins| origins.get(&span)) {
            Some(PropertyKeyOrigin::Unquoted) => self.candidate(name),
            Some(PropertyKeyOrigin::Quoted) => self.quoted(name),
            None => {}
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
                    self.occupy(cooked.as_str());
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
        self.classify_literal(literal.span, literal.value.as_str());
    }

    fn visit_template_literal(&mut self, template: &TemplateLiteral<'a>) {
        if template.expressions.is_empty()
            && self.key_annotated.contains(&template.span.start)
            && let [quasi] = template.quasis.as_slice()
            && let Some(cooked) = quasi.value.cooked
        {
            self.candidate(cooked.as_str());
        }
        walk::walk_template_literal(self, template);
    }
}

fn collect(
    options: &ManglePropertiesOptions,
    program: &Program<'_>,
    provenance: Option<&PropertyKeyProvenance>,
    key_annotated: &FxHashSet<u32>,
) -> PropertyCollectState {
    let mut collector = PropertyCollector::new(options, provenance, key_annotated);
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
    let mut names: Vec<_> = state.frequencies.iter().collect();
    names.sort_unstable_by(|(name_a, count_a), (name_b, count_b)| {
        count_b.cmp(count_a).then_with(|| name_a.as_str().cmp(name_b.as_str()))
    });

    let mut occupied = state.occupied.clone();
    occupied.extend(state.frequencies.keys().cloned());
    occupied.extend(options.reserved.iter().cloned());
    occupied.extend(options.cache.keys().cloned());
    occupied.extend(options.cache.values().flatten().cloned());

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

struct PropertyRewriter<'a, 'm> {
    mapping: &'m PropertyMapping,
    mangle_quoted: bool,
    provenance: Option<&'m PropertyKeyProvenance>,
    key_annotated: &'m FxHashSet<u32>,
    rewritten: FxHashSet<Span>,
    ast: AstBuilder<'a>,
}

impl<'a> PropertyRewriter<'a, '_> {
    fn should_rewrite_literal(&self, span: Span, quoted: bool) -> bool {
        if self.key_annotated.contains(&span.start) {
            return true;
        }
        match self.provenance.and_then(|origins| origins.get(&span)) {
            Some(PropertyKeyOrigin::Unquoted) => true,
            Some(PropertyKeyOrigin::Quoted) => self.mangle_quoted,
            None => !quoted || self.mangle_quoted,
        }
    }

    fn target(&self, original: &str) -> Option<&CompactStr> {
        self.mapping.get(original)
    }

    fn rename_string_literal(&mut self, literal: &mut StringLiteral<'a>, quoted: bool) {
        if self.rewritten.contains(&literal.span)
            || !self.should_rewrite_literal(literal.span, quoted)
        {
            return;
        }
        if let Some(target) = self.target(literal.value.as_str()) {
            literal.value = Str::from_str_in(target.as_str(), &self.ast);
            literal.raw = None;
            self.rewritten.insert(literal.span);
        }
    }

    fn rename_template_literal(&mut self, template: &mut TemplateLiteral<'a>, quoted: bool) {
        if self.rewritten.contains(&template.span)
            || !self.should_rewrite_literal(template.span, quoted)
            || !template.expressions.is_empty()
        {
            return;
        }
        if let [quasi] = template.quasis.as_mut_slice()
            && let Some(cooked) = quasi.value.cooked
            && let Some(target) = self.target(cooked.as_str())
        {
            let target = Str::from_str_in(target.as_str(), &self.ast);
            quasi.value.cooked = Some(target);
            quasi.value.raw = target;
            self.rewritten.insert(template.span);
        }
    }

    fn rename_key_expression(&mut self, expression: &mut Expression<'a>) {
        match expression.get_inner_expression_mut() {
            Expression::StringLiteral(literal) => self.rename_string_literal(literal, true),
            Expression::TemplateLiteral(template)
                if self.key_annotated.contains(&template.span.start)
                    || self
                        .provenance
                        .is_some_and(|origins| origins.contains_key(&template.span)) =>
            {
                self.rename_template_literal(template, true);
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
        &mut self,
        key: &mut PropertyKey<'a>,
        computed: &mut bool,
        original_string: Option<(CompactStr, Span)>,
    ) {
        if let Some((original, span)) = original_string
            && self.should_rewrite_literal(span, true)
            && let Some(target) = self.target(original.as_str())
        {
            *key = PropertyKey::StaticIdentifier(IdentifierName::boxed(
                span,
                Ident::from_str_in(target.as_str(), &self.ast),
                &self.ast,
            ));
            *computed = false;
            self.rewritten.insert(span);
            return;
        }
        if *computed && let Some(expression) = key.as_expression_mut() {
            self.rename_key_expression(expression);
        }
    }

    fn rename_static_key(&mut self, key: &mut PropertyKey<'a>) {
        if let PropertyKey::StaticIdentifier(identifier) = key
            && !self.rewritten.contains(&identifier.span)
            && let Some(target) = self.target(identifier.name.as_str())
        {
            identifier.name = Ident::from_str_in(target.as_str(), &self.ast);
            self.rewritten.insert(identifier.span);
        }
    }
}

impl<'a> VisitMut<'a> for PropertyRewriter<'a, '_> {
    fn visit_static_member_expression(&mut self, expression: &mut StaticMemberExpression<'a>) {
        let original = CompactStr::from(expression.property.name.as_str());
        walk_mut::walk_static_member_expression(self, expression);
        if !self.rewritten.contains(&expression.property.span)
            && let Some(target) = self.target(original.as_str())
        {
            expression.property.name = Ident::from_str_in(target.as_str(), &self.ast);
            self.rewritten.insert(expression.property.span);
        }
    }

    fn visit_expression(&mut self, expression: &mut Expression<'a>) {
        if let Expression::ComputedMemberExpression(member) = expression
            && let Expression::StringLiteral(literal) = &member.expression
        {
            let original = CompactStr::from(literal.value.as_str());
            let property_span = literal.span;
            if self.should_rewrite_literal(property_span, true)
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
                self.rewritten.insert(property_span);
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
                self.rewritten.insert(binding_span);
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
        if self.key_annotated.contains(&literal.span.start)
            || self.provenance.is_some_and(|origins| origins.contains_key(&literal.span))
        {
            self.rename_string_literal(literal, true);
        }
    }

    fn visit_template_literal(&mut self, template: &mut TemplateLiteral<'a>) {
        if self.key_annotated.contains(&template.span.start) {
            self.rename_template_literal(template, true);
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
            cache: options.cache.clone(),
            options,
            state: PropertyCollectState::default(),
            mapping: PropertyMapping::default(),
        }
    }

    /// Collect rewriteable occurrences and occupied spellings without mutating the AST.
    pub fn collect(&mut self, program: &Program<'_>, provenance: Option<&PropertyKeyProvenance>) {
        let key_annotated = key_annotated_spans(program);
        let state = collect(&self.options, program, provenance, &key_annotated);
        for (name, frequency) in state.frequencies {
            let total = self.state.frequencies.entry(name).or_default();
            *total = total.saturating_add(frequency);
        }
        self.state.occupied.extend(state.occupied);
    }

    /// Assign deterministic names by descending occurrence frequency and lexical tie-break.
    pub fn assign(&mut self) -> &PropertyMapping {
        let (mapping, cache) = assign(&self.options, &self.state);
        self.mapping = mapping;
        self.cache = cache;
        &self.mapping
    }

    /// Rewrite every selected occurrence once.
    pub fn rewrite<'a>(
        &self,
        program: &mut Program<'a>,
        allocator: &'a Allocator,
        provenance: Option<&PropertyKeyProvenance>,
    ) {
        if self.mapping.is_empty() {
            return;
        }
        let key_annotated = key_annotated_spans(program);
        let mut rewriter = PropertyRewriter {
            mapping: &self.mapping,
            mangle_quoted: self.options.mangle_quoted,
            provenance,
            key_annotated: &key_annotated,
            rewritten: FxHashSet::default(),
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
        assert_eq!(cache["_keep"], None);
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
    }

    #[test]
    fn invalid_cache_targets_cannot_enter_the_rust_engine() {
        let mut cache = ManglePropertyCache::default();
        for target in ["a-b", "__proto__", ""] {
            let error = cache.insert("_field".into(), Some(target.into())).unwrap_err();
            assert_eq!(error.original.as_str(), "_field");
            assert_eq!(error.target.as_str(), target);
            assert!(cache.is_empty());
        }
        cache.insert("_field".into(), Some("valid$name".into())).unwrap();
        assert_eq!(cache["_field"].as_deref(), Some("valid$name"));
    }
}
