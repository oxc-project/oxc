//! Removal-only cleanup for dead object-copy spreads of getter-free literals.
//!
//! `({ ...Proto })` in discarded position is kept whenever the spread argument
//! is an identifier, because copying a binding's own enumerable properties runs
//! that object's getters. When the binding provably still holds a getter-free
//! object literal, there are no getters to run and the copy is dead code — the
//! pattern from rolldown/rolldown#8582, which keeps whole modules alive.
//!
//! Proving it needs a whole-symbol fact ("nothing else can have installed an
//! accessor"), but the minifier's symbol data is per-pass scratch that is only
//! coherent at a pass boundary. So the fact is never computed mid-traversal:
//!
//! ```text
//! ordinary peephole pass
//!   ├─ changed → discard this pass's marks and continue
//!   └─ quiet   → census against the settled references
//!                  ↓
//!               publish candidates, run the cleanup batch
//!                  ↓
//!               removals reopen ordinary DCE
//! ```
//!
//! During traversal each object literal records the `ReferenceId` of every
//! direct `...Identifier` argument it copies — at the spread node, so the cost
//! is one bit per spread site rather than work per identifier. A pass that
//! changed anything throws its marks away; only a QUIET pass gets to consume
//! them, and a quiet pass by definition ended with the AST it started with, so
//! its marks, its `SymbolValue` entries, its liveness data and the post-flush
//! `Scoping` all describe the same program.
//!
//! The candidate test is then a positive census — *every* live reference of the
//! symbol is one of those marks — rather than a list of read positions deemed
//! harmless. Anything that is not literally an object-copy spread disqualifies
//! the symbol: a call argument, a member write, an array spread, a reference
//! this pass never saw. That is what keeps the fact honest as the AST changes
//! underneath it, and it needs no sanction list to maintain.

use oxc_allocator::{Allocator, BitSet, Vec as ArenaVec};
#[cfg(debug_assertions)]
use oxc_ast_visit::Visit;

use oxc_ast::ast::*;
use oxc_semantic::Scoping;
use oxc_syntax::{scope::ScopeFlags, symbol::SymbolId};

use oxc_ecmascript::side_effects::MayHaveSideEffects;

use crate::{
    ReusableTraverseCtx, Traverse, TraverseCtx, generated::ancestor::Ancestor,
    minifier_traverse::traverse_mut_with_ctx, peephole::PeepholeOptimizations,
    symbol_value::FreshValueKind,
};

/// Per-run state for the quiet-pass object-spread cleanup.
pub struct SpreadCleanup<'a> {
    /// References the current pass saw as the direct `...Identifier` argument
    /// of an object literal, and that passed the mark-time proof.
    ///
    /// Reset at the start of every pass. Marks from a pass that changed the AST
    /// are never consumed: consumption only follows a quiet pass, whose own
    /// marks are complete and current.
    ///
    /// Sized once from the initial `references_len()`. A reference minted
    /// during the loop is past capacity and so cannot be marked, which makes
    /// the census treat it as an unknown use and reject its symbol — the
    /// conservative direction, and the same capacity guard
    /// `PassChanges::removed_references` relies on.
    ///
    marks: BitSet<'a>,

    /// Symbols this boundary's census proved to hold a getter-free object
    /// literal whose every live reference is a dead object-copy copy.
    ///
    /// Written by [`settle_candidates`] and read only by the removal batch that
    /// immediately follows it, within the same boundary. Nothing carries it
    /// into an ordinary pass, so no transform can invalidate it while it is
    /// live, and re-deriving it at each boundary is what makes the construction
    /// idempotent.
    candidates: BitSet<'a>,
}

impl<'a> SpreadCleanup<'a> {
    pub fn new(scoping: &Scoping, allocator: &'a Allocator) -> Self {
        Self {
            marks: BitSet::new_in(scoping.references_len(), allocator),
            candidates: BitSet::new_in(scoping.symbols_len(), allocator),
        }
    }

    /// Discard all facts from the previous ordinary-pass boundary.
    ///
    /// Candidates remain live across the removal rounds that immediately
    /// follow one quiet pass, but must never survive into the next ordinary
    /// pass: that pass can expose new uses which invalidate the old census.
    pub fn begin_pass(&mut self) {
        self.marks.clear();
        self.candidates.clear();
    }
}

/// Record one `...Identifier` object-copy use.
///
/// Called from `PeepholeOptimizations::exit_spread_element`, so the cost is
/// bounded by the number of spread sites rather than by the properties of every
/// object literal. Array and call spreads reach the same hook, hence the parent
/// test: only an object literal's own `properties` list is a copy.
///
/// The generated walk fires this exactly once per node per pass, in traversal
/// order — `minimize_statements` and its helpers re-process already-walked
/// statements with plain functions and never re-enter the traversal dispatch.
///
/// A spread that fails the mark-time proof is simply left unmarked, which makes
/// the census reject its symbol.
pub fn mark_object_copy_spread<'a>(spread: &SpreadElement<'a>, ctx: &mut TraverseCtx<'a>) {
    // The mark-time order proof and `PeepholeOptimizations::summary_order_proven`
    // read the same signal — "a `SymbolValue` entry exists right now" — as
    // evidence that the declarator was traversed earlier this pass. That
    // consumer needs the flag because it runs from positions reached during
    // statement-list re-processing; marking is a traversal hook and cannot be,
    // which is exactly what this asserts. Move marking to `exit_expression` and
    // the assertion is what tells you the proof no longer holds.
    debug_assert!(
        !ctx.state.reprocessing_statements,
        "spread marks require traversal-order recording; statement-list \
         re-processing invalidates the declarator-precedes-use proof",
    );
    if !matches!(ctx.parent(), Ancestor::ObjectExpressionProperties(_)) {
        return;
    }
    let Expression::Identifier(ident) = &spread.argument else { return };
    let Some(reference_id) = ident.reference_id.get() else { return };
    let Some(symbol_id) = ctx.scoping().get_reference(reference_id).symbol_id() else { return };
    if !mark_time_proof_holds(symbol_id, ctx) {
        return;
    }
    let index = reference_id.index();
    if index < ctx.state.spread_cleanup.marks.capacity() {
        ctx.state.spread_cleanup.marks.set_bit(index);
    }
}

/// Whether this spread may stand as evidence about the symbol's value.
///
/// Both halves are checked here rather than at the boundary because here the
/// traversal cursor IS the spread's own position — at the boundary there is no
/// cursor to ask.
///
/// - Declaration before use: `SymbolState::values` is per-pass scratch written
///   by `exit_variable_declarator`, so an entry existing at this spread proves
///   the declarator precedes it in this pass's traversal.
/// - Same function: an entry orders the traversal, not execution. A nested
///   function can run before a hoisted `var`'s initializer, when the binding
///   holds `undefined` rather than the classified literal.
///
/// Both are stricter than the object case alone needs — spreading `undefined`
/// is a no-op, and a `const` read before its declarator is a TDZ
/// `ReferenceError` inside the documented `No TDZ Violation` assumption — but
/// they are what lets the census be a plain membership test, with no reasoning
/// about which reads observe which value.
fn mark_time_proof_holds(symbol_id: SymbolId, ctx: &TraverseCtx<'_>) -> bool {
    if ctx.state.symbols.value(symbol_id).is_none() {
        return false;
    }
    let scoping = ctx.scoping();
    let symbol_scope_id = scoping.symbol_scope_id(symbol_id);
    !scoping
        .scope_ancestors(ctx.current_scope_id())
        .take_while(|&scope_id| scope_id != symbol_scope_id)
        .any(|scope_id| scoping.scope_flags(scope_id).is_function())
}

/// Publish the symbols whose dead object-copy spreads the immediate cleanup
/// batch may remove, and report whether there are any.
///
/// Called only at a quiet-pass boundary, after `finish_pass` has flushed the
/// pass's changes, so `Scoping` lists exactly the references the settled AST
/// contains.
///
/// A symbol qualifies when all of the following hold:
///
/// - its per-pass value entry classifies it `FreshValueKind::Object`, which
///   `fresh_value_kind` grants only to a literal with no getter, no setter and
///   no `__proto__:` key, and which `init_value` withholds entirely from a
///   symbol with more than one value declaration;
/// - it has no resolved write, so nothing replaces the classified value;
/// - its declaring scope contains no direct `eval`, which could reach the
///   binding without a resolved reference;
/// - liveness data exists and does not mark it implicitly observable (a module
///   export, a script global, an Annex B alias, a `using` resource). Absent
///   liveness means observability is unknown, which is not a candidate;
/// - the census: every live resolved reference is one of this pass's marks. An
///   ordinary read, a call argument, a member write, an array spread, or a
///   reference minted after the last flush is not a mark, and any one of them
///   disqualifies the symbol.
pub fn settle_candidates(
    #[cfg_attr(not(debug_assertions), expect(unused_variables))] program: &Program<'_>,
    ctx: &mut TraverseCtx<'_>,
) -> bool {
    if ctx.state.spread_cleanup.marks.is_empty() {
        return false;
    }
    let mut found_candidates = false;
    // Scanning dense symbol IDs avoids allocating a temporary symbol list and
    // is no broader than scanning the reference-sized mark bitset. The census
    // itself rejects symbols without at least one marked live reference.
    for index in 0..ctx.scoping().symbols_len() {
        let symbol_id = SymbolId::from_usize(index);
        if !ctx.state.spread_cleanup.candidates.contains(symbol_id.index())
            && is_candidate(symbol_id, ctx)
        {
            ctx.state.spread_cleanup.candidates.set_bit(symbol_id.index());
            found_candidates = true;
        }
    }
    #[cfg(debug_assertions)]
    if found_candidates {
        debug_assert_census_complete(program, ctx);
    }
    found_candidates
}

/// Debug-only guard on the census's core premise: `Scoping` lists EVERY live
/// reference of a candidate symbol.
///
/// The dangerous direction is a live reference to a candidate that the census
/// never saw — an escape channel hidden behind an id the resolved-reference
/// list omits, or a node whose `ReferenceId` is aliased by another. Either
/// would let a copy be removed whose value was mutated behind the census's
/// back. This is the same shape as `debug_assert_no_over_prune`, and costs
/// nothing in release.
///
/// Two mechanisms could produce it, and both hold today: the minifier's only
/// `clone_in` (`substitute_alternate_syntax.rs`) resets `reference_id` and
/// mints fresh ones, and `remove_unused_object_expr`'s literal rebuilding always
/// exits through `ctx.replace_expression`, which records a change and so cannot
/// happen on a quiet pass. Neither is enforced by anything but this walk.
#[cfg(debug_assertions)]
fn debug_assert_census_complete(program: &Program<'_>, ctx: &TraverseCtx<'_>) {
    struct CensusCheck<'b, 'c> {
        ctx: &'b TraverseCtx<'c>,
    }
    impl<'a> Visit<'a> for CensusCheck<'_, '_> {
        fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
            let Some(reference_id) = it.reference_id.get() else { return };
            let Some(symbol_id) = self.ctx.scoping().get_reference(reference_id).symbol_id() else {
                return;
            };
            if !self.ctx.state.spread_cleanup.candidates.contains(symbol_id.index()) {
                return;
            }
            assert!(
                self.ctx.state.spread_cleanup.marks.contains(reference_id.index()),
                "spread census incompleteness: reference {} of candidate symbol {} is live in \
                 the program but was never marked as an object copy — `Scoping` did not list \
                 it, or two nodes share the id",
                reference_id.index(),
                symbol_id.index(),
            );
        }
    }
    CensusCheck { ctx }.visit_program(program);
}

fn is_candidate(symbol_id: SymbolId, ctx: &TraverseCtx<'_>) -> bool {
    let Some(value) = ctx.state.symbols.value(symbol_id) else { return false };
    if value.kind != FreshValueKind::Object || value.references.has_writes() {
        return false;
    }
    let scoping = ctx.scoping();
    if scoping.scope_flags(scoping.symbol_scope_id(symbol_id)).contains(ScopeFlags::DirectEval) {
        return false;
    }
    let Some(liveness) = ctx.state.symbols.liveness() else { return false };
    if liveness.is_implicitly_observable(symbol_id) {
        return false;
    }
    let mut reference_ids = scoping.get_resolved_reference_ids(symbol_id).iter();
    reference_ids.next().is_some_and(|reference_id| {
        ctx.state.spread_cleanup.marks.contains(reference_id.index())
            && reference_ids
                .all(|reference_id| ctx.state.spread_cleanup.marks.contains(reference_id.index()))
    })
}

/// One round of the boundary's removal batch: delete the dead object copies the
/// census just proved, and report whether anything went.
///
/// This is a restricted scope-aware traversal, not a peephole pass. Nothing
/// else transforms while it runs, so the candidate set cannot be invalidated
/// underneath it, and the two shapes it removes are the only ones it knows:
///
/// - an expression statement whose value is discarded and whose every effect is
///   a candidate copy or an effect-free property;
/// - a variable declarator with no live reference left whose initializer is the
///   same.
///
/// The second is what makes a chain collapse. `const P1 = { ...P0 }` keeps `P0`
/// alive only through `P1`'s declarator, so each layer becomes removable one
/// round after the layer above it. The caller loops rounds — pruning references
/// between them so the next round sees current counts — until a round removes
/// nothing. References only ever shrink, so it terminates, and a census member
/// stays a member: every remaining reference was already a mark.
pub fn remove_dead_copies<'a>(
    program: &mut Program<'a>,
    ctx: &mut ReusableTraverseCtx<'a>,
) -> bool {
    let mut batch = RemovalBatch { removed: false };
    traverse_mut_with_ctx(&mut batch, program, ctx);
    batch.removed
}

struct RemovalBatch {
    removed: bool,
}

impl<'a> Traverse<'a> for RemovalBatch {
    fn exit_statements(
        &mut self,
        statements: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        statements.retain_mut(|statement| !self.statement_is_dead(statement, ctx));
    }
}

impl RemovalBatch {
    fn statement_is_dead<'a>(
        &mut self,
        statement: &mut Statement<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> bool {
        match statement {
            Statement::ExpressionStatement(statement) => {
                if !self.strip_discarded_expression(&mut statement.expression, ctx) {
                    return false;
                }
                ctx.drop_expression(&statement.expression);
                self.removed = true;
                true
            }
            Statement::VariableDeclaration(declaration) => {
                self.strip_dead_declarators(declaration, ctx)
            }
            _ => false,
        }
    }

    /// Take the census-proven copies out of an expression whose value is
    /// discarded, and report whether nothing effectful is left.
    ///
    /// Deliberately narrow: only the shapes this batch can rewrite. Anything
    /// else is left to ordinary DCE, which sees it on the next pass.
    ///
    /// Every operand of a discarded sequence is itself discarded, the last one
    /// included — true only because the statement's own value is unused, which
    /// is why the position test lives here rather than at the mark site, where
    /// a sequence in value position is indistinguishable. Full minify fuses
    /// statements into sequences, so a copy routinely ends up beside operands
    /// that must stay; stripping per operand rather than testing the whole
    /// statement is what keeps those removals.
    fn strip_discarded_expression<'a>(
        &mut self,
        expression: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> bool {
        match expression {
            Expression::ObjectExpression(object) => {
                if object.properties.iter().all(|property| Self::property_is_dead(property, ctx)) {
                    // The caller drops the literal whole, references included.
                    return true;
                }
                // Something in the literal must stay, so take out only the
                // copies themselves. Their argument is a bare identifier, so
                // one `drop_expression` prunes exactly the reference that goes.
                object.properties.retain(|property| {
                    let ObjectPropertyKind::SpreadProperty(spread) = property else { return true };
                    if !Self::is_candidate_copy(&spread.argument, ctx) {
                        return true;
                    }
                    ctx.drop_expression(&spread.argument);
                    self.removed = true;
                    false
                });
                false
            }
            Expression::SequenceExpression(sequence) => {
                sequence.expressions.retain_mut(|operand| {
                    if self.strip_discarded_expression(operand, ctx) {
                        ctx.drop_expression(operand);
                        self.removed = true;
                        false
                    } else {
                        true
                    }
                });
                sequence.expressions.is_empty()
            }
            _ => false,
        }
    }

    /// The non-mutating form, for an initializer that is about to be dropped
    /// whole rather than rewritten in place.
    fn discarded_expression_is_dead<'a>(
        expression: &Expression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
        match expression {
            Expression::ObjectExpression(object) => {
                object.properties.iter().all(|property| Self::property_is_dead(property, ctx))
            }
            Expression::SequenceExpression(sequence) => sequence
                .expressions
                .iter()
                .all(|expression| Self::discarded_expression_is_dead(expression, ctx)),
            _ => false,
        }
    }

    fn property_is_dead<'a>(property: &ObjectPropertyKind<'a>, ctx: &TraverseCtx<'a>) -> bool {
        if let ObjectPropertyKind::SpreadProperty(spread) = property
            && Self::is_candidate_copy(&spread.argument, ctx)
        {
            return true;
        }
        !property.may_have_side_effects(ctx)
    }

    fn is_candidate_copy<'a>(argument: &Expression<'a>, ctx: &TraverseCtx<'a>) -> bool {
        let Expression::Identifier(ident) = argument else { return false };
        let Some(reference_id) = ident.reference_id.get() else { return false };
        ctx.scoping().get_reference(reference_id).symbol_id().is_some_and(|symbol_id| {
            ctx.state.spread_cleanup.candidates.contains(symbol_id.index())
        })
    }

    /// Drop declarators whose binding has no live reference left and whose
    /// initializer is dead, and report whether the declaration is now empty.
    ///
    /// `should_remove_unused_declarator` supplies the unusedness rule (and its
    /// `using` and configuration guards); requiring the initializer to be dead
    /// on top is what keeps this from discarding effects ordinary DCE would
    /// have preserved by rewriting the declarator into an expression statement.
    fn strip_dead_declarators<'a>(
        &mut self,
        declaration: &mut VariableDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> bool {
        if !PeepholeOptimizations::can_remove_unused_declarators(ctx) {
            return false;
        }
        let kind = declaration.kind;
        declaration.declarations.retain_mut(|declarator| {
            if !PeepholeOptimizations::should_remove_unused_declarator(declarator, kind, ctx)
                || !declarator
                    .init
                    .as_ref()
                    .is_some_and(|init| Self::discarded_expression_is_dead(init, ctx))
            {
                return true;
            }
            ctx.drop_variable_declarator(declarator);
            self.removed = true;
            false
        });
        declaration.declarations.is_empty()
    }
}
