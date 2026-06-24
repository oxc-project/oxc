//! Property-name mangling engine.
//!
//! This module implements opt-in property-name mangling (`obj.longName` -> `obj.a`).
//! It is **off by default**: nothing is mangled unless the user supplies a `mangle`
//! regex via [`ManglePropertiesOptions`].
//!
//! This file contains the whole engine: the option/cache types, the eligibility check,
//! the name-assignment function, the read-only collect pass, the in-place rewrite pass,
//! and the [`PropertyMangler`] driver that runs the two halves around compress/mangle.

use oxc_allocator::{Allocator, TakeIn};
use oxc_ast::{
    AstBuilder,
    ast::{
        AssignmentTargetPropertyIdentifier, AssignmentTargetPropertyProperty, BinaryExpression,
        BinaryOperator, CallExpression, ComputedMemberExpression, Expression, JSXAttributeName,
        NewExpression, Program, PropertyKey, StaticMemberExpression, StringLiteral, WithStatement,
    },
};
use oxc_ast_visit::{
    Visit, VisitMut,
    walk::{
        walk_assignment_target_property_identifier, walk_assignment_target_property_property,
        walk_binary_expression, walk_call_expression, walk_computed_member_expression,
        walk_jsx_attribute_name, walk_new_expression, walk_property_key,
        walk_static_member_expression, walk_with_statement,
    },
    walk_mut,
};
use oxc_mangler::base54;
use oxc_str::CompactStr;
use rustc_hash::{FxHashMap, FxHashSet};

/// Property names that are always reserved regardless of the user's regex.
///
/// These are well-known protocol / interop names whose mangling would break
/// reflection, JSON serialization, promises, or common host behavior.
const PROTOCOL_DENYLIST: &[&str] =
    &["then", "toJSON", "toString", "valueOf", "length", "name", "message"];

/// Options controlling property mangling.
///
/// Feature is **off** when `mangle` is `None`.
#[derive(Default, Clone, Debug)]
pub struct ManglePropertiesOptions {
    /// Names matching this regex are candidates for mangling. `None` => feature off.
    pub mangle: Option<lazy_regex::Regex>,
    /// Names matching this regex are reserved (never mangled), even if `mangle` matches.
    pub reserve: Option<lazy_regex::Regex>,
    /// Explicit reserved names. Added to (never replaces) the always-reserved set.
    pub reserved: FxHashSet<CompactStr>,
    /// Whether to mangle quoted keys.
    ///
    /// When `false` (default), a quoted/computed string in a property-key position
    /// (`x['_foo']`, `{'_foo':1}`, `'_foo' in x`) reserves that name program-wide.
    /// When `true`, such strings become mangle candidates (subject to the same
    /// eligibility check) and are renamed consistently with their unquoted siblings.
    pub mangle_quoted: bool,
    /// Whether to emit human-readable debug names. v1: always `false` (deferred).
    pub debug: bool,
    /// Cross-build name cache (old -> new / reserved).
    pub cache: PropertyMangleCache,
}

/// Persistent old-name -> assigned-name cache, so repeated builds produce stable names.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct PropertyMangleCache {
    pub map: FxHashMap<CompactStr, CacheValue>,
}

/// A cached decision for a property name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CacheValue {
    /// The name was mangled to this new name.
    Name(CompactStr),
    /// The name is reserved and must never be mangled.
    Reserved,
}

/// Whether `name` is always reserved, regardless of the user's regex.
fn is_always_reserved(name: &str) -> bool {
    matches!(name, "__proto__" | "constructor" | "prototype") || PROTOCOL_DENYLIST.contains(&name)
}

/// Whether `name` is eligible for mangling under `opts`.
fn eligible(opts: &ManglePropertiesOptions, name: &str) -> bool {
    opts.mangle.as_ref().is_some_and(|re| re.is_match(name))
        && !opts.reserve.as_ref().is_some_and(|re| re.is_match(name))
        && !opts.reserved.contains(name)
        && !is_always_reserved(name)
}

/// Assign final mangled names.
///
/// `candidates` are eligible unquoted names; `reserved` are program-wide reservations.
/// Returns the old -> new map and mutates the shared `cache`.
///
/// The iteration order is deterministic (sorted) so that a shared cache reproduces
/// the same names across builds, and the produced names are pairwise disjoint and
/// never collide with reserved/always-reserved names.
///
/// When a cached name (`CacheValue::Name`) cannot be honored because its target
/// collides with something already occupied this build (a program-wide reservation,
/// an always-reserved name, a `Reserved`-pinned candidate's original spelling, or a
/// name already claimed by another candidate), the candidate is **regenerated** a
/// fresh valid name and the cache is rewritten — it is never left with its original
/// spelling. The only names that keep their original spelling are cache `Reserved`
/// entries (the user explicitly pinned them); those are marked occupied up front so
/// they can never be handed to another candidate.
fn assign(
    candidates: &FxHashSet<CompactStr>,
    reserved: &FxHashSet<CompactStr>,
    cache: &mut PropertyMangleCache,
) -> FxHashMap<CompactStr, CompactStr> {
    // Deterministic order so a shared cache is reproducible.
    let mut names: Vec<&CompactStr> = candidates.difference(reserved).collect();
    names.sort_unstable();

    // Candidates the user pinned as Reserved keep their original spelling, so those
    // names are occupied and must never be handed to another candidate.
    let kept: FxHashSet<CompactStr> = names
        .iter()
        .filter(|n| matches!(cache.map.get(**n), Some(CacheValue::Reserved)))
        .map(|n| (*n).clone())
        .collect();

    // Existing cache targets are avoided when GENERATING new names, so cross-build
    // reuse stays stable (a future build honoring the cache won't collide).
    let existing_targets: FxHashSet<CompactStr> = cache
        .map
        .values()
        .filter_map(|v| if let CacheValue::Name(n) = v { Some(n.clone()) } else { None })
        .collect();

    let mut claimed: FxHashSet<CompactStr> = FxHashSet::default(); // outputs assigned this build
    let mut counter: u32 = 0;
    let mut map = FxHashMap::default();

    for name in names {
        match cache.map.get(name) {
            // Pinned: keep the original name, don't mangle.
            Some(CacheValue::Reserved) => {}
            // Honor the cached name only if it doesn't collide with anything occupied.
            Some(CacheValue::Name(n))
                if !reserved.contains(n.as_str())
                    && !is_always_reserved(n)
                    && !kept.contains(n)
                    && !claimed.contains(n) =>
            {
                map.insert(name.clone(), n.clone());
                claimed.insert(n.clone());
            }
            // No cache entry, or the cached name collided -> generate a fresh valid name
            // and (re)write it into the cache.
            _ => {
                let n = loop {
                    let c = CompactStr::from(base54(counter).as_str());
                    counter = counter.checked_add(1).expect("property name space exhausted");
                    if !reserved.contains(&c)
                        && !is_always_reserved(&c)
                        && !kept.contains(&c)
                        && !claimed.contains(&c)
                        && !existing_targets.contains(&c)
                    {
                        break c;
                    }
                };
                map.insert(name.clone(), n.clone());
                claimed.insert(n.clone());
                cache.map.insert(name.clone(), CacheValue::Name(n));
            }
        }
    }
    map
}

/// The result of the read-only collect pass over the original (pre-compress) program.
///
/// `candidates` are eligible names seen unquoted (mangle these); `reserved` are names
/// seen in a position that must keep its spelling (quoted/computed keys, the LHS string
/// of `'x' in obj`, JSX attribute names, assignment-target shorthands). `bail` is set when
/// the program contains `with` or a direct `eval` / `Function` constructor, which makes
/// property mangling unsafe for the whole program.
#[derive(Default)]
pub struct PropertyCollectState {
    /// Eligible names seen unquoted.
    pub candidates: FxHashSet<CompactStr>,
    /// Names that must never be mangled (quoted/computed/in-LHS/JSX-attr/assignment-target).
    pub reserved: FxHashSet<CompactStr>,
    /// `with` or direct `eval` / `Function` present anywhere => disable mangling.
    pub bail: bool,
}

/// Read-only visitor that classifies every property-bearing position in the program.
struct PropertyCollector<'a, 'o> {
    opts: &'o ManglePropertiesOptions,
    state: PropertyCollectState,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a, 'o> PropertyCollector<'a, 'o> {
    fn new(opts: &'o ManglePropertiesOptions) -> Self {
        Self { opts, state: PropertyCollectState::default(), _marker: std::marker::PhantomData }
    }

    /// An unquoted occurrence: mangle it if eligible, otherwise it is reserved program-wide.
    fn candidate(&mut self, name: &str) {
        if eligible(self.opts, name) {
            self.state.candidates.insert(CompactStr::from(name));
        } else {
            self.state.reserved.insert(CompactStr::from(name));
        }
    }

    /// A quoted/computed/never-mangle occurrence.
    ///
    /// When `mangle_quoted` is on, the name is instead treated as a candidate (the same
    /// eligibility rules apply), so it can be renamed consistently with unquoted siblings.
    /// When off, it is reserved program-wide (the safe default).
    fn quoted(&mut self, name: &str) {
        if self.opts.mangle_quoted {
            self.candidate(name);
        } else {
            self.state.reserved.insert(CompactStr::from(name));
        }
    }

    /// Reserve a name program-wide, regardless of `mangle_quoted`.
    fn reserve(&mut self, name: &str) {
        self.state.reserved.insert(CompactStr::from(name));
    }

    /// Classify a quoted/computed string occurrence in a key/index position, recursing
    /// through the wrapped forms esbuild handles: conditionals (`y ? '_a' : '_b'`) and
    /// comma/sequence expressions (`(y, '_a')`). Only the statically-reachable key
    /// strings are classified; non-literal sub-expressions are ignored (they are not
    /// statically-known keys).
    fn classify_key_expression(&mut self, expr: &Expression<'a>) {
        match expr.get_inner_expression() {
            Expression::StringLiteral(lit) => self.quoted(lit.value.as_str()),
            Expression::ConditionalExpression(cond) => {
                self.classify_key_expression(&cond.consequent);
                self.classify_key_expression(&cond.alternate);
            }
            Expression::SequenceExpression(seq) => {
                if let Some(last) = seq.expressions.last() {
                    self.classify_key_expression(last);
                }
            }
            _ => {}
        }
    }

    /// Classify a [`PropertyKey`] (object/binding/class member key, or assignment-target name).
    ///
    /// - `{ foo: 1 }` -> candidate `foo`.
    /// - `{ '_a': 1 }` / `{ ['_a']: 1 }` -> quoted `_a` (bare string key; the computed flag
    ///   is irrelevant to the name, and the rewrite un-quotes both forms identically).
    /// - `{ [(y, '_a')]: 1 }` / `{ [y ? '_a' : z]: 1 }` -> the wrapped computed forms recurse
    ///   to the statically-reachable key strings.
    /// - `{ 0: 1 }` -> reserve the numeric key.
    /// - `{ [foo('_keep')]: 1 }`, `{ [x]: 1 }`, `#priv` -> nothing (not a statically-known key).
    fn classify_property_key(&mut self, key: &PropertyKey<'a>) {
        match key {
            PropertyKey::StaticIdentifier(ident) => self.candidate(ident.name.as_str()),
            PropertyKey::NumericLiteral(lit) => self.reserve(&lit.value.to_string()),
            PropertyKey::PrivateIdentifier(_) => {}
            // A bare string key, or a wrapped computed key (sequence / conditional). Both are
            // classified through the key-expression recursion, which handles the string
            // literal and ignores non-literal sub-expressions.
            key => {
                if let Some(expr) = key.as_expression() {
                    self.classify_key_expression(expr);
                }
            }
        }
    }
}

impl<'a> Visit<'a> for PropertyCollector<'a, '_> {
    fn visit_static_member_expression(&mut self, it: &StaticMemberExpression<'a>) {
        self.candidate(it.property.name.as_str());
        walk_static_member_expression(self, it);
    }

    fn visit_computed_member_expression(&mut self, it: &ComputedMemberExpression<'a>) {
        // `x['_a']`, and the wrapped forms `x[y ? '_a' : z]` / `x[(y, '_a')]`.
        // A non-literal index (`x[foo('_a')]`) is not a statically-known key: skipped.
        self.classify_key_expression(&it.expression);
        walk_computed_member_expression(self, it);
    }

    fn visit_property_key(&mut self, it: &PropertyKey<'a>) {
        // `visit_property_key` is reached for object / class / binding member keys. It cannot
        // see the parent's `computed` flag, but the name is the same whether the key is
        // `{'_a':1}` or `{['_a']:1}`, and the wrapped computed forms (`{[(y,'_a')]:1}`) are
        // handled by recursing into the key expression.
        self.classify_property_key(it);
        walk_property_key(self, it);
    }

    fn visit_assignment_target_property_identifier(
        &mut self,
        it: &AssignmentTargetPropertyIdentifier<'a>,
    ) {
        // The shorthand `({ foo } = obj)` is reserved, never mangled.
        self.reserve(it.binding.name.as_str());
        walk_assignment_target_property_identifier(self, it);
    }

    fn visit_assignment_target_property_property(
        &mut self,
        it: &AssignmentTargetPropertyProperty<'a>,
    ) {
        self.classify_property_key(&it.name);
        walk_assignment_target_property_property(self, it);
    }

    fn visit_jsx_attribute_name(&mut self, it: &JSXAttributeName<'a>) {
        // A JSX attribute becomes a props key, so reserve it.
        if let JSXAttributeName::Identifier(ident) = it {
            self.reserve(ident.name.as_str());
        }
        walk_jsx_attribute_name(self, it);
    }

    fn visit_binary_expression(&mut self, it: &BinaryExpression<'a>) {
        // `'foo' in obj`, plus the wrapped forms `(y ? '_a' : z) in obj` / `(y, '_a') in obj`.
        if it.operator == BinaryOperator::In {
            self.classify_key_expression(&it.left);
        }
        walk_binary_expression(self, it);
    }

    fn visit_with_statement(&mut self, it: &WithStatement<'a>) {
        self.state.bail = true;
        walk_with_statement(self, it);
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Expression::Identifier(ident) = &it.callee
            && matches!(ident.name.as_str(), "eval" | "Function")
        {
            self.state.bail = true;
        }
        walk_call_expression(self, it);
    }

    fn visit_new_expression(&mut self, it: &NewExpression<'a>) {
        if let Expression::Identifier(ident) = &it.callee
            && ident.name.as_str() == "Function"
        {
            self.state.bail = true;
        }
        walk_new_expression(self, it);
    }
}

/// Walk the **original** (pre-compress) program and classify every property occurrence.
///
/// Returns the candidate/reserved sets and the whole-program `bail` flag.
fn collect(opts: &ManglePropertiesOptions, program: &Program) -> PropertyCollectState {
    let mut collector = PropertyCollector::new(opts);
    collector.visit_program(program);
    collector.state
}

/// Mutable visitor that renames every property occurrence whose name is in `map`.
///
/// Renamed positions are unquoted member properties (`StaticMemberExpression.property`)
/// and `StaticIdentifier` property keys. When `mangle_quoted` is enabled, quoted/computed
/// string keys are also renamed:
/// - A direct string index `x['_a']` is un-quoted to `x.<new>` (a base54 name is always a
///   valid identifier). Optional chaining is preserved.
/// - A direct object/class string key `{'_a':1}` / `{['_a']:1}` becomes a `StaticIdentifier`
///   key with `computed = false`.
/// - The `in`-LHS string and the wrapped computed forms (`(y, '_a')`, `y ? '_a' : z`) keep
///   their string-literal shape but are renamed in place.
///
/// `AssignmentTargetPropertyIdentifier` shorthands are **not** renamed: their names were
/// added to the reserved set during collect, so they never appear in `map`.
struct PropertyRewriter<'a, 'm> {
    /// Old name -> new (mangled) name.
    map: &'m FxHashMap<CompactStr, CompactStr>,
    /// Whether quoted keys are renamed (else only the unquoted positions are touched).
    mangle_quoted: bool,
    /// Allocates the new name strings into the program's arena.
    ast: AstBuilder<'a>,
}

impl<'a> PropertyRewriter<'a, '_> {
    /// Rename an in-place string literal (used for the `in`-LHS and wrapped computed keys).
    fn rename_string_literal(&self, lit: &mut StringLiteral<'a>) {
        if let Some(new_name) = self.map.get(lit.value.as_str()) {
            lit.value = self.ast.str(new_name.as_str());
        }
    }

    /// Rename the string literals reachable through the wrapped key forms (conditional /
    /// sequence), in place. Used for the `in`-LHS, member-index, and computed-key wrappers
    /// that cannot be un-quoted.
    fn rename_key_expression(&self, expr: &mut Expression<'a>) {
        match expr {
            Expression::StringLiteral(lit) => self.rename_string_literal(lit),
            Expression::ParenthesizedExpression(paren) => {
                self.rename_key_expression(&mut paren.expression);
            }
            Expression::ConditionalExpression(cond) => {
                self.rename_key_expression(&mut cond.consequent);
                self.rename_key_expression(&mut cond.alternate);
            }
            Expression::SequenceExpression(seq) => {
                if let Some(last) = seq.expressions.last_mut() {
                    self.rename_key_expression(last);
                }
            }
            _ => {}
        }
    }

    /// Rename a property key that may be a string literal, un-quoting it to a
    /// `StaticIdentifier` and clearing the parent's `computed` flag when possible. Returns
    /// `true` if the key was a (direct) string literal it handled.
    fn rewrite_string_key(&self, key: &mut PropertyKey<'a>, computed: &mut bool) -> bool {
        if let PropertyKey::StringLiteral(lit) = key {
            if let Some(new_name) = self.map.get(lit.value.as_str()) {
                let ident =
                    self.ast.alloc_identifier_name(lit.span, self.ast.ident(new_name.as_str()));
                *key = PropertyKey::StaticIdentifier(ident);
                *computed = false;
            }
            true
        } else {
            false
        }
    }
}

impl<'a> VisitMut<'a> for PropertyRewriter<'a, '_> {
    fn visit_static_member_expression(&mut self, it: &mut StaticMemberExpression<'a>) {
        if let Some(new_name) = self.map.get(it.property.name.as_str()) {
            it.property.name = self.ast.ident(new_name.as_str());
        }
        walk_mut::walk_static_member_expression(self, it);
    }

    fn visit_expression(&mut self, it: &mut Expression<'a>) {
        // Un-quote a direct string index `x['_a']` -> `x.<new>`, preserving optional chaining.
        // Wrapped index forms (`x[y ? '_a' : z]`) are renamed in place via the
        // computed-member visitor below.
        if self.mangle_quoted
            && let Expression::ComputedMemberExpression(member) = it
            && let Expression::StringLiteral(lit) = &member.expression
            && let Some(new_name) = self.map.get(lit.value.as_str())
        {
            let property = self.ast.identifier_name(lit.span, self.ast.ident(new_name.as_str()));
            let new_member = self.ast.alloc_static_member_expression(
                member.span,
                member.object.take_in(&self.ast.allocator),
                property,
                member.optional,
            );
            *it = Expression::StaticMemberExpression(new_member);
        }
        walk_mut::walk_expression(self, it);
    }

    fn visit_computed_member_expression(&mut self, it: &mut ComputedMemberExpression<'a>) {
        // Wrapped index forms that cannot be un-quoted: rename in place.
        if self.mangle_quoted {
            self.rename_key_expression(&mut it.expression);
        }
        walk_mut::walk_computed_member_expression(self, it);
    }

    fn visit_property_key(&mut self, it: &mut PropertyKey<'a>) {
        if let PropertyKey::StaticIdentifier(ident) = it
            && let Some(new_name) = self.map.get(ident.name.as_str())
        {
            ident.name = self.ast.ident(new_name.as_str());
        }
        walk_mut::walk_property_key(self, it);
    }

    fn visit_object_property(&mut self, it: &mut oxc_ast::ast::ObjectProperty<'a>) {
        if self.mangle_quoted {
            // A shorthand string key cannot exist, so un-quoting the key is always safe.
            self.rewrite_string_key(&mut it.key, &mut it.computed);
            if it.computed {
                self.rename_key_expression_in_key(&mut it.key);
            }
        }
        walk_mut::walk_object_property(self, it);
    }

    fn visit_property_definition(&mut self, it: &mut oxc_ast::ast::PropertyDefinition<'a>) {
        if self.mangle_quoted {
            self.rewrite_string_key(&mut it.key, &mut it.computed);
            if it.computed {
                self.rename_key_expression_in_key(&mut it.key);
            }
        }
        walk_mut::walk_property_definition(self, it);
    }

    fn visit_accessor_property(&mut self, it: &mut oxc_ast::ast::AccessorProperty<'a>) {
        if self.mangle_quoted {
            self.rewrite_string_key(&mut it.key, &mut it.computed);
            if it.computed {
                self.rename_key_expression_in_key(&mut it.key);
            }
        }
        walk_mut::walk_accessor_property(self, it);
    }

    fn visit_method_definition(&mut self, it: &mut oxc_ast::ast::MethodDefinition<'a>) {
        if self.mangle_quoted {
            self.rewrite_string_key(&mut it.key, &mut it.computed);
            if it.computed {
                self.rename_key_expression_in_key(&mut it.key);
            }
        }
        walk_mut::walk_method_definition(self, it);
    }

    fn visit_binding_property(&mut self, it: &mut oxc_ast::ast::BindingProperty<'a>) {
        if self.mangle_quoted {
            self.rewrite_string_key(&mut it.key, &mut it.computed);
            if it.computed {
                self.rename_key_expression_in_key(&mut it.key);
            }
        }
        walk_mut::walk_binding_property(self, it);
    }

    fn visit_assignment_target_property_property(
        &mut self,
        it: &mut AssignmentTargetPropertyProperty<'a>,
    ) {
        if self.mangle_quoted {
            self.rewrite_string_key(&mut it.name, &mut it.computed);
            if it.computed {
                self.rename_key_expression_in_key(&mut it.name);
            }
        }
        walk_mut::walk_assignment_target_property_property(self, it);
    }

    fn visit_binary_expression(&mut self, it: &mut BinaryExpression<'a>) {
        // `'_a' in x` (and wrapped forms) keep the string literal but rename in place.
        if self.mangle_quoted && it.operator == BinaryOperator::In {
            self.rename_key_expression(&mut it.left);
        }
        walk_mut::walk_binary_expression(self, it);
    }
}

impl<'a> PropertyRewriter<'a, '_> {
    /// Rename string literals inside a wrapped computed key (`[(y,'_a')]`, `[y ? '_a' : z]`),
    /// which keep their structure (cannot be un-quoted).
    fn rename_key_expression_in_key(&self, key: &mut PropertyKey<'a>) {
        if let Some(expr) = key.as_expression_mut() {
            self.rename_key_expression(expr);
        }
    }
}

/// Driver that runs the two halves of property mangling around the compress/mangle passes.
///
/// Usage:
/// 1. [`PropertyMangler::new`] with the options.
/// 2. [`PropertyMangler::collect`] over the **original** program (before compress un-quotes keys).
/// 3. [`PropertyMangler::rewrite`] over the program **after** variable mangling.
pub struct PropertyMangler {
    opts: ManglePropertiesOptions,
    state: PropertyCollectState,
}

impl PropertyMangler {
    /// Create a new driver from the property-mangling options.
    pub fn new(opts: ManglePropertiesOptions) -> Self {
        Self { opts, state: PropertyCollectState::default() }
    }

    /// Run the read-only collect pass over the **original** (pre-compress) program.
    ///
    /// Call this before compress un-quotes any keys, so the reserved set captures the
    /// original quoting.
    pub fn collect(&mut self, program: &Program) {
        self.state = collect(&self.opts, program);
    }

    /// Assign final names and rewrite the program in place, returning the updated cache.
    ///
    /// Does nothing (returns the unchanged cache) when the collect pass bailed, or when no
    /// name ends up being mangled. Call this **after** variable mangling.
    pub fn rewrite<'a>(
        mut self,
        program: &mut Program<'a>,
        allocator: &'a Allocator,
    ) -> PropertyMangleCache {
        if self.state.bail {
            return self.opts.cache;
        }
        let map = assign(&self.state.candidates, &self.state.reserved, &mut self.opts.cache);
        if map.is_empty() {
            return self.opts.cache;
        }
        let mut rewriter = PropertyRewriter {
            map: &map,
            mangle_quoted: self.opts.mangle_quoted,
            ast: AstBuilder::new(allocator),
        };
        rewriter.visit_program(program);
        self.opts.cache
    }
}

#[cfg(test)]
#[expect(clippy::needless_lifetimes)]
mod tests {
    use super::*;
    use lazy_regex::Regex;

    fn opts(re: &str) -> ManglePropertiesOptions {
        ManglePropertiesOptions {
            mangle: Some(Regex::new(re).unwrap()),
            reserve: None,
            reserved: FxHashSet::default(),
            mangle_quoted: false,
            debug: false,
            cache: PropertyMangleCache::default(),
        }
    }

    fn opts_quoted(re: &str) -> ManglePropertiesOptions {
        ManglePropertiesOptions { mangle_quoted: true, ..opts(re) }
    }

    #[test]
    fn eligibility() {
        let o = opts("^_");
        assert!(eligible(&o, "_foo"));
        assert!(!eligible(&o, "foo")); // no regex match
        assert!(!eligible(&o, "__proto__")); // always reserved
        assert!(eligible(&o, "_then")); // matches ^_, not in denylist => eligible
        // protocol denylist wins even if it matches:
        let o2 = opts(".");
        assert!(!eligible(&o2, "then"));
        assert!(!eligible(&o2, "toJSON"));
    }

    #[test]
    fn assignment_is_deterministic_and_disjoint() {
        let cands: FxHashSet<CompactStr> =
            ["_a", "_b"].iter().map(|s| CompactStr::from(*s)).collect();
        let reserved = FxHashSet::default();
        let mut cache = PropertyMangleCache::default();
        let m1 = assign(&cands, &reserved, &mut cache);
        let m2 = assign(&cands, &reserved, &mut PropertyMangleCache::default());
        assert_eq!(m1, m2); // deterministic
        let names: FxHashSet<_> = m1.values().collect();
        assert_eq!(names.len(), m1.len()); // no two map to the same name
    }

    #[test]
    fn cache_reuse_and_reserved() {
        let cands: FxHashSet<CompactStr> = std::iter::once(CompactStr::from("_a")).collect();
        let mut cache = PropertyMangleCache::default();
        cache.map.insert("_a".into(), CacheValue::Name("Z".into()));
        let m = assign(&cands, &FxHashSet::default(), &mut cache);
        assert_eq!(m[&CompactStr::from("_a")].as_str(), "Z"); // honors cache
    }

    #[test]
    fn cache_collision_is_remangled_not_corrupted() {
        let cands: FxHashSet<CompactStr> = std::iter::once(CompactStr::from("_a")).collect();
        let reserved: FxHashSet<CompactStr> = std::iter::once(CompactStr::from("b")).collect();
        let mut cache = PropertyMangleCache::default();
        cache.map.insert("_a".into(), CacheValue::Name("b".into())); // collides with reserved `b`
        let m = assign(&cands, &reserved, &mut cache);
        // The cached name `b` collides with a reservation, so `_a` is regenerated a fresh
        // valid name instead of being kept/skipped. It must be mapped, and never to `b`.
        let out = &m[&CompactStr::from("_a")];
        assert_ne!(out.as_str(), "b"); // never collides with the reservation
        // The cache is rewritten to the fresh name so future builds stay consistent.
        assert_eq!(cache.map.get(&CompactStr::from("_a")), Some(&CacheValue::Name(out.clone())));
    }

    #[test]
    fn generation_collision_does_not_corrupt() {
        // `e` is cached to `Name("b")`, but `b` is reserved -> `e` must be regenerated.
        // `_x` has no cache entry -> it is generated fresh. Neither output may collide.
        let cands: FxHashSet<CompactStr> =
            ["_x", "e"].iter().map(|s| CompactStr::from(*s)).collect();
        let reserved: FxHashSet<CompactStr> = std::iter::once(CompactStr::from("b")).collect();
        let mut cache = PropertyMangleCache::default();
        cache.map.insert("e".into(), CacheValue::Name("b".into()));
        let m = assign(&cands, &reserved, &mut cache);
        let x_out = m.get(&CompactStr::from("_x"));
        let e_out = m.get(&CompactStr::from("e"));
        assert!(x_out.is_some());
        assert!(e_out.is_some());
        // The two outputs must be distinct: no two source props map to one name.
        assert_ne!(x_out, e_out);
        // `e`'s collided cache name was dropped; its output is a fresh name, not `b`.
        assert_ne!(e_out.unwrap().as_str(), "b");
        // `_x`'s output must not be the literal `e` left dangling, nor `b`.
        assert_ne!(x_out.unwrap().as_str(), "b");
    }

    #[test]
    fn cache_reuse_collision_does_not_corrupt() {
        // `_a` is cached to `_z`, but `_z` is itself a candidate cached to `q`.
        // `q` is reserved, so `_z` is regenerated; `_a`'s cached `_z` must NOT be reused
        // as `_z`'s own (now different) output, and the two outputs must be distinct.
        let cands: FxHashSet<CompactStr> =
            ["_a", "_z"].iter().map(|s| CompactStr::from(*s)).collect();
        let reserved: FxHashSet<CompactStr> = std::iter::once(CompactStr::from("q")).collect();
        let mut cache = PropertyMangleCache::default();
        cache.map.insert("_a".into(), CacheValue::Name("_z".into()));
        cache.map.insert("_z".into(), CacheValue::Name("q".into()));
        let m = assign(&cands, &reserved, &mut cache);
        let a_out = &m[&CompactStr::from("_a")];
        let z_out = &m[&CompactStr::from("_z")];
        assert_ne!(a_out, z_out); // the two outputs are distinct
    }

    fn collect_src<'a>(
        alloc: &'a oxc_allocator::Allocator,
        opts: &ManglePropertiesOptions,
        src: &str,
    ) -> PropertyCollectState {
        let st = oxc_span::SourceType::mjs();
        let ret = oxc_parser::Parser::new(alloc, src, st).parse();
        collect(opts, &ret.program)
    }

    #[test]
    fn collect_classifies() {
        let alloc = oxc_allocator::Allocator::default();
        let s = collect_src(&alloc, &opts("^_"), "a._x; b['_y']; ({ _z: 1, q: 2 });");
        assert!(s.candidates.contains("_x")); // unquoted member
        assert!(s.reserved.contains("_y")); // quoted member (mangle_quoted off)
        assert!(s.candidates.contains("_z")); // identifier key matching regex
        assert!(s.reserved.contains("q")); // identifier key not matching => reserved
        assert!(!s.bail);
    }

    #[test]
    fn collect_bails_on_with_and_eval() {
        let alloc = oxc_allocator::Allocator::default();
        assert!(collect_src(&alloc, &opts("^_"), "with (o) { a._x }").bail);
        assert!(collect_src(&alloc, &opts("^_"), "eval('a._x')").bail);
    }

    #[test]
    fn collect_reserves_in_operator_and_assignment_target() {
        let alloc = oxc_allocator::Allocator::default();
        let s = collect_src(&alloc, &opts("^_"), "'_x' in o; ({ _y } = o);");
        assert!(s.reserved.contains("_x")); // `in` LHS (mangle_quoted off)
        assert!(s.reserved.contains("_y")); // assignment-target shorthand
    }

    #[test]
    fn collect_quoted_candidates_when_mangle_quoted() {
        let alloc = oxc_allocator::Allocator::default();
        let s = collect_src(
            &alloc,
            &opts_quoted("^_"),
            "a['_x']; ({ '_y': 1 }); ({ ['_z']: 1 }); ({ [(q, '_w')]: 1 }); '_v' in o; \
             a[c ? '_u' : d];",
        );
        for name in ["_x", "_y", "_z", "_w", "_v", "_u"] {
            assert!(s.candidates.contains(name), "{name} should be a candidate");
            assert!(!s.reserved.contains(name), "{name} should not be reserved");
        }
        // The assignment-target shorthand is still reserved regardless of mangle_quoted.
        let s2 = collect_src(&alloc, &opts_quoted("^_"), "({ _k } = o);");
        assert!(s2.reserved.contains("_k"));
    }

    #[test]
    fn collect_does_not_mangle_call_argument_strings() {
        let alloc = oxc_allocator::Allocator::default();
        // The string is a function argument / inside a computed call, never a key.
        let s = collect_src(
            &alloc,
            &opts_quoted("^_"),
            "foo('_keep'); x[foo('_keep2')]; ({ [foo('_keep3')]: x });",
        );
        for name in ["_keep", "_keep2", "_keep3"] {
            assert!(!s.candidates.contains(name), "{name} must not be a candidate");
        }
    }
}
