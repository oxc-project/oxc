//! Property-name mangling engine.
//!
//! This module implements opt-in property-name mangling (`obj.longName` -> `obj.a`).
//! It is **off by default**: nothing is mangled unless the user supplies a `mangle`
//! regex via [`ManglePropertiesOptions`].
//!
//! This file contains the whole engine: the option/cache types, the eligibility check,
//! the name-assignment function, the read-only collect pass, the in-place rewrite pass,
//! and the [`PropertyMangler`] driver that runs the two halves around compress/mangle.

use oxc_allocator::Allocator;
use oxc_ast::{
    AstBuilder,
    ast::{
        AssignmentTargetPropertyIdentifier, AssignmentTargetPropertyProperty, BinaryExpression,
        BinaryOperator, CallExpression, ComputedMemberExpression, Expression, JSXAttributeName,
        NewExpression, Program, PropertyKey, StaticMemberExpression, WithStatement,
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
    /// Whether to mangle quoted keys. v1: always `false`.
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
fn assign(
    candidates: &FxHashSet<CompactStr>,
    reserved: &FxHashSet<CompactStr>,
    cache: &mut PropertyMangleCache,
) -> FxHashMap<CompactStr, CompactStr> {
    // Deterministic order so a shared cache is reproducible.
    let mut names: Vec<&CompactStr> = candidates.difference(reserved).collect();
    names.sort_unstable();
    // Seed `seeded` with existing cache targets so freshly-generated names never alias
    // a name that a (possibly later) cached candidate will reuse. This set is used only
    // to constrain newly generated names; reusing a candidate's own cached name is fine.
    let seeded: FxHashSet<CompactStr> = cache
        .map
        .values()
        .filter_map(|v| match v {
            CacheValue::Name(n) => Some(n.clone()),
            CacheValue::Reserved => None,
        })
        .collect();
    // Names actually claimed by an output during this build.
    let mut assigned: FxHashSet<CompactStr> = FxHashSet::default();
    let mut counter: u32 = 0;
    let mut map = FxHashMap::default();
    for name in names {
        match cache.map.get(name) {
            Some(CacheValue::Reserved) => {}
            Some(CacheValue::Name(n)) => {
                // Cache validation: never reuse a name that collides this build.
                if reserved.contains(n.as_str()) || assigned.contains(n) || is_always_reserved(n) {
                    continue; // safe-skip
                }
                map.insert(name.clone(), n.clone());
                assigned.insert(n.clone());
            }
            None => {
                let n = loop {
                    let c = CompactStr::from(base54(counter).as_str());
                    counter += 1;
                    if !seeded.contains(&c)
                        && !assigned.contains(&c)
                        && !reserved.contains(&c)
                        && !is_always_reserved(&c)
                    {
                        break c;
                    }
                };
                map.insert(name.clone(), n.clone());
                assigned.insert(n.clone());
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

    /// A quoted/computed/never-mangle occurrence: reserve the name program-wide.
    fn reserve(&mut self, name: &str) {
        self.state.reserved.insert(CompactStr::from(name));
    }

    /// Classify a [`PropertyKey`] (object/binding/class member key, or assignment-target name).
    fn classify_property_key(&mut self, key: &PropertyKey<'a>) {
        match key {
            PropertyKey::StaticIdentifier(ident) => self.candidate(ident.name.as_str()),
            PropertyKey::StringLiteral(lit) => self.reserve(lit.value.as_str()),
            PropertyKey::NumericLiteral(lit) => self.reserve(&lit.value.to_string()),
            // Private identifiers are never object properties; skip.
            PropertyKey::PrivateIdentifier(_) => {}
            // Computed key: reserve only if it is a string literal, otherwise skip.
            key => {
                if let PropertyKey::StringLiteral(lit) = key {
                    self.reserve(lit.value.as_str());
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
        if let Expression::StringLiteral(lit) = &it.expression {
            self.reserve(lit.value.as_str());
        }
        walk_computed_member_expression(self, it);
    }

    fn visit_property_key(&mut self, it: &PropertyKey<'a>) {
        self.classify_property_key(it);
        walk_property_key(self, it);
    }

    fn visit_assignment_target_property_identifier(
        &mut self,
        it: &AssignmentTargetPropertyIdentifier<'a>,
    ) {
        // Decision #2: the shorthand `({ foo } = obj)` is reserved in v1, never mangled.
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
        // `'foo' in obj` reserves `foo`.
        if it.operator == BinaryOperator::In
            && let Expression::StringLiteral(lit) = &it.left
        {
            self.reserve(lit.value.as_str());
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
/// Renamed positions are unquoted member properties (`StaticMemberExpression.property`) and
/// `StaticIdentifier` property keys (object/binding/class member keys, and the key of an
/// `AssignmentTargetPropertyProperty`, which is reached through the `PropertyKey` walk).
///
/// `AssignmentTargetPropertyIdentifier` shorthands are **not** renamed: their names were
/// added to the reserved set during collect, so they never appear in `map`.
struct PropertyRewriter<'a, 'm> {
    /// Old name -> new (mangled) name.
    map: &'m FxHashMap<CompactStr, CompactStr>,
    /// Allocates the new name strings into the program's arena.
    ast: AstBuilder<'a>,
}

impl<'a> VisitMut<'a> for PropertyRewriter<'a, '_> {
    fn visit_static_member_expression(&mut self, it: &mut StaticMemberExpression<'a>) {
        if let Some(new_name) = self.map.get(it.property.name.as_str()) {
            it.property.name = self.ast.ident(new_name.as_str());
        }
        walk_mut::walk_static_member_expression(self, it);
    }

    fn visit_property_key(&mut self, it: &mut PropertyKey<'a>) {
        if let PropertyKey::StaticIdentifier(ident) = it
            && let Some(new_name) = self.map.get(ident.name.as_str())
        {
            ident.name = self.ast.ident(new_name.as_str());
        }
        walk_mut::walk_property_key(self, it);
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
        let mut rewriter = PropertyRewriter { map: &map, ast: AstBuilder::new(allocator) };
        rewriter.visit_program(program);
        self.opts.cache
    }
}

#[cfg(test)]
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
    fn cache_collision_is_skipped_not_corrupted() {
        let cands: FxHashSet<CompactStr> = std::iter::once(CompactStr::from("_a")).collect();
        let reserved: FxHashSet<CompactStr> = std::iter::once(CompactStr::from("b")).collect();
        let mut cache = PropertyMangleCache::default();
        cache.map.insert("_a".into(), CacheValue::Name("b".into())); // collides with reserved `b`
        let m = assign(&cands, &reserved, &mut cache);
        assert!(!m.contains_key(&CompactStr::from("_a"))); // skipped, not mapped to `b`
    }

    fn collect_src(src: &str, re: &str) -> PropertyCollectState {
        let alloc = oxc_allocator::Allocator::default();
        let st = oxc_span::SourceType::mjs();
        let ret = oxc_parser::Parser::new(&alloc, src, st).parse();
        collect(&opts(re), &ret.program)
    }

    #[test]
    fn collect_classifies() {
        let s = collect_src("a._x; b['_y']; ({ _z: 1, q: 2 });", "^_");
        assert!(s.candidates.contains("_x")); // unquoted member
        assert!(s.reserved.contains("_y")); // quoted member
        assert!(s.candidates.contains("_z")); // identifier key matching regex
        assert!(s.reserved.contains("q")); // identifier key not matching => reserved
        assert!(!s.bail);
    }

    #[test]
    fn collect_bails_on_with_and_eval() {
        assert!(collect_src("with (o) { a._x }", "^_").bail);
        assert!(collect_src("eval('a._x')", "^_").bail);
    }

    #[test]
    fn collect_reserves_in_operator_and_assignment_target() {
        let s = collect_src("'_x' in o; ({ _y } = o);", "^_");
        assert!(s.reserved.contains("_x")); // `in` LHS
        assert!(s.reserved.contains("_y")); // assignment-target shorthand
    }
}
