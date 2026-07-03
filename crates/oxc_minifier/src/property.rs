//! Property-name mangling engine.
//!
//! This module implements opt-in property-name mangling (`obj.longName` -> `obj.a`).
//! It is **off by default**: nothing is mangled unless the user supplies a `mangle`
//! regex via [`ManglePropertiesOptions`].
//!
//! This file contains the whole engine: the option type, the eligibility check,
//! the name-assignment function, the read-only collect pass, the in-place rewrite pass,
//! and the [`PropertyMangler`] driver that runs the two halves around compress/mangle.

use oxc_allocator::{Allocator, TakeIn};
use oxc_ast::{
    ast::{
        AssignmentTargetPropertyIdentifier, AssignmentTargetPropertyProperty, BinaryExpression,
        BinaryOperator, CallExpression, Comment, CommentPosition, ComputedMemberExpression,
        Expression, IdentifierName, JSXAttributeName, JSXMemberExpression, NewExpression, Program,
        PropertyKey, StaticMemberExpression, StringLiteral, TemplateLiteral, WithStatement,
    },
    builder::AstBuilder,
};
use oxc_ast_visit::{
    Visit, VisitMut,
    walk::{
        walk_assignment_target_property_identifier, walk_assignment_target_property_property,
        walk_binary_expression, walk_call_expression, walk_computed_member_expression,
        walk_jsx_attribute_name, walk_jsx_member_expression, walk_new_expression,
        walk_property_key, walk_static_member_expression, walk_template_literal,
        walk_with_statement,
    },
    walk_mut,
};
use oxc_ecmascript::StringToNumber;
use oxc_mangler::base54;
use oxc_span::Span;
use oxc_str::{CompactStr, Ident, Str};
use oxc_syntax::number::ToJsString;
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
}

/// Whether `name` is always reserved, regardless of the user's regex.
fn is_always_reserved(name: &str) -> bool {
    matches!(name, "__proto__" | "constructor" | "prototype") || PROTOCOL_DENYLIST.contains(&name)
}

/// The canonical JS string form of a number used as a property key.
///
/// `ToPropertyKey` coerces a numeric key/index with `ToString`, so `{ 1e21: 1 }`, `obj[1e21]`
/// and `obj['1e+21']` all address the property named `"1e+21"`. Rust's `f64::to_string`
/// (`Display`) is NOT JS `ToString` — it spells `1e21` as `1000000000000000000000` — so use
/// the ECMAScript number-to-string algorithm.
fn numeric_key_string(value: f64) -> String {
    if value == 0.0 { "0".to_string() } else { value.to_js_string() }
}

/// Whether `name` is the canonical JS string form of some number, i.e. `ToString(ToNumber(name))
/// == name`. Such a string aliases a numeric key/index (`obj['0']` <-> `obj[0]`, `obj['1e+21']`
/// <-> `obj[1e21]`), so it must never be mangled — a numeric-literal access elsewhere would no
/// longer resolve to the renamed property. Mirrors esbuild, which reserves every numeric-looking
/// property string.
fn is_canonical_numeric_string(name: &str) -> bool {
    let value = name.string_to_number();
    if value.is_nan() {
        // Only the literal "NaN" round-trips to the string "NaN"; anything else that parses to
        // NaN (e.g. "abc") is not a numeric spelling.
        return name == "NaN";
    }
    numeric_key_string(value) == name
}

/// Whether `comment`'s text is exactly a `/* @__KEY__ */` or `/* #__KEY__ */` annotation.
///
/// The leading `@` or `#` is required: a bare `/* __KEY__ */` does NOT count (esbuild
/// parity — see `TestManglePropsKeyComment`). Surrounding whitespace is ignored.
fn is_key_annotation(comment: &Comment, source_text: &str) -> bool {
    let text = comment.content_span().source_text(source_text).trim();
    matches!(text.strip_prefix(['@', '#']), Some("__KEY__"))
}

/// Build the set of start offsets of string/template literals that are immediately
/// preceded by a `/* @__KEY__ */` / `/* #__KEY__ */` comment.
///
/// Each leading comment's `attached_to` is the start offset of the token it precedes,
/// which for an annotated literal is exactly that literal's `span.start`. So the set
/// of `attached_to` offsets of all key-annotation comments is the set of annotated
/// literal spans.
///
/// Only **leading** comments are considered: trailing comment attachment is never computed,
/// so a trailing `/* @__KEY__ */` has `attached_to == 0` and would otherwise falsely annotate
/// whatever literal begins the program.
fn key_annotated_spans(program: &Program) -> FxHashSet<u32> {
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

/// Whether `name` is eligible for mangling under `opts`.
fn eligible(opts: &ManglePropertiesOptions, name: &str) -> bool {
    opts.mangle.as_ref().is_some_and(|re| re.is_match(name))
        && !opts.reserve.as_ref().is_some_and(|re| re.is_match(name))
        && !opts.reserved.contains(name)
        && !is_always_reserved(name)
        && !is_canonical_numeric_string(name)
}

/// Assign final mangled names.
///
/// `classes` is the collect pass's classification of every distinct property name: the
/// `Candidate` entries are mangled. Generated names must be disjoint from EVERY name in
/// `classes` (`Candidate` originals included, not just the `Reserved` entries): the rename
/// passes re-match positions by NAME (un-quoted keys, annotated literals, keys that compress
/// un-quotes between collect and rewrite), so a generated name equal to any source property
/// name could be picked up by a later value-based lookup and renamed a second time. Keeping
/// the generated names out of `classes` makes "apply at most once" structural. Generated
/// names also avoid the always-reserved set and the user's explicit `reserved` names; the
/// `reserve` REGEX is deliberately NOT applied to generated names (esbuild parity: the regex
/// only filters source-seen names). Returns the old -> new map.
///
/// The iteration order is deterministic (sorted) so the same input always produces the same
/// names. Names come from a monotonic `base54` counter, so the outputs are pairwise disjoint
/// by construction; the counter skips any name that collides with the sets above.
fn assign(
    opts: &ManglePropertiesOptions,
    classes: &FxHashMap<CompactStr, Class>,
) -> FxHashMap<CompactStr, CompactStr> {
    // `Reserved` already won over `Candidate` during collect, so the `Candidate` keys are exactly
    // the names to mangle (no candidate/reserved set difference needed). `CompactStr: Borrow<str>`
    // lets the base54 `&str` probe `classes` and `opts.reserved` directly.

    // Deterministic order so the same input always produces the same names.
    let mut names: Vec<&CompactStr> = classes
        .iter()
        .filter_map(|(name, class)| (*class == Class::Candidate).then_some(name))
        .collect();
    names.sort_unstable();

    let mut counter: u32 = 0;
    let mut map = FxHashMap::default();

    for name in names {
        // The counter only ever advances, so successive `base54` names are distinct: the outputs
        // are pairwise disjoint without tracking assigned names. Skip a generated name that
        // collides with ANY property name seen in the program, an always-reserved name, or an
        // explicitly reserved name (see the doc comment for why `Candidate` originals count).
        let n = loop {
            let candidate = base54(counter);
            counter = counter.checked_add(1).expect("property name space exhausted");
            let candidate = candidate.as_str();
            // Test the `&str` view against the sets, allocating a `CompactStr` only once a name
            // survives (discarded names on collision cost nothing).
            if !classes.contains_key(candidate)
                && !is_always_reserved(candidate)
                && !opts.reserved.contains(candidate)
            {
                break CompactStr::from(candidate);
            }
        };
        map.insert(name.clone(), n);
    }
    map
}

/// How a distinct property name was classified during the collect pass.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Class {
    /// Eligible, and only ever seen in a mangle-able (unquoted) position: mangle it.
    Candidate,
    /// Must never be mangled — seen in a quoted/computed key, the LHS of `'x' in obj`,
    /// a JSX attribute, or an assignment-target shorthand (or it failed eligibility).
    /// `Reserved` always wins: a name seen in any reserve-position stays `Reserved`
    /// even if it was also seen unquoted.
    Reserved,
}

/// What made the whole program bail out of property mangling.
///
/// Each of these can reference property names dynamically (by string), so mangling any
/// property becomes unsafe program-wide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertyMangleBailKind {
    /// A `with (obj) { ... }` statement.
    With,
    /// A direct `eval(...)` call.
    DirectEval,
    /// The `Function` constructor (`Function(...)` or `new Function(...)`).
    FunctionConstructor,
}

/// A whole-program bail: property mangling was disabled and no name was renamed.
///
/// Carries the node that triggered the bail so callers can surface an actionable
/// diagnostic. The **first** trigger encountered during collect wins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PropertyMangleBail {
    /// Which construct forced the bail.
    pub kind: PropertyMangleBailKind,
    /// Span of the triggering node.
    pub span: Span,
}

/// The result of the read-only collect pass over the original (pre-compress) program.
///
/// `names` classifies every distinct property name seen exactly once (see [`Class`]).
/// Folding the candidate/reserved split into one map keeps the per-occurrence hot path
/// to a single hash lookup (a property name repeats many times in a real bundle), instead
/// of probing two separate sets. `bail` is `Some` when the program contains `with` or a direct
/// `eval` / `Function` constructor, which makes property mangling unsafe for the whole program.
#[derive(Default)]
pub struct PropertyCollectState {
    /// Every distinct property name, classified once. `Reserved` wins over `Candidate`.
    pub names: FxHashMap<CompactStr, Class>,
    /// `with` or direct `eval` / `Function` present anywhere => disable mangling. Holds the
    /// first trigger (kind + span) so the reason can be reported; `None` means no bail.
    pub bail: Option<PropertyMangleBail>,
}

#[cfg(test)]
impl PropertyCollectState {
    /// Whether `name` was classified as a mangle candidate.
    fn is_candidate(&self, name: &str) -> bool {
        matches!(self.names.get(name), Some(Class::Candidate))
    }

    /// Whether `name` was classified as reserved (never mangled).
    fn is_reserved(&self, name: &str) -> bool {
        matches!(self.names.get(name), Some(Class::Reserved))
    }
}

/// Read-only visitor that classifies every property-bearing position in the program.
struct PropertyCollector<'o> {
    opts: &'o ManglePropertiesOptions,
    state: PropertyCollectState,
    /// Start offsets of string/template literals annotated with `/* @__KEY__ */` / `/* #__KEY__ */`.
    key_annotated: &'o FxHashSet<u32>,
}

impl<'o> PropertyCollector<'o> {
    fn new(opts: &'o ManglePropertiesOptions, key_annotated: &'o FxHashSet<u32>) -> Self {
        Self { opts, state: PropertyCollectState::default(), key_annotated }
    }

    /// An unquoted occurrence: mangle it if eligible, otherwise it is reserved program-wide.
    fn candidate(&mut self, name: &str) {
        // A property name repeats many times in a real bundle. Once it has been classified the
        // decision is fixed, so a single `contains_key` short-circuits before re-running
        // `eligible` (which evaluates the mangle/reserve REGEXES) and before re-allocating a
        // `CompactStr`. This turns per-occurrence regex work into per-distinct-name work.
        // `CompactStr: Borrow<str>`, so the lookup takes the `&str` directly with no allocation.
        if self.state.names.contains_key(name) {
            return;
        }
        let class = if eligible(self.opts, name) { Class::Candidate } else { Class::Reserved };
        self.state.names.insert(CompactStr::from(name), class);
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
            self.reserve(name);
        }
    }

    /// Record a whole-program bail. The first trigger wins, so a later hazard never
    /// overwrites the reported reason/span.
    fn bail(&mut self, kind: PropertyMangleBailKind, span: Span) {
        if self.state.bail.is_none() {
            self.state.bail = Some(PropertyMangleBail { kind, span });
        }
    }

    /// Reserve a name program-wide, regardless of `mangle_quoted`.
    fn reserve(&mut self, name: &str) {
        // `Reserved` wins: overwrite an existing `Candidate` (downgrade) or `Reserved` (no-op)
        // with one `get_mut`, allocating a `CompactStr` only for a name not yet seen.
        if let Some(class) = self.state.names.get_mut(name) {
            *class = Class::Reserved;
        } else {
            self.state.names.insert(CompactStr::from(name), Class::Reserved);
        }
    }

    /// Classify a quoted/computed string occurrence in a key/index position, recursing
    /// through the wrapped forms esbuild handles: conditionals (`y ? '_a' : '_b'`) and
    /// comma/sequence expressions (`(y, '_a')`). Only the statically-reachable key
    /// strings are classified; non-literal sub-expressions are ignored (they are not
    /// statically-known keys).
    fn classify_key_expression(&mut self, expr: &Expression<'_>) {
        match expr.get_inner_expression() {
            // A key-annotated string is a candidate (handled by `visit_string_literal`);
            // the annotation overrides the default quoted-reserve, so skip it here.
            Expression::StringLiteral(lit) if self.key_annotated.contains(&lit.span.start) => {}
            Expression::StringLiteral(lit) => self.quoted(lit.value.as_str()),
            // A no-substitution template (`` `_a` ``) in a key/index position is runtime-
            // equivalent to the string `'_a'`, so classify it the same way. An annotated
            // template is a candidate (handled by `visit_template_literal`); skip it here.
            Expression::TemplateLiteral(tmpl)
                if tmpl.expressions.is_empty() && self.key_annotated.contains(&tmpl.span.start) => {
            }
            Expression::TemplateLiteral(tmpl) if tmpl.expressions.is_empty() => {
                if let [quasi] = tmpl.quasis.as_slice()
                    && let Some(cooked) = quasi.value.cooked
                {
                    self.quoted(cooked.as_str());
                }
            }
            Expression::ConditionalExpression(cond) => {
                self.classify_key_expression(&cond.consequent);
                self.classify_key_expression(&cond.alternate);
            }
            Expression::SequenceExpression(seq) => {
                if let Some(last) = seq.expressions.last() {
                    self.classify_key_expression(last);
                }
            }
            // A numeric computed index (`x[0]`, `x[1e21]`) addresses the property whose name is
            // the number's JS string form; reserve that spelling so a quoted `x['0']` / `x['1e+21']`
            // sibling stays aligned with it.
            Expression::NumericLiteral(lit) => self.reserve(&numeric_key_string(lit.value)),
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
    fn classify_property_key(&mut self, key: &PropertyKey<'_>) {
        match key {
            PropertyKey::StaticIdentifier(ident) => self.candidate(ident.name.as_str()),
            PropertyKey::NumericLiteral(lit) => self.reserve(&numeric_key_string(lit.value)),
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

impl<'a> Visit<'a> for PropertyCollector<'_> {
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

    fn visit_jsx_member_expression(&mut self, it: &JSXMemberExpression<'a>) {
        // `<ns._comp/>` uses `_comp` as a member of the element name; reserve it (esbuild does the
        // same) so a plain-JS `ns._comp` access is not mangled out of alignment. JSX members are
        // never renamed, only reserved.
        self.reserve(it.property.name.as_str());
        walk_jsx_member_expression(self, it);
    }

    fn visit_binary_expression(&mut self, it: &BinaryExpression<'a>) {
        // `'foo' in obj`, plus the wrapped forms `(y ? '_a' : z) in obj` / `(y, '_a') in obj`.
        if it.operator == BinaryOperator::In {
            self.classify_key_expression(&it.left);
        }
        walk_binary_expression(self, it);
    }

    fn visit_with_statement(&mut self, it: &WithStatement<'a>) {
        self.bail(PropertyMangleBailKind::With, it.span);
        walk_with_statement(self, it);
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        // Match through parens (and TS wrappers): per ECMA-262 `(eval)(x)` is still a DIRECT
        // eval (parentheses preserve the reference), and the AST keeps `ParenthesizedExpression`
        // nodes (`preserve_parens`). A sequence `(0, eval)(x)` stays indirect:
        // `get_inner_expression` does not unwrap it.
        if let Expression::Identifier(ident) = it.callee.get_inner_expression() {
            match ident.name.as_str() {
                "eval" => self.bail(PropertyMangleBailKind::DirectEval, it.span),
                "Function" => self.bail(PropertyMangleBailKind::FunctionConstructor, it.span),
                _ => {}
            }
        }
        walk_call_expression(self, it);
    }

    fn visit_new_expression(&mut self, it: &NewExpression<'a>) {
        // `new (Function)(...)` still reaches the `Function` constructor; see through parens
        // as in `visit_call_expression`.
        if let Expression::Identifier(ident) = it.callee.get_inner_expression()
            && ident.name.as_str() == "Function"
        {
            self.bail(PropertyMangleBailKind::FunctionConstructor, it.span);
        }
        walk_new_expression(self, it);
    }

    fn visit_string_literal(&mut self, it: &StringLiteral<'a>) {
        // A string literal directly preceded by `/* @__KEY__ */` / `/* #__KEY__ */`
        // is treated as a property name even outside a key position (e.g. a call
        // argument). The eligibility/regex still gates it.
        if self.key_annotated.contains(&it.span.start) {
            self.candidate(it.value.as_str());
        }
    }

    fn visit_template_literal(&mut self, it: &TemplateLiteral<'a>) {
        // A no-substitution template (`` `_foo` ``) preceded by a key annotation is
        // also treated as a property name. Templates with interpolations are not
        // themselves property names; their interpolated string operands are visited
        // recursively (each may carry its own annotation), so recurse.
        if it.expressions.is_empty()
            && self.key_annotated.contains(&it.span.start)
            && let [quasi] = it.quasis.as_slice()
            && let Some(cooked) = quasi.value.cooked
        {
            self.candidate(cooked.as_str());
        }
        walk_template_literal(self, it);
    }
}

/// Walk the **original** (pre-compress) program and classify every property occurrence.
///
/// Returns the candidate/reserved sets and the whole-program `bail` flag.
fn collect(
    opts: &ManglePropertiesOptions,
    key_annotated: &FxHashSet<u32>,
    program: &Program,
) -> PropertyCollectState {
    let mut collector = PropertyCollector::new(opts, key_annotated);
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
    /// Start offsets of string/template literals annotated with a key comment.
    key_annotated: &'m FxHashSet<u32>,
    /// When `true`, ONLY key-annotated string/template literals are renamed; member and key
    /// positions are left untouched. Used by the pre-compress `rename_annotated_literals` pass.
    rename_annotated_only: bool,
    /// Allocates the new name strings into the program's arena.
    ast: AstBuilder<'a>,
}

impl<'a> PropertyRewriter<'a, '_> {
    /// Rename an in-place string literal (used for the `in`-LHS and wrapped computed keys).
    fn rename_string_literal(&self, lit: &mut StringLiteral<'a>) {
        if let Some(new_name) = self.map.get(lit.value.as_str()) {
            lit.value = Str::from_str_in(new_name.as_str(), &self.ast);
        }
    }

    /// Rename an in-place no-substitution template literal (the template analogue of
    /// [`Self::rename_string_literal`], for the `in`-LHS and wrapped computed keys).
    fn rename_template_literal(&self, tmpl: &mut TemplateLiteral<'a>) {
        if tmpl.expressions.is_empty()
            && let [quasi] = tmpl.quasis.as_mut_slice()
            && let Some(cooked) = quasi.value.cooked
            && let Some(new_name) = self.map.get(cooked.as_str())
        {
            let new_str = Str::from_str_in(new_name.as_str(), &self.ast);
            quasi.value.cooked = Some(new_str);
            quasi.value.raw = new_str;
        }
    }

    /// Rename the string literals reachable through the wrapped key forms (conditional /
    /// sequence), in place. Used for the `in`-LHS, member-index, and computed-key wrappers
    /// that cannot be un-quoted.
    fn rename_key_expression(&self, expr: &mut Expression<'a>) {
        // Unwrap the same wrappers (parens + TS `as`/`satisfies`/`!`/...) that the collect
        // side strips via `get_inner_expression`, so the two passes stay in lockstep.
        match expr.get_inner_expression_mut() {
            Expression::StringLiteral(lit) => self.rename_string_literal(lit),
            Expression::TemplateLiteral(tmpl) => self.rename_template_literal(tmpl),
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
                let ident = IdentifierName::boxed(
                    lit.span,
                    Ident::from_str_in(new_name.as_str(), &self.ast),
                    &self.ast,
                );
                *key = PropertyKey::StaticIdentifier(ident);
                *computed = false;
            }
            true
        } else {
            false
        }
    }

    /// Rewrite a member-key position (object/class/binding member key, assignment target):
    /// un-quote a direct string key, then rename string literals inside a wrapped computed key.
    /// Shared by the six `visit_*` methods, which differ only in the key field they pass.
    fn rewrite_key_position(&self, key: &mut PropertyKey<'a>, computed: &mut bool) {
        self.rewrite_string_key(key, computed);
        if *computed {
            self.rename_key_expression_in_key(key);
        }
    }
}

impl<'a> VisitMut<'a> for PropertyRewriter<'a, '_> {
    fn visit_static_member_expression(&mut self, it: &mut StaticMemberExpression<'a>) {
        if !self.rename_annotated_only
            && let Some(new_name) = self.map.get(it.property.name.as_str())
        {
            it.property.name = Ident::from_str_in(new_name.as_str(), &self.ast);
        }
        walk_mut::walk_static_member_expression(self, it);
    }

    fn visit_expression(&mut self, it: &mut Expression<'a>) {
        // Un-quote a direct string index `x['_a']` -> `x.<new>`, preserving optional chaining.
        // Wrapped index forms (`x[y ? '_a' : z]`) are renamed in place via the
        // computed-member visitor below.
        if !self.rename_annotated_only
            && self.mangle_quoted
            && let Expression::ComputedMemberExpression(member) = it
            && let Expression::StringLiteral(lit) = &member.expression
            && let Some(new_name) = self.map.get(lit.value.as_str())
        {
            let property = IdentifierName::new(
                lit.span,
                Ident::from_str_in(new_name.as_str(), &self.ast),
                &self.ast,
            );
            let new_member = StaticMemberExpression::boxed(
                member.span,
                member.object.take_in(&self.ast),
                property,
                member.optional,
                &self.ast,
            );
            *it = Expression::StaticMemberExpression(new_member);
            // The fresh property identifier holds a generated name, which is never a key in
            // `map` (`assign` keeps generated names disjoint from every source property name),
            // so a re-lookup could not rename it again anyway. Only the object subtree has
            // work left; visit it directly instead of re-dispatching the walk on the
            // replacement.
            if let Expression::StaticMemberExpression(new_member) = it {
                self.visit_expression(&mut new_member.object);
            }
            return;
        }
        walk_mut::walk_expression(self, it);
    }

    fn visit_computed_member_expression(&mut self, it: &mut ComputedMemberExpression<'a>) {
        // Wrapped index forms that cannot be un-quoted: rename in place.
        if !self.rename_annotated_only && self.mangle_quoted {
            self.rename_key_expression(&mut it.expression);
        }
        walk_mut::walk_computed_member_expression(self, it);
    }

    fn visit_property_key(&mut self, it: &mut PropertyKey<'a>) {
        if !self.rename_annotated_only
            && let PropertyKey::StaticIdentifier(ident) = it
            && let Some(new_name) = self.map.get(ident.name.as_str())
        {
            ident.name = Ident::from_str_in(new_name.as_str(), &self.ast);
        }
        walk_mut::walk_property_key(self, it);
    }

    fn visit_object_property(&mut self, it: &mut oxc_ast::ast::ObjectProperty<'a>) {
        // Walk children FIRST (which renames a real `StaticIdentifier` key and the value), THEN
        // un-quote the string key: the fresh identifier holds a generated name — never a key in
        // `map` — so nothing is left for the walk to do on it. A shorthand string key cannot
        // exist, so un-quoting the key is always safe.
        walk_mut::walk_object_property(self, it);
        if !self.rename_annotated_only && self.mangle_quoted {
            self.rewrite_key_position(&mut it.key, &mut it.computed);
        }
    }

    fn visit_property_definition(&mut self, it: &mut oxc_ast::ast::PropertyDefinition<'a>) {
        walk_mut::walk_property_definition(self, it);
        if !self.rename_annotated_only && self.mangle_quoted {
            self.rewrite_key_position(&mut it.key, &mut it.computed);
        }
    }

    fn visit_accessor_property(&mut self, it: &mut oxc_ast::ast::AccessorProperty<'a>) {
        walk_mut::walk_accessor_property(self, it);
        if !self.rename_annotated_only && self.mangle_quoted {
            self.rewrite_key_position(&mut it.key, &mut it.computed);
        }
    }

    fn visit_method_definition(&mut self, it: &mut oxc_ast::ast::MethodDefinition<'a>) {
        walk_mut::walk_method_definition(self, it);
        if !self.rename_annotated_only && self.mangle_quoted {
            self.rewrite_key_position(&mut it.key, &mut it.computed);
        }
    }

    fn visit_binding_property(&mut self, it: &mut oxc_ast::ast::BindingProperty<'a>) {
        walk_mut::walk_binding_property(self, it);
        if !self.rename_annotated_only && self.mangle_quoted {
            self.rewrite_key_position(&mut it.key, &mut it.computed);
        }
    }

    fn visit_assignment_target_property_property(
        &mut self,
        it: &mut AssignmentTargetPropertyProperty<'a>,
    ) {
        walk_mut::walk_assignment_target_property_property(self, it);
        if !self.rename_annotated_only && self.mangle_quoted {
            self.rewrite_key_position(&mut it.name, &mut it.computed);
        }
    }

    fn visit_binary_expression(&mut self, it: &mut BinaryExpression<'a>) {
        // `'_a' in x` (and wrapped forms) keep the string literal but rename in place.
        if !self.rename_annotated_only && self.mangle_quoted && it.operator == BinaryOperator::In {
            self.rename_key_expression(&mut it.left);
        }
        walk_mut::walk_binary_expression(self, it);
    }

    fn visit_string_literal(&mut self, it: &mut StringLiteral<'a>) {
        // A key-annotated string literal is renamed in place (it stays a string),
        // regardless of `mangle_quoted` and regardless of its syntactic position
        // (call argument, `in`-LHS, template interpolation, computed key, ...).
        if self.key_annotated.contains(&it.span.start)
            && let Some(new_name) = self.map.get(it.value.as_str())
        {
            it.value = Str::from_str_in(new_name.as_str(), &self.ast);
            // Drop the now-stale raw text so codegen re-serializes from `value`.
            it.raw = None;
        }
    }

    fn visit_template_literal(&mut self, it: &mut TemplateLiteral<'a>) {
        // A key-annotated no-substitution template is renamed in place (stays a template).
        // Templates with interpolations are not themselves property names; their string
        // operands are renamed by `visit_string_literal` through the recursion below.
        if it.expressions.is_empty()
            && self.key_annotated.contains(&it.span.start)
            && let [quasi] = it.quasis.as_mut_slice()
            && let Some(cooked) = quasi.value.cooked
            && let Some(new_name) = self.map.get(cooked.as_str())
        {
            let new_str = Str::from_str_in(new_name.as_str(), &self.ast);
            quasi.value.cooked = Some(new_str);
            quasi.value.raw = new_str;
        }
        walk_mut::walk_template_literal(self, it);
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
/// 3. [`PropertyMangler::rename_annotated_literals`] over the **original** program (before compress,
///    so key-annotated strings inside template interpolations are renamed before the compressor
///    folds them into the surrounding quasi).
/// 4. [`PropertyMangler::rewrite`] over the program **after** variable mangling.
pub struct PropertyMangler {
    opts: ManglePropertiesOptions,
    state: PropertyCollectState,
    /// Start offsets of string/template literals annotated with `/* @__KEY__ */` /
    /// `/* #__KEY__ */`, computed during `collect` and reused during the rename passes.
    key_annotated: FxHashSet<u32>,
    /// Final old -> new name map, assigned once in `rename_annotated_literals` and reused by
    /// `rewrite`. `None` until names are assigned.
    map: Option<FxHashMap<CompactStr, CompactStr>>,
}

impl PropertyMangler {
    /// Create a new driver from the property-mangling options.
    pub fn new(opts: ManglePropertiesOptions) -> Self {
        Self {
            opts,
            state: PropertyCollectState::default(),
            key_annotated: FxHashSet::default(),
            map: None,
        }
    }

    /// The whole-program bail recorded during [`Self::collect`], if any.
    ///
    /// `Some` means property mangling was disabled for the entire program (a `with`
    /// statement, a direct `eval`, or the `Function` constructor was found) and no name
    /// was renamed; the [`PropertyMangleBail`] carries the reason and the triggering span.
    /// Read this after `collect` to surface a diagnostic.
    pub fn bail(&self) -> Option<PropertyMangleBail> {
        self.state.bail
    }

    /// Run the read-only collect pass over the **original** (pre-compress) program.
    ///
    /// Call this before compress un-quotes any keys, so the reserved set captures the
    /// original quoting.
    pub fn collect(&mut self, program: &Program) {
        self.key_annotated = key_annotated_spans(program);
        self.state = collect(&self.opts, &self.key_annotated, program);
    }

    /// Assign the final names and rename key-annotated string/template literals **in place**,
    /// over the **original** program (before compress).
    ///
    /// Annotated strings inside template interpolations (`` `${/* @__KEY__ */ '_x'}` ``) must be
    /// renamed here, because the compressor folds constant interpolations into the surrounding
    /// quasi — after which the annotated string node no longer exists to rewrite. Renaming an
    /// annotated literal's value in place is position-independent and safe to do this early.
    ///
    /// Does nothing when collect bailed or nothing is mangled. After renaming, the annotated
    /// spans are dropped from `key_annotated`: every annotated literal with a mapping now
    /// holds its final NEW name — never a key in `map`, since `assign` keeps generated names
    /// disjoint from every source property name — so the later `rewrite` pass has nothing
    /// left to do at these spans.
    pub fn rename_annotated_literals<'a>(
        &mut self,
        program: &mut Program<'a>,
        allocator: &'a Allocator,
    ) {
        if self.state.bail.is_some() || self.key_annotated.is_empty() {
            return;
        }
        let map = assign(&self.opts, &self.state.names);
        if map.is_empty() {
            self.map = Some(map);
            return;
        }
        {
            let mut rewriter = PropertyRewriter {
                map: &map,
                mangle_quoted: self.opts.mangle_quoted,
                key_annotated: &self.key_annotated,
                rename_annotated_only: true,
                ast: AstBuilder::new(allocator),
            };
            rewriter.visit_program(program);
        }
        // Every annotated literal with a mapping has now been renamed in place; its value
        // holds a NEW name, which is never a key in `map` (generated names are disjoint from
        // every source property name), so the later `rewrite` pass would only no-op on these
        // spans. Drop them to skip the redundant lookups. The direct collect+rewrite API never
        // calls this method, so it keeps `key_annotated` intact and renames each annotated
        // literal there instead.
        self.key_annotated.clear();
        self.map = Some(map);
    }

    /// Assign final names (if not already) and rewrite the program in place.
    ///
    /// Does nothing when the collect pass bailed, or when no name ends up being mangled.
    /// Call this **after** variable mangling.
    pub fn rewrite<'a>(mut self, program: &mut Program<'a>, allocator: &'a Allocator) {
        if self.state.bail.is_some() {
            return;
        }
        // `map` is already assigned if `rename_annotated_literals` ran; otherwise assign now.
        let map = match self.map.take() {
            Some(map) => map,
            None => assign(&self.opts, &self.state.names),
        };
        if map.is_empty() {
            return;
        }
        let mut rewriter = PropertyRewriter {
            map: &map,
            mangle_quoted: self.opts.mangle_quoted,
            key_annotated: &self.key_annotated,
            rename_annotated_only: false,
            ast: AstBuilder::new(allocator),
        };
        rewriter.visit_program(program);
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
        }
    }

    fn opts_quoted(re: &str) -> ManglePropertiesOptions {
        ManglePropertiesOptions { mangle_quoted: true, ..opts(re) }
    }

    /// Build a classification map the way collect would: `reserved` wins, so it is inserted
    /// first and `candidates` only fill the names not already reserved.
    fn classes(candidates: &[&str], reserved: &[&str]) -> FxHashMap<CompactStr, Class> {
        let mut map = FxHashMap::default();
        for name in reserved {
            map.insert(CompactStr::from(*name), Class::Reserved);
        }
        for name in candidates {
            map.entry(CompactStr::from(*name)).or_insert(Class::Candidate);
        }
        map
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
        let o = opts("^_");
        let cands = classes(&["_a", "_b"], &[]);
        let m1 = assign(&o, &cands);
        let m2 = assign(&o, &cands);
        assert_eq!(m1, m2); // deterministic
        let names: FxHashSet<_> = m1.values().collect();
        assert_eq!(names.len(), m1.len()); // no two map to the same name
    }

    #[test]
    fn generated_names_avoid_reserved() {
        // A generated name must skip a property classified `Reserved`. Only `_a` matches, so it
        // is the sole candidate; `e` (base54's first name) is reserved, so `_a` must take `t`.
        let o = opts("^_");
        let cands = classes(&["_a"], &["e"]);
        let m = assign(&o, &cands);
        assert_eq!(m[&CompactStr::from("_a")].as_str(), "t");
    }

    #[test]
    fn generated_names_avoid_candidate_originals() {
        // Generated names are disjoint from EVERY source property name, `Candidate` originals
        // included: with `e` itself a candidate, `_a` must skip `e` and take `t`, and `e` takes
        // `n`. This is what makes the rewrite's by-name re-matching apply at most once.
        let o = opts(".");
        let cands = classes(&["_a", "e"], &[]);
        let m = assign(&o, &cands);
        assert_eq!(m[&CompactStr::from("_a")].as_str(), "t");
        assert_eq!(m[&CompactStr::from("e")].as_str(), "n");
    }

    #[test]
    fn generated_names_avoid_user_reserved() {
        // An explicitly reserved name is never handed out (terser/esbuild parity). The
        // `reserve` REGEX, by contrast, only filters source-seen names.
        let mut o = opts("^_");
        o.reserved.insert("e".into());
        let cands = classes(&["_a"], &[]);
        let m = assign(&o, &cands);
        assert_eq!(m[&CompactStr::from("_a")].as_str(), "t");
    }

    fn collect_src<'a>(
        alloc: &'a oxc_allocator::Allocator,
        opts: &ManglePropertiesOptions,
        src: &str,
    ) -> PropertyCollectState {
        let st = oxc_span::SourceType::mjs();
        let ret = oxc_parser::Parser::new(alloc, src, st).parse();
        let key_annotated = key_annotated_spans(&ret.program);
        collect(opts, &key_annotated, &ret.program)
    }

    #[test]
    fn collect_classifies() {
        let alloc = oxc_allocator::Allocator::default();
        let s = collect_src(&alloc, &opts("^_"), "a._x; b['_y']; ({ _z: 1, q: 2 });");
        assert!(s.is_candidate("_x")); // unquoted member
        assert!(s.is_reserved("_y")); // quoted member (mangle_quoted off)
        assert!(s.is_candidate("_z")); // identifier key matching regex
        assert!(s.is_reserved("q")); // identifier key not matching => reserved
        assert!(s.bail.is_none());
    }

    #[test]
    fn collect_bails_on_with_and_eval() {
        let alloc = oxc_allocator::Allocator::default();
        let with = collect_src(&alloc, &opts("^_"), "with (o) { a._x }").bail;
        assert_eq!(with.map(|b| b.kind), Some(PropertyMangleBailKind::With));
        let eval = collect_src(&alloc, &opts("^_"), "eval('a._x')").bail;
        assert_eq!(eval.map(|b| b.kind), Some(PropertyMangleBailKind::DirectEval));
    }

    #[test]
    fn collect_reserves_in_operator_and_assignment_target() {
        let alloc = oxc_allocator::Allocator::default();
        let s = collect_src(&alloc, &opts("^_"), "'_x' in o; ({ _y } = o);");
        assert!(s.is_reserved("_x")); // `in` LHS (mangle_quoted off)
        assert!(s.is_reserved("_y")); // assignment-target shorthand
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
            assert!(s.is_candidate(name), "{name} should be a candidate");
            assert!(!s.is_reserved(name), "{name} should not be reserved");
        }
        // The assignment-target shorthand is still reserved regardless of mangle_quoted.
        let s2 = collect_src(&alloc, &opts_quoted("^_"), "({ _k } = o);");
        assert!(s2.is_reserved("_k"));
    }

    #[test]
    fn collect_classifies_no_substitution_template_keys() {
        let alloc = oxc_allocator::Allocator::default();
        // A no-substitution template (`` `_a` ``) in a key/index position is runtime-equivalent
        // to the string `'_a'`, so (mangle_quoted off) it reserves the name program-wide.
        let s = collect_src(
            &alloc,
            &opts("^_"),
            "a[`_x`]; `_v` in o; ({ [`_z`]: 1 }); ({ [(q, `_w`)]: 1 }); a[c ? `_u` : d];",
        );
        for name in ["_x", "_v", "_z", "_w", "_u"] {
            assert!(s.is_reserved(name), "{name} should be reserved");
            assert!(!s.is_candidate(name), "{name} should not be a candidate");
        }
        // A template WITH a substitution is not a statically-known key: neither classified.
        let s2 = collect_src(&alloc, &opts("^_"), "a[`_d${y}`];");
        assert!(!s2.is_reserved("_d"));
        assert!(!s2.is_candidate("_d"));
    }

    #[test]
    fn collect_template_candidates_when_mangle_quoted() {
        let alloc = oxc_allocator::Allocator::default();
        // With mangle_quoted on, the same no-substitution templates become candidates,
        // renamed consistently with their unquoted siblings.
        let s = collect_src(
            &alloc,
            &opts_quoted("^_"),
            "a[`_x`]; `_v` in o; ({ [`_z`]: 1 }); a[c ? `_u` : d];",
        );
        for name in ["_x", "_v", "_z", "_u"] {
            assert!(s.is_candidate(name), "{name} should be a candidate");
            assert!(!s.is_reserved(name), "{name} should not be reserved");
        }
    }

    #[test]
    fn collect_key_annotation() {
        let alloc = oxc_allocator::Allocator::default();
        // `@__KEY__` / `#__KEY__` annotate the following string/template as a property name,
        // even in non-key positions. A bare `/* __KEY__ */` (no `@`/`#`) does NOT.
        let s = collect_src(
            &alloc,
            &opts("_"),
            "x(/* __KEY__ */ '_doNotMangleThis', /* __KEY__ */ `_doNotMangleThis`);\n\
             x(/* @__KEY__ */ '_mangleThis', /* @__KEY__ */ `_mangleThis2`);\n\
             x(/* #__KEY__ */ '_mangleHash');\n\
             /* @__KEY__ */ 'notMangled';",
        );
        // Annotated and regex-matching => candidate.
        assert!(s.is_candidate("_mangleThis"));
        assert!(s.is_candidate("_mangleThis2"));
        assert!(s.is_candidate("_mangleHash"));
        // No `@`/`#` => not an annotation => plain argument string, not a candidate.
        assert!(!s.is_candidate("_doNotMangleThis"));
        assert!(!s.is_reserved("_doNotMangleThis"));
        // Annotated but regex does not match => not a candidate (eligibility still gates).
        assert!(!s.is_candidate("notMangled"));
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
            assert!(!s.is_candidate(name), "{name} must not be a candidate");
        }
    }
}
