/// Infer reactive scope variables.
///
/// Port of `ReactiveScopes/InferReactiveScopeVariables.ts` from the React Compiler.
///
/// For each mutable variable, infers a reactive scope which will construct that
/// variable. Variables that co-mutate are assigned to the same reactive scope.
///
/// This is the 1st of 4 passes that determine how to break a function into
/// discrete reactive scopes:
/// 1. InferReactiveScopeVariables (this pass)
/// 2. AlignReactiveScopesToBlockScopes
/// 3. MergeOverlappingReactiveScopes
/// 4. BuildReactiveBlocks
use oxc_span::Span;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    compiler_error::SourceLocation,
    hir::{
        HIRFunction, Identifier, IdentifierId, InstructionId, MutableRange, ReactiveScope,
        visitors::each_instruction_operand,
    },
    utils::disjoint_set::DisjointSet,
};

/// Check if an identifier is mutable at the given instruction.
pub fn is_mutable(identifier: &Identifier, at_instruction: InstructionId) -> bool {
    let range = &identifier.mutable_range;
    at_instruction >= range.start && at_instruction < range.end
}

/// Merge two source locations, taking the wider span.
///
/// If either location is generated, returns the other. If both are real source
/// locations, returns a span covering both.
fn merge_location(l: SourceLocation, r: SourceLocation) -> SourceLocation {
    match (l, r) {
        (SourceLocation::Generated, r) => r,
        (l, SourceLocation::Generated) => l,
        (SourceLocation::Source(l_span), SourceLocation::Source(r_span)) => SourceLocation::Source(
            Span::new(l_span.start.min(r_span.start), l_span.end.max(r_span.end)),
        ),
    }
}

/// Infer reactive scope variables for the function.
///
/// Phase 1: Groups co-mutating identifiers using disjoint sets.
/// Phase 2: Creates ReactiveScope objects and assigns them to identifiers.
///
/// # Panics
/// Panics if a scope has an invalid mutable range.
pub fn infer_reactive_scope_variables(func: &mut HIRFunction) {
    // Phase 1: Find groups of co-mutating identifiers using disjoint sets
    let mut scope_identifiers = find_disjoint_mutable_values(func);

    // Collect identifier info (mutable_range, loc) indexed by IdentifierId.
    // We need this because the DisjointSet only stores IdentifierIds, but
    // creating scopes requires the mutable ranges and source locations.
    let mut identifier_info: FxHashMap<IdentifierId, (MutableRange, SourceLocation)> =
        FxHashMap::default();
    for block in func.body.blocks.values() {
        for phi in &block.phis {
            let id = phi.place.identifier.id;
            identifier_info
                .entry(id)
                .or_insert((phi.place.identifier.mutable_range, phi.place.identifier.loc));
            for phi_operand in phi.operands.values() {
                let oid = phi_operand.identifier.id;
                identifier_info
                    .entry(oid)
                    .or_insert((phi_operand.identifier.mutable_range, phi_operand.identifier.loc));
            }
        }
        for instr in &block.instructions {
            let lid = instr.lvalue.identifier.id;
            identifier_info
                .entry(lid)
                .or_insert((instr.lvalue.identifier.mutable_range, instr.lvalue.identifier.loc));
            for place in each_instruction_operand(instr) {
                let oid = place.identifier.id;
                identifier_info
                    .entry(oid)
                    .or_insert((place.identifier.mutable_range, place.identifier.loc));
            }
        }
    }

    // Phase 2: Create ReactiveScope objects for each group
    //
    // Maps each group root IdentifierId to its ReactiveScope.
    let mut scopes: FxHashMap<IdentifierId, ReactiveScope> = FxHashMap::default();

    scope_identifiers.for_each(|identifier_id, group_identifier_id| {
        let Some(&(mutable_range, loc)) = identifier_info.get(identifier_id) else {
            return;
        };

        if let Some(scope) = scopes.get_mut(group_identifier_id) {
            // Extend the existing scope's range
            if scope.range.start == InstructionId(0) {
                scope.range.start = mutable_range.start;
            } else if mutable_range.start != InstructionId(0) {
                scope.range.start = InstructionId(scope.range.start.0.min(mutable_range.start.0));
            }
            scope.range.end = InstructionId(scope.range.end.0.max(mutable_range.end.0));
            scope.loc = merge_location(scope.loc, loc);
        } else {
            // Create a new scope for this group
            let scope_id = func.env.next_scope_id();
            scopes.insert(
                *group_identifier_id,
                ReactiveScope {
                    id: scope_id,
                    range: mutable_range,
                    dependencies: FxHashSet::default(),
                    declarations: FxHashMap::default(),
                    reassignments: FxHashSet::default(),
                    early_return_value: None,
                    merged: FxHashSet::default(),
                    loc,
                },
            );
        }
    });

    // Build a map from each IdentifierId to its group's scope (and the scope's range).
    // We need to call for_each again to build the full mapping because for_each
    // only provides (item, group) pairs; we need to look up the scope by group.
    let mut id_to_scope: FxHashMap<IdentifierId, (Box<ReactiveScope>, MutableRange)> =
        FxHashMap::default();
    scope_identifiers.for_each(|identifier_id, group_identifier_id| {
        if let Some(scope) = scopes.get(group_identifier_id) {
            let scope_range = scope.range;
            id_to_scope
                .entry(*identifier_id)
                .or_insert_with(|| (Box::new(scope.clone()), scope_range));
        }
    });

    // Compute max instruction ID for validation
    let mut max_instruction = InstructionId(0);
    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            if instr.id.0 > max_instruction.0 {
                max_instruction = instr.id;
            }
        }
        let terminal_id = block.terminal.id();
        if terminal_id.0 > max_instruction.0 {
            max_instruction = terminal_id;
        }
    }

    // Validate that all scopes have properly initialized, valid mutable ranges
    // within the span of instructions for this function.
    for scope in scopes.values() {
        if scope.range.start == InstructionId(0)
            || scope.range.end == InstructionId(0)
            || max_instruction == InstructionId(0)
            || scope.range.end.0 > max_instruction.0 + 1
        {
            // In the TS version this is a CompilerError.invariant, but since this
            // function doesn't return Result, we panic to match the TS behavior
            // of throwing an invariant error.
            panic!(
                "Invalid mutable range for scope: Scope @{} has range [{}:{}] but the valid range is [1:{}]",
                scope.id.0,
                scope.range.start.0,
                scope.range.end.0,
                max_instruction.0 + 1,
            );
        }
    }

    // Phase 2b: Walk the HIR and assign scopes to identifiers
    assign_scopes_to_identifiers(func, &id_to_scope);
}

/// Walk the entire HIR function and assign scope + mutable_range to every
/// identifier whose IdentifierId appears in `id_to_scope`.
fn assign_scopes_to_identifiers(
    func: &mut HIRFunction,
    id_to_scope: &FxHashMap<IdentifierId, (Box<ReactiveScope>, MutableRange)>,
) {
    fn apply_scope(
        identifier: &mut Identifier,
        id_to_scope: &FxHashMap<IdentifierId, (Box<ReactiveScope>, MutableRange)>,
    ) {
        if let Some((scope, range)) = id_to_scope.get(&identifier.id) {
            identifier.scope = Some(scope.clone());
            identifier.mutable_range = *range;
        }
    }

    fn apply_scope_to_place(
        place: &mut crate::hir::Place,
        id_to_scope: &FxHashMap<IdentifierId, (Box<ReactiveScope>, MutableRange)>,
    ) {
        apply_scope(&mut place.identifier, id_to_scope);
    }

    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        let Some(block) = func.body.blocks.get_mut(&block_id) else {
            continue;
        };

        // Apply scopes to phi nodes
        for phi in &mut block.phis {
            apply_scope_to_place(&mut phi.place, id_to_scope);
            for phi_operand in phi.operands.values_mut() {
                apply_scope_to_place(phi_operand, id_to_scope);
            }
        }

        // Apply scopes to instructions
        for instr in &mut block.instructions {
            // Apply to the instruction lvalue
            apply_scope_to_place(&mut instr.lvalue, id_to_scope);

            // Apply to instruction-value-specific lvalues and operands
            apply_scope_to_instruction_value(&mut instr.value, id_to_scope);
        }
    }
}

/// Apply scope assignments to all identifiers within an instruction value.
fn apply_scope_to_instruction_value(
    value: &mut crate::hir::InstructionValue,
    id_to_scope: &FxHashMap<IdentifierId, (Box<ReactiveScope>, MutableRange)>,
) {
    use crate::hir::InstructionValue;

    fn apply(
        place: &mut crate::hir::Place,
        id_to_scope: &FxHashMap<IdentifierId, (Box<ReactiveScope>, MutableRange)>,
    ) {
        if let Some((scope, range)) = id_to_scope.get(&place.identifier.id) {
            place.identifier.scope = Some(scope.clone());
            place.identifier.mutable_range = *range;
        }
    }

    fn apply_call_args(
        args: &mut [crate::hir::CallArg],
        id_to_scope: &FxHashMap<IdentifierId, (Box<ReactiveScope>, MutableRange)>,
    ) {
        for arg in args.iter_mut() {
            match arg {
                crate::hir::CallArg::Place(p) => apply(p, id_to_scope),
                crate::hir::CallArg::Spread(s) => apply(&mut s.place, id_to_scope),
            }
        }
    }

    fn apply_pattern(
        pattern: &mut crate::hir::Pattern,
        id_to_scope: &FxHashMap<IdentifierId, (Box<ReactiveScope>, MutableRange)>,
    ) {
        match pattern {
            crate::hir::Pattern::Array(arr) => {
                for item in &mut arr.items {
                    match item {
                        crate::hir::ArrayPatternElement::Place(p) => apply(p, id_to_scope),
                        crate::hir::ArrayPatternElement::Spread(s) => {
                            apply(&mut s.place, id_to_scope);
                        }
                        crate::hir::ArrayPatternElement::Hole => {}
                    }
                }
            }
            crate::hir::Pattern::Object(obj) => {
                for prop in &mut obj.properties {
                    match prop {
                        crate::hir::ObjectPatternProperty::Property(p) => {
                            if let crate::hir::ObjectPropertyKey::Computed(ref mut place) = p.key {
                                apply(place, id_to_scope);
                            }
                            apply(&mut p.place, id_to_scope);
                        }
                        crate::hir::ObjectPatternProperty::Spread(s) => {
                            apply(&mut s.place, id_to_scope);
                        }
                    }
                }
            }
        }
    }

    match value {
        InstructionValue::LoadLocal(v) => apply(&mut v.place, id_to_scope),
        InstructionValue::LoadContext(v) => apply(&mut v.place, id_to_scope),
        InstructionValue::DeclareLocal(v) => apply(&mut v.lvalue.place, id_to_scope),
        InstructionValue::DeclareContext(v) => apply(&mut v.lvalue_place, id_to_scope),
        InstructionValue::StoreLocal(v) => {
            apply(&mut v.lvalue.place, id_to_scope);
            apply(&mut v.value, id_to_scope);
        }
        InstructionValue::StoreContext(v) => {
            apply(&mut v.lvalue_place, id_to_scope);
            apply(&mut v.value, id_to_scope);
        }
        InstructionValue::Destructure(v) => {
            apply_pattern(&mut v.lvalue.pattern, id_to_scope);
            apply(&mut v.value, id_to_scope);
        }
        InstructionValue::BinaryExpression(v) => {
            apply(&mut v.left, id_to_scope);
            apply(&mut v.right, id_to_scope);
        }
        InstructionValue::UnaryExpression(v) => apply(&mut v.value, id_to_scope),
        InstructionValue::TypeCastExpression(v) => apply(&mut v.value, id_to_scope),
        InstructionValue::CallExpression(v) => {
            apply(&mut v.callee, id_to_scope);
            apply_call_args(&mut v.args, id_to_scope);
        }
        InstructionValue::MethodCall(v) => {
            apply(&mut v.receiver, id_to_scope);
            apply(&mut v.property, id_to_scope);
            apply_call_args(&mut v.args, id_to_scope);
        }
        InstructionValue::NewExpression(v) => {
            apply(&mut v.callee, id_to_scope);
            apply_call_args(&mut v.args, id_to_scope);
        }
        InstructionValue::ObjectExpression(v) => {
            for prop in &mut v.properties {
                match prop {
                    crate::hir::ObjectPatternProperty::Property(p) => {
                        if let crate::hir::ObjectPropertyKey::Computed(ref mut place) = p.key {
                            apply(place, id_to_scope);
                        }
                        apply(&mut p.place, id_to_scope);
                    }
                    crate::hir::ObjectPatternProperty::Spread(s) => {
                        apply(&mut s.place, id_to_scope);
                    }
                }
            }
        }
        InstructionValue::ArrayExpression(v) => {
            for elem in &mut v.elements {
                match elem {
                    crate::hir::ArrayExpressionElement::Place(p) => apply(p, id_to_scope),
                    crate::hir::ArrayExpressionElement::Spread(s) => {
                        apply(&mut s.place, id_to_scope);
                    }
                    crate::hir::ArrayExpressionElement::Hole => {}
                }
            }
        }
        InstructionValue::JsxExpression(v) => {
            if let crate::hir::JsxTag::Place(ref mut p) = v.tag {
                apply(p, id_to_scope);
            }
            for attr in &mut v.props {
                match attr {
                    crate::hir::JsxAttribute::Attribute { place, .. } => {
                        apply(place, id_to_scope);
                    }
                    crate::hir::JsxAttribute::Spread { argument } => {
                        apply(argument, id_to_scope);
                    }
                }
            }
            if let Some(children) = &mut v.children {
                for child in children.iter_mut() {
                    apply(child, id_to_scope);
                }
            }
        }
        InstructionValue::JsxFragment(v) => {
            for child in &mut v.children {
                apply(child, id_to_scope);
            }
        }
        InstructionValue::PropertyLoad(v) => apply(&mut v.object, id_to_scope),
        InstructionValue::PropertyStore(v) => {
            apply(&mut v.object, id_to_scope);
            apply(&mut v.value, id_to_scope);
        }
        InstructionValue::PropertyDelete(v) => apply(&mut v.object, id_to_scope),
        InstructionValue::ComputedLoad(v) => {
            apply(&mut v.object, id_to_scope);
            apply(&mut v.property, id_to_scope);
        }
        InstructionValue::ComputedStore(v) => {
            apply(&mut v.object, id_to_scope);
            apply(&mut v.property, id_to_scope);
            apply(&mut v.value, id_to_scope);
        }
        InstructionValue::ComputedDelete(v) => {
            apply(&mut v.object, id_to_scope);
            apply(&mut v.property, id_to_scope);
        }
        InstructionValue::StoreGlobal(v) => apply(&mut v.value, id_to_scope),
        InstructionValue::FunctionExpression(v) => {
            for ctx in &mut v.lowered_func.func.context {
                apply(ctx, id_to_scope);
            }
        }
        InstructionValue::ObjectMethod(v) => {
            for ctx in &mut v.lowered_func.func.context {
                apply(ctx, id_to_scope);
            }
        }
        InstructionValue::TaggedTemplateExpression(v) => apply(&mut v.tag, id_to_scope),
        InstructionValue::TemplateLiteral(v) => {
            for subexpr in &mut v.subexprs {
                apply(subexpr, id_to_scope);
            }
        }
        InstructionValue::Await(v) => apply(&mut v.value, id_to_scope),
        InstructionValue::GetIterator(v) => apply(&mut v.collection, id_to_scope),
        InstructionValue::IteratorNext(v) => {
            apply(&mut v.iterator, id_to_scope);
            apply(&mut v.collection, id_to_scope);
        }
        InstructionValue::NextPropertyOf(v) => apply(&mut v.value, id_to_scope),
        InstructionValue::PrefixUpdate(v) => {
            apply(&mut v.lvalue, id_to_scope);
            apply(&mut v.value, id_to_scope);
        }
        InstructionValue::PostfixUpdate(v) => {
            apply(&mut v.lvalue, id_to_scope);
            apply(&mut v.value, id_to_scope);
        }
        InstructionValue::StartMemoize(v) => {
            if let Some(deps) = &mut v.deps {
                for dep in deps.iter_mut() {
                    if let crate::hir::ManualMemoDependencyRoot::NamedLocal {
                        ref mut value, ..
                    } = dep.root
                    {
                        apply(value, id_to_scope);
                    }
                }
            }
        }
        InstructionValue::FinishMemoize(v) => apply(&mut v.decl, id_to_scope),
        InstructionValue::LoadGlobal(_)
        | InstructionValue::Primitive(_)
        | InstructionValue::JsxText(_)
        | InstructionValue::RegExpLiteral(_)
        | InstructionValue::MetaProperty(_)
        | InstructionValue::Debugger(_)
        | InstructionValue::UnsupportedNode(_) => {}
    }
}

/// Find all sets of disjoint mutable values in the function.
///
/// Port of `findDisjointMutableValues` from the TS reference.
pub fn find_disjoint_mutable_values(func: &HIRFunction) -> DisjointSet<IdentifierId> {
    let mut scope_identifiers: DisjointSet<IdentifierId> = DisjointSet::new();

    let mut declarations: FxHashMap<crate::hir::DeclarationId, IdentifierId> = FxHashMap::default();

    for block in func.body.blocks.values() {
        // If a phi is mutated after creation, then we need to alias all of its
        // operands such that they are assigned to the same scope.
        for phi in &block.phis {
            let phi_range = &phi.place.identifier.mutable_range;
            let first_instr_id =
                block.instructions.first().map_or(block.terminal.id(), |instr| instr.id);
            if phi_range.start.0 + 1 != phi_range.end.0 && phi_range.end > first_instr_id {
                let mut operands = vec![phi.place.identifier.id];
                if let Some(&decl_id) = declarations.get(&phi.place.identifier.declaration_id) {
                    operands.push(decl_id);
                }
                for phi_operand in phi.operands.values() {
                    operands.push(phi_operand.identifier.id);
                }
                scope_identifiers.union(&operands);
            } else if func.env.config.enable_forest {
                for phi_operand in phi.operands.values() {
                    scope_identifiers.union(&[phi.place.identifier.id, phi_operand.identifier.id]);
                }
            }
        }

        for instr in &block.instructions {
            let mut operands: Vec<IdentifierId> = Vec::new();

            // Check the lvalue: include if its mutable range extends beyond
            // the instruction, or if it may allocate
            let lvalue_range = &instr.lvalue.identifier.mutable_range;
            if lvalue_range.end.0 > lvalue_range.start.0 + 1 || may_allocate(&func.env, instr) {
                operands.push(instr.lvalue.identifier.id);
            }

            match &instr.value {
                crate::hir::InstructionValue::DeclareLocal(v) => {
                    declare_identifier(&mut declarations, &v.lvalue.place);
                }
                crate::hir::InstructionValue::DeclareContext(v) => {
                    declare_identifier(&mut declarations, &v.lvalue_place);
                }
                crate::hir::InstructionValue::StoreLocal(v) => {
                    declare_identifier(&mut declarations, &v.lvalue.place);
                    let lvalue_range = &v.lvalue.place.identifier.mutable_range;
                    if lvalue_range.end.0 > lvalue_range.start.0 + 1 {
                        operands.push(v.lvalue.place.identifier.id);
                    }
                    if is_mutable(&v.value.identifier, instr.id)
                        && v.value.identifier.mutable_range.start.0 > 0
                    {
                        operands.push(v.value.identifier.id);
                    }
                }
                crate::hir::InstructionValue::StoreContext(v) => {
                    declare_identifier(&mut declarations, &v.lvalue_place);
                    let lvalue_range = &v.lvalue_place.identifier.mutable_range;
                    if lvalue_range.end.0 > lvalue_range.start.0 + 1 {
                        operands.push(v.lvalue_place.identifier.id);
                    }
                    if is_mutable(&v.value.identifier, instr.id)
                        && v.value.identifier.mutable_range.start.0 > 0
                    {
                        operands.push(v.value.identifier.id);
                    }
                }
                crate::hir::InstructionValue::Destructure(v) => {
                    for place in each_pattern_operand(&v.lvalue.pattern) {
                        declare_identifier(&mut declarations, place);
                        let pr = &place.identifier.mutable_range;
                        if pr.end.0 > pr.start.0 + 1 {
                            operands.push(place.identifier.id);
                        }
                    }
                    if is_mutable(&v.value.identifier, instr.id)
                        && v.value.identifier.mutable_range.start.0 > 0
                    {
                        operands.push(v.value.identifier.id);
                    }
                }
                crate::hir::InstructionValue::MethodCall(_) => {
                    for operand in each_instruction_operand(instr) {
                        if is_mutable(&operand.identifier, instr.id)
                            && operand.identifier.mutable_range.start.0 > 0
                        {
                            operands.push(operand.identifier.id);
                        }
                    }
                    // Ensure that the ComputedLoad to resolve the method is
                    // in the same scope as the call itself
                    if let crate::hir::InstructionValue::MethodCall(v) = &instr.value {
                        operands.push(v.property.identifier.id);
                    }
                }
                _ => {
                    for operand in each_instruction_operand(instr) {
                        if is_mutable(&operand.identifier, instr.id)
                            && operand.identifier.mutable_range.start.0 > 0
                        {
                            operands.push(operand.identifier.id);
                        }
                    }
                }
            }

            if !operands.is_empty() {
                scope_identifiers.union(&operands);
            }
        }
    }

    scope_identifiers
}

fn declare_identifier(
    declarations: &mut FxHashMap<crate::hir::DeclarationId, IdentifierId>,
    place: &crate::hir::Place,
) {
    declarations.entry(place.identifier.declaration_id).or_insert(place.identifier.id);
}

/// Iterate over all Places in a destructuring pattern.
fn each_pattern_operand(pattern: &crate::hir::Pattern) -> Vec<&crate::hir::Place> {
    let mut places = Vec::new();
    collect_pattern_operands(pattern, &mut places);
    places
}

fn collect_pattern_operands<'a>(
    pattern: &'a crate::hir::Pattern,
    out: &mut Vec<&'a crate::hir::Place>,
) {
    match pattern {
        crate::hir::Pattern::Array(arr) => {
            for item in &arr.items {
                match item {
                    crate::hir::ArrayPatternElement::Place(p) => out.push(p),
                    crate::hir::ArrayPatternElement::Spread(s) => out.push(&s.place),
                    crate::hir::ArrayPatternElement::Hole => {}
                }
            }
        }
        crate::hir::Pattern::Object(obj) => {
            for prop in &obj.properties {
                match prop {
                    crate::hir::ObjectPatternProperty::Property(p) => out.push(&p.place),
                    crate::hir::ObjectPatternProperty::Spread(s) => out.push(&s.place),
                }
            }
        }
    }
}

/// Check if an instruction may allocate a new value.
///
/// Port of `mayAllocate` from the TS reference.
fn may_allocate(
    _env: &crate::hir::environment::Environment,
    instruction: &crate::hir::Instruction,
) -> bool {
    use crate::hir::InstructionValue;

    match &instruction.value {
        InstructionValue::Destructure(v) => does_pattern_contain_spread(&v.lvalue.pattern),
        InstructionValue::PostfixUpdate(_)
        | InstructionValue::PrefixUpdate(_)
        | InstructionValue::Await(_)
        | InstructionValue::DeclareLocal(_)
        | InstructionValue::DeclareContext(_)
        | InstructionValue::StoreLocal(_)
        | InstructionValue::LoadGlobal(_)
        | InstructionValue::MetaProperty(_)
        | InstructionValue::TypeCastExpression(_)
        | InstructionValue::LoadLocal(_)
        | InstructionValue::LoadContext(_)
        | InstructionValue::StoreContext(_)
        | InstructionValue::PropertyDelete(_)
        | InstructionValue::ComputedLoad(_)
        | InstructionValue::ComputedDelete(_)
        | InstructionValue::JsxText(_)
        | InstructionValue::TemplateLiteral(_)
        | InstructionValue::Primitive(_)
        | InstructionValue::GetIterator(_)
        | InstructionValue::IteratorNext(_)
        | InstructionValue::NextPropertyOf(_)
        | InstructionValue::Debugger(_)
        | InstructionValue::StartMemoize(_)
        | InstructionValue::FinishMemoize(_)
        | InstructionValue::UnaryExpression(_)
        | InstructionValue::BinaryExpression(_)
        | InstructionValue::PropertyLoad(_)
        | InstructionValue::StoreGlobal(_) => false,
        InstructionValue::TaggedTemplateExpression(_)
        | InstructionValue::CallExpression(_)
        | InstructionValue::MethodCall(_) => {
            instruction.lvalue.identifier.type_ != crate::hir::types::Type::Primitive
        }
        InstructionValue::RegExpLiteral(_)
        | InstructionValue::PropertyStore(_)
        | InstructionValue::ComputedStore(_)
        | InstructionValue::ArrayExpression(_)
        | InstructionValue::JsxExpression(_)
        | InstructionValue::JsxFragment(_)
        | InstructionValue::NewExpression(_)
        | InstructionValue::ObjectExpression(_)
        | InstructionValue::UnsupportedNode(_)
        | InstructionValue::ObjectMethod(_)
        | InstructionValue::FunctionExpression(_) => true,
    }
}

/// Check if a destructuring pattern contains a spread element.
fn does_pattern_contain_spread(pattern: &crate::hir::Pattern) -> bool {
    match pattern {
        crate::hir::Pattern::Array(arr) => {
            arr.items.iter().any(|item| matches!(item, crate::hir::ArrayPatternElement::Spread(_)))
        }
        crate::hir::Pattern::Object(obj) => obj
            .properties
            .iter()
            .any(|prop| matches!(prop, crate::hir::ObjectPatternProperty::Spread(_))),
    }
}
