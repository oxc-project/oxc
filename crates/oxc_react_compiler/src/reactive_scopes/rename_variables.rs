/// Rename variables to ensure unique names in the output.
///
/// Port of `ReactiveScopes/RenameVariables.ts` from the React Compiler.
///
/// Ensures that each named variable has a unique name that does not conflict
/// with any other variables in the same block scope. Note that the scoping is
/// based on the final inferred blocks, not the block scopes that were present
/// in the original source.
///
/// Variables are renamed using their original name followed by a number,
/// starting with 0 and incrementing until a unique name is found. For temporary
/// values that are promoted to named variables, the starting name is "T0" for
/// values that appear in JSX tag position and "t0" otherwise.
///
/// Returns a Set of all the unique variable names in the function after renaming.
use rustc_hash::{FxHashMap, FxHashSet};

use crate::hir::{
    DeclarationId, HIRFunction, Identifier, IdentifierName, InstructionValue, ReactiveBlock,
    ReactiveFunction, ReactiveInstruction, ReactiveParam, ReactiveStatement, ReactiveTerminal,
    ReactiveValue, hir_builder::compute_rpo_order,
};

// =====================================================================================
// Public API
// =====================================================================================

/// Rename variables in the reactive function to ensure uniqueness.
///
/// Mutates `identifier.name` fields in place so every named variable
/// has a unique name that does not conflict with any other variable in
/// the same block scope.
///
/// Returns the set of all unique variable names after renaming.
pub fn rename_variables(func: &mut ReactiveFunction) -> FxHashSet<String> {
    let globals = collect_referenced_globals(func);
    let mut scopes = Scopes::new(globals);
    rename_variables_impl(func, &mut scopes);
    let mut result = scopes.names;
    for g in &scopes.globals {
        result.insert(g.clone());
    }
    result
}

fn rename_variables_impl(func: &mut ReactiveFunction, scopes: &mut Scopes) {
    scopes.enter(|scopes| {
        for param in &mut func.params {
            match param {
                ReactiveParam::Place(place) => {
                    scopes.visit(&mut place.identifier);
                }
                ReactiveParam::Spread(spread) => {
                    scopes.visit(&mut spread.place.identifier);
                }
            }
        }
        visit_block(&mut func.body, scopes);
    });
}

// =====================================================================================
// Scopes
// =====================================================================================

struct Scopes {
    /// Memoize renamed identifiers by declaration id.
    seen: FxHashMap<DeclarationId, IdentifierName>,
    /// Scope stack for name tracking. Each scope maps name -> declaration id.
    stack: Vec<FxHashMap<String, DeclarationId>>,
    /// Collected global names.
    globals: FxHashSet<String>,
    /// All assigned names (returned at the end).
    names: FxHashSet<String>,
}

impl Scopes {
    fn new(globals: FxHashSet<String>) -> Self {
        Self {
            seen: FxHashMap::default(),
            stack: vec![FxHashMap::default()],
            globals,
            names: FxHashSet::default(),
        }
    }

    fn visit(&mut self, identifier: &mut Identifier) {
        // In TypeScript, identifiers are reference types — renaming one reference
        // renames all others with the same declaration_id automatically. In Rust,
        // identifiers are cloned values, so we must propagate renames explicitly.
        // Always check `seen` first: if another copy of this identifier (same
        // declaration_id) was already renamed, apply that name to this copy too.
        if let Some(mapped_name) = self.seen.get(&identifier.declaration_id) {
            identifier.name = Some(mapped_name.clone());
            return;
        }

        let original_name = match &identifier.name {
            Some(name) => name.clone(),
            None => return,
        };

        let original_value = original_name.value().to_string();
        let is_promoted_temp = is_promoted_temporary(&original_value);
        let is_promoted_jsx_temp = is_promoted_jsx_temporary(&original_value);

        let mut id: u32 = 0;
        let mut name = if is_promoted_temp {
            let n = format!("t{id}");
            id += 1;
            n
        } else if is_promoted_jsx_temp {
            let n = format!("T{id}");
            id += 1;
            n
        } else {
            original_value.clone()
        };

        while self.lookup(&name).is_some() || self.globals.contains(&name) {
            if is_promoted_temp {
                name = format!("t{id}");
            } else if is_promoted_jsx_temp {
                name = format!("T{id}");
            } else {
                name = format!("{original_value}${id}");
            }
            id += 1;
        }

        let identifier_name = IdentifierName::Named(name.clone());
        identifier.name = Some(identifier_name.clone());
        self.seen.insert(identifier.declaration_id, identifier_name);
        if let Some(top) = self.stack.last_mut() {
            top.insert(name.clone(), identifier.declaration_id);
        }
        self.names.insert(name);
    }

    fn lookup(&self, name: &str) -> Option<DeclarationId> {
        for scope in self.stack.iter().rev() {
            if let Some(&decl_id) = scope.get(name) {
                return Some(decl_id);
            }
        }
        None
    }

    fn enter(&mut self, f: impl FnOnce(&mut Self)) {
        self.stack.push(FxHashMap::default());
        f(self);
        self.stack.pop();
    }
}

/// Check if the name is a promoted temporary (starts with `#t`).
fn is_promoted_temporary(name: &str) -> bool {
    name.starts_with("#t")
}

/// Check if the name is a promoted JSX temporary (starts with `#T`).
fn is_promoted_jsx_temporary(name: &str) -> bool {
    name.starts_with("#T")
}

// =====================================================================================
// Visitor — walks the reactive tree mutably
// =====================================================================================

fn visit_block(block: &mut ReactiveBlock, scopes: &mut Scopes) {
    scopes.enter(|scopes| {
        traverse_block(block, scopes);
    });
}

fn traverse_block(block: &mut ReactiveBlock, scopes: &mut Scopes) {
    for stmt in block.iter_mut() {
        match stmt {
            ReactiveStatement::Instruction(instr_stmt) => {
                visit_instruction(&mut instr_stmt.instruction, scopes);
            }
            ReactiveStatement::Terminal(term_stmt) => {
                visit_terminal(&mut term_stmt.terminal, scopes);
            }
            ReactiveStatement::Scope(scope_block) => {
                visit_scope(scope_block, scopes);
            }
            ReactiveStatement::PrunedScope(pruned_block) => {
                visit_pruned_scope(pruned_block, scopes);
            }
        }
    }
}

fn visit_scope(scope_block: &mut crate::hir::ReactiveScopeBlock, scopes: &mut Scopes) {
    // Visit scope declarations first (matches TS visitScope).
    // In TypeScript, `scope.declarations` is a Map that preserves insertion order.
    // Our Rust IndexMap also preserves insertion order, so we iterate directly
    // (matching the TS behavior where Map iterates in insertion order).
    for declaration in scope_block.scope.declarations.values_mut() {
        scopes.visit(&mut declaration.identifier);
    }

    // In TS, identifiers are reference types, so renaming a declaration's identifier
    // automatically updates all other references to the same identifier (including in
    // scope.dependencies, scope.reassignments, and scope.early_return_value). In Rust,
    // identifiers are cloned values, so we must explicitly visit them all.
    let old_deps: Vec<_> = scope_block.scope.dependencies.drain().collect();
    for mut dep in old_deps {
        scopes.visit(&mut dep.identifier);
        scope_block.scope.dependencies.insert(dep);
    }
    for reassignment in &mut scope_block.scope.reassignments {
        scopes.visit(reassignment);
    }
    // The early_return_value identifier is a clone of the declaration identifier;
    // update it so codegen emits the correct (renamed) variable name.
    if let Some(ref mut early_return) = scope_block.scope.early_return_value {
        scopes.visit(&mut early_return.value);
    }

    visit_block(&mut scope_block.instructions, scopes);
}

fn visit_pruned_scope(
    pruned_block: &mut crate::hir::PrunedReactiveScopeBlock,
    scopes: &mut Scopes,
) {
    // TS: traverseBlock(scopeBlock.instructions, state) — no enter, no declarations visit
    traverse_block(&mut pruned_block.instructions, scopes);
}

fn visit_instruction(instr: &mut ReactiveInstruction, scopes: &mut Scopes) {
    // Visit lvalue (matches TS: eachInstructionLValue which yields instr.lvalue + value lvalues)
    if let Some(ref mut lvalue) = instr.lvalue {
        scopes.visit(&mut lvalue.identifier);
    }
    visit_instruction_value_lvalues(&mut instr.value, scopes);

    // Visit value (matches TS: this.visitValue(instruction.id, instruction.value, state))
    visit_value(&mut instr.value, scopes);
}

/// Visit lvalues embedded in instruction values (DeclareLocal, StoreLocal, etc.)
fn visit_instruction_value_lvalues(value: &mut ReactiveValue, scopes: &mut Scopes) {
    if let ReactiveValue::Instruction(inner) = value {
        match inner.as_mut() {
            InstructionValue::DeclareLocal(v) => {
                scopes.visit(&mut v.lvalue.place.identifier);
            }
            InstructionValue::DeclareContext(v) => {
                scopes.visit(&mut v.lvalue_place.identifier);
            }
            InstructionValue::StoreLocal(v) => {
                scopes.visit(&mut v.lvalue.place.identifier);
            }
            InstructionValue::StoreContext(v) => {
                scopes.visit(&mut v.lvalue_place.identifier);
            }
            InstructionValue::Destructure(v) => {
                visit_pattern_lvalues(&mut v.lvalue.pattern, scopes);
            }
            InstructionValue::PrefixUpdate(v) => {
                scopes.visit(&mut v.lvalue.identifier);
            }
            InstructionValue::PostfixUpdate(v) => {
                scopes.visit(&mut v.lvalue.identifier);
            }
            _ => {}
        }
    }
}

fn visit_pattern_lvalues(pattern: &mut crate::hir::Pattern, scopes: &mut Scopes) {
    match pattern {
        crate::hir::Pattern::Array(arr) => {
            for item in &mut arr.items {
                match item {
                    crate::hir::ArrayPatternElement::Place(p) => {
                        scopes.visit(&mut p.identifier);
                    }
                    crate::hir::ArrayPatternElement::Spread(s) => {
                        scopes.visit(&mut s.place.identifier);
                    }
                    crate::hir::ArrayPatternElement::Hole => {}
                }
            }
        }
        crate::hir::Pattern::Object(obj) => {
            for prop in &mut obj.properties {
                match prop {
                    crate::hir::ObjectPatternProperty::Property(p) => {
                        scopes.visit(&mut p.place.identifier);
                    }
                    crate::hir::ObjectPatternProperty::Spread(s) => {
                        scopes.visit(&mut s.place.identifier);
                    }
                }
            }
        }
    }
}

/// Visit a reactive value — handles composite values (Logical, Ternary, Sequence,
/// OptionalCall) and leaf instruction values.
///
/// Matches the TS `visitValue` + `traverseValue` pattern, plus the override in
/// RenameVariables' Visitor that handles FunctionExpression/ObjectMethod.
fn visit_value(value: &mut ReactiveValue, scopes: &mut Scopes) {
    // First: traverseValue — recurse into composite values
    traverse_value(value, scopes);

    // Then: the RenameVariables Visitor override visits HIR functions for
    // FunctionExpression/ObjectMethod
    if let ReactiveValue::Instruction(inner) = value {
        match inner.as_mut() {
            InstructionValue::FunctionExpression(v) => {
                visit_hir_function(&mut v.lowered_func.func, scopes);
            }
            InstructionValue::ObjectMethod(v) => {
                visit_hir_function(&mut v.lowered_func.func, scopes);
            }
            _ => {}
        }
    }
}

fn traverse_value(value: &mut ReactiveValue, scopes: &mut Scopes) {
    match value {
        ReactiveValue::OptionalCall(v) => {
            visit_value(&mut v.value, scopes);
        }
        ReactiveValue::Logical(v) => {
            visit_value(&mut v.left, scopes);
            visit_value(&mut v.right, scopes);
        }
        ReactiveValue::Ternary(v) => {
            visit_value(&mut v.test, scopes);
            visit_value(&mut v.consequent, scopes);
            visit_value(&mut v.alternate, scopes);
        }
        ReactiveValue::Sequence(v) => {
            for instr in &mut v.instructions {
                visit_instruction(instr, scopes);
            }
            visit_value(&mut v.value, scopes);
        }
        ReactiveValue::Instruction(inner) => {
            // Leaf instruction value — visit all operand places
            visit_instruction_value_operands(inner.as_mut(), scopes);
        }
    }
}

/// Visit all operand places in an instruction value (the "reads" / operands).
fn visit_instruction_value_operands(value: &mut InstructionValue, scopes: &mut Scopes) {
    match value {
        InstructionValue::CallExpression(v) => {
            scopes.visit(&mut v.callee.identifier);
            visit_call_args(&mut v.args, scopes);
        }
        InstructionValue::NewExpression(v) => {
            scopes.visit(&mut v.callee.identifier);
            visit_call_args(&mut v.args, scopes);
        }
        InstructionValue::MethodCall(v) => {
            scopes.visit(&mut v.receiver.identifier);
            scopes.visit(&mut v.property.identifier);
            visit_call_args(&mut v.args, scopes);
        }
        InstructionValue::BinaryExpression(v) => {
            scopes.visit(&mut v.left.identifier);
            scopes.visit(&mut v.right.identifier);
        }
        InstructionValue::UnaryExpression(v) => {
            scopes.visit(&mut v.value.identifier);
        }
        InstructionValue::LoadLocal(v) => {
            scopes.visit(&mut v.place.identifier);
        }
        InstructionValue::LoadContext(v) => {
            scopes.visit(&mut v.place.identifier);
        }
        InstructionValue::StoreLocal(v) => {
            scopes.visit(&mut v.value.identifier);
        }
        InstructionValue::StoreContext(v) => {
            scopes.visit(&mut v.lvalue_place.identifier);
            scopes.visit(&mut v.value.identifier);
        }
        InstructionValue::StoreGlobal(v) => {
            scopes.visit(&mut v.value.identifier);
        }
        InstructionValue::Destructure(v) => {
            scopes.visit(&mut v.value.identifier);
        }
        InstructionValue::PropertyLoad(v) => {
            scopes.visit(&mut v.object.identifier);
        }
        InstructionValue::PropertyStore(v) => {
            scopes.visit(&mut v.object.identifier);
            scopes.visit(&mut v.value.identifier);
        }
        InstructionValue::PropertyDelete(v) => {
            scopes.visit(&mut v.object.identifier);
        }
        InstructionValue::ComputedLoad(v) => {
            scopes.visit(&mut v.object.identifier);
            scopes.visit(&mut v.property.identifier);
        }
        InstructionValue::ComputedStore(v) => {
            scopes.visit(&mut v.object.identifier);
            scopes.visit(&mut v.property.identifier);
            scopes.visit(&mut v.value.identifier);
        }
        InstructionValue::ComputedDelete(v) => {
            scopes.visit(&mut v.object.identifier);
            scopes.visit(&mut v.property.identifier);
        }
        InstructionValue::TypeCastExpression(v) => {
            scopes.visit(&mut v.value.identifier);
        }
        InstructionValue::JsxExpression(v) => {
            if let crate::hir::JsxTag::Place(p) = &mut v.tag {
                scopes.visit(&mut p.identifier);
            }
            for attr in &mut v.props {
                match attr {
                    crate::hir::JsxAttribute::Attribute { place, .. } => {
                        scopes.visit(&mut place.identifier);
                    }
                    crate::hir::JsxAttribute::Spread { argument } => {
                        scopes.visit(&mut argument.identifier);
                    }
                }
            }
            if let Some(children) = &mut v.children {
                for child in children {
                    scopes.visit(&mut child.identifier);
                }
            }
        }
        InstructionValue::JsxFragment(v) => {
            for child in &mut v.children {
                scopes.visit(&mut child.identifier);
            }
        }
        InstructionValue::ObjectExpression(v) => {
            for prop in &mut v.properties {
                match prop {
                    crate::hir::ObjectPatternProperty::Property(p) => {
                        if let crate::hir::ObjectPropertyKey::Computed(place) = &mut p.key {
                            scopes.visit(&mut place.identifier);
                        }
                        scopes.visit(&mut p.place.identifier);
                    }
                    crate::hir::ObjectPatternProperty::Spread(s) => {
                        scopes.visit(&mut s.place.identifier);
                    }
                }
            }
        }
        InstructionValue::ArrayExpression(v) => {
            for elem in &mut v.elements {
                match elem {
                    crate::hir::ArrayExpressionElement::Place(p) => {
                        scopes.visit(&mut p.identifier);
                    }
                    crate::hir::ArrayExpressionElement::Spread(s) => {
                        scopes.visit(&mut s.place.identifier);
                    }
                    crate::hir::ArrayExpressionElement::Hole => {}
                }
            }
        }
        InstructionValue::FunctionExpression(v) => {
            for ctx in &mut v.lowered_func.func.context {
                scopes.visit(&mut ctx.identifier);
            }
        }
        InstructionValue::ObjectMethod(v) => {
            for ctx in &mut v.lowered_func.func.context {
                scopes.visit(&mut ctx.identifier);
            }
        }
        InstructionValue::TaggedTemplateExpression(v) => {
            scopes.visit(&mut v.tag.identifier);
        }
        InstructionValue::TemplateLiteral(v) => {
            for subexpr in &mut v.subexprs {
                scopes.visit(&mut subexpr.identifier);
            }
        }
        InstructionValue::Await(v) => {
            scopes.visit(&mut v.value.identifier);
        }
        InstructionValue::GetIterator(v) => {
            scopes.visit(&mut v.collection.identifier);
        }
        InstructionValue::IteratorNext(v) => {
            scopes.visit(&mut v.iterator.identifier);
            scopes.visit(&mut v.collection.identifier);
        }
        InstructionValue::NextPropertyOf(v) => {
            scopes.visit(&mut v.value.identifier);
        }
        InstructionValue::PrefixUpdate(v) => {
            scopes.visit(&mut v.value.identifier);
        }
        InstructionValue::PostfixUpdate(v) => {
            scopes.visit(&mut v.value.identifier);
        }
        InstructionValue::StartMemoize(v) => {
            if let Some(deps) = &mut v.deps {
                for dep in deps {
                    if let crate::hir::ManualMemoDependencyRoot::NamedLocal { value, .. } =
                        &mut dep.root
                    {
                        scopes.visit(&mut value.identifier);
                    }
                }
            }
        }
        InstructionValue::FinishMemoize(v) => {
            scopes.visit(&mut v.decl.identifier);
        }
        InstructionValue::LoadGlobal(_)
        | InstructionValue::MetaProperty(_)
        | InstructionValue::RegExpLiteral(_)
        | InstructionValue::Primitive(_)
        | InstructionValue::JsxText(_)
        | InstructionValue::DeclareLocal(_)
        | InstructionValue::DeclareContext(_)
        | InstructionValue::Debugger(_)
        | InstructionValue::UnsupportedNode(_) => {}
    }
}

fn visit_call_args(args: &mut [crate::hir::CallArg], scopes: &mut Scopes) {
    for arg in args {
        match arg {
            crate::hir::CallArg::Place(p) => {
                scopes.visit(&mut p.identifier);
            }
            crate::hir::CallArg::Spread(s) => {
                scopes.visit(&mut s.place.identifier);
            }
        }
    }
}

/// Visit a reactive terminal — visits operand places and recurses into child blocks.
fn visit_terminal(terminal: &mut ReactiveTerminal, scopes: &mut Scopes) {
    match terminal {
        ReactiveTerminal::Break(_) | ReactiveTerminal::Continue(_) => {}
        ReactiveTerminal::Return(t) => {
            scopes.visit(&mut t.value.identifier);
        }
        ReactiveTerminal::Throw(t) => {
            scopes.visit(&mut t.value.identifier);
        }
        ReactiveTerminal::For(t) => {
            visit_value(&mut t.init, scopes);
            visit_value(&mut t.test, scopes);
            visit_block(&mut t.r#loop, scopes);
            if let Some(update) = &mut t.update {
                visit_value(update, scopes);
            }
        }
        ReactiveTerminal::ForOf(t) => {
            visit_value(&mut t.init, scopes);
            visit_value(&mut t.test, scopes);
            visit_block(&mut t.r#loop, scopes);
        }
        ReactiveTerminal::ForIn(t) => {
            visit_value(&mut t.init, scopes);
            visit_block(&mut t.r#loop, scopes);
        }
        ReactiveTerminal::DoWhile(t) => {
            visit_block(&mut t.r#loop, scopes);
            visit_value(&mut t.test, scopes);
        }
        ReactiveTerminal::While(t) => {
            visit_value(&mut t.test, scopes);
            visit_block(&mut t.r#loop, scopes);
        }
        ReactiveTerminal::If(t) => {
            scopes.visit(&mut t.test.identifier);
            visit_block(&mut t.consequent, scopes);
            if let Some(alt) = &mut t.alternate {
                visit_block(alt, scopes);
            }
        }
        ReactiveTerminal::Switch(t) => {
            scopes.visit(&mut t.test.identifier);
            for case in &mut t.cases {
                if let Some(ref mut test) = case.test {
                    scopes.visit(&mut test.identifier);
                }
                if let Some(ref mut block) = case.block {
                    visit_block(block, scopes);
                }
            }
        }
        ReactiveTerminal::Label(t) => {
            visit_block(&mut t.block, scopes);
        }
        ReactiveTerminal::Try(t) => {
            visit_block(&mut t.block, scopes);
            if let Some(ref mut binding) = t.handler_binding {
                scopes.visit(&mut binding.identifier);
            }
            visit_block(&mut t.handler, scopes);
        }
    }
}

/// Visit a HIR function (used for FunctionExpression / ObjectMethod lowered functions).
///
/// Matches TS `visitHirFunction`: visits params, then for each block visits
/// instructions (including recursing into nested functions), then visits
/// terminal operands.
fn visit_hir_function(func: &mut HIRFunction, scopes: &mut Scopes) {
    // Save the outer function's `seen` map and replace it with one seeded from
    // the context (captured) variables only.
    //
    // In TypeScript, identifiers are reference types — renaming one reference
    // renames all references to the same object. In Rust, identifiers are cloned
    // values with DeclarationIds that may coincide between outer and inner
    // functions (due to shared environment counters during build_hir). Using the
    // outer function's full `seen` map would cause the inner function's own
    // identifiers to be incorrectly renamed to whatever the outer function's
    // same-DeclarationId identifier was renamed to.
    //
    // However, context (captured) variables DO need to inherit their rename from
    // the outer function — they share the same DeclarationId intentionally, and
    // references to them inside the inner function body must use the same name.
    // So we seed the fresh `seen` map with just the context variable entries.
    let saved_seen = std::mem::take(&mut scopes.seen);
    for ctx in &func.context {
        if let Some(mapped_name) = saved_seen.get(&ctx.identifier.declaration_id) {
            scopes.seen.insert(ctx.identifier.declaration_id, mapped_name.clone());
        }
    }
    for param in &mut func.params {
        match param {
            ReactiveParam::Place(place) => {
                scopes.visit(&mut place.identifier);
            }
            ReactiveParam::Spread(spread) => {
                scopes.visit(&mut spread.place.identifier);
            }
        }
    }
    let block_ids = compute_rpo_order(func.body.entry, &func.body.blocks);
    for block_id in &block_ids {
        let Some(block) = func.body.blocks.get_mut(block_id) else {
            continue;
        };
        for instr in &mut block.instructions {
            visit_hir_instruction(instr, scopes);
            match &mut instr.value {
                InstructionValue::FunctionExpression(v) => {
                    visit_hir_function(&mut v.lowered_func.func, scopes);
                }
                InstructionValue::ObjectMethod(v) => {
                    visit_hir_function(&mut v.lowered_func.func, scopes);
                }
                _ => {}
            }
        }
        visit_hir_terminal_operands(&mut block.terminal, scopes);
    }
    // Restore the outer function's `seen` map.
    scopes.seen = saved_seen;
}

/// Visit a HIR instruction — visits lvalue then operands.
fn visit_hir_instruction(instr: &mut crate::hir::Instruction, scopes: &mut Scopes) {
    // Visit lvalue
    scopes.visit(&mut instr.lvalue.identifier);
    // Visit value lvalues
    visit_hir_instruction_value_lvalues(&mut instr.value, scopes);
    // Visit operands
    visit_instruction_value_operands(&mut instr.value, scopes);
}

fn visit_hir_instruction_value_lvalues(value: &mut InstructionValue, scopes: &mut Scopes) {
    match value {
        InstructionValue::DeclareLocal(v) => {
            scopes.visit(&mut v.lvalue.place.identifier);
        }
        InstructionValue::DeclareContext(v) => {
            scopes.visit(&mut v.lvalue_place.identifier);
        }
        InstructionValue::StoreLocal(v) => {
            scopes.visit(&mut v.lvalue.place.identifier);
        }
        InstructionValue::StoreContext(v) => {
            scopes.visit(&mut v.lvalue_place.identifier);
        }
        InstructionValue::Destructure(v) => {
            visit_pattern_lvalues(&mut v.lvalue.pattern, scopes);
        }
        InstructionValue::PrefixUpdate(v) => {
            scopes.visit(&mut v.lvalue.identifier);
        }
        InstructionValue::PostfixUpdate(v) => {
            scopes.visit(&mut v.lvalue.identifier);
        }
        _ => {}
    }
}

/// Visit terminal operands in the HIR (not reactive terminals).
fn visit_hir_terminal_operands(terminal: &mut crate::hir::Terminal, scopes: &mut Scopes) {
    match terminal {
        crate::hir::Terminal::Throw(t) => {
            scopes.visit(&mut t.value.identifier);
        }
        crate::hir::Terminal::Return(t) => {
            scopes.visit(&mut t.value.identifier);
        }
        crate::hir::Terminal::If(t) => {
            scopes.visit(&mut t.test.identifier);
        }
        crate::hir::Terminal::Branch(t) => {
            scopes.visit(&mut t.test.identifier);
        }
        crate::hir::Terminal::Switch(t) => {
            scopes.visit(&mut t.test.identifier);
            for case in &mut t.cases {
                if let Some(ref mut test) = case.test {
                    scopes.visit(&mut test.identifier);
                }
            }
        }
        crate::hir::Terminal::Try(t) => {
            if let Some(ref mut binding) = t.handler_binding {
                scopes.visit(&mut binding.identifier);
            }
        }
        crate::hir::Terminal::Unsupported(_)
        | crate::hir::Terminal::Unreachable(_)
        | crate::hir::Terminal::Goto(_)
        | crate::hir::Terminal::For(_)
        | crate::hir::Terminal::ForOf(_)
        | crate::hir::Terminal::ForIn(_)
        | crate::hir::Terminal::DoWhile(_)
        | crate::hir::Terminal::While(_)
        | crate::hir::Terminal::Logical(_)
        | crate::hir::Terminal::Ternary(_)
        | crate::hir::Terminal::Optional(_)
        | crate::hir::Terminal::Label(_)
        | crate::hir::Terminal::Sequence(_)
        | crate::hir::Terminal::MaybeThrow(_)
        | crate::hir::Terminal::Scope(_)
        | crate::hir::Terminal::PrunedScope(_) => {}
    }
}

// =====================================================================================
// Collect referenced globals
// =====================================================================================

/// Collect all globally-referenced names from the reactive function.
///
/// Port of `CollectReferencedGlobals.ts` — traverses the reactive tree
/// (including nested HIR functions) and collects every `LoadGlobal` binding name.
fn collect_referenced_globals(func: &ReactiveFunction) -> FxHashSet<String> {
    let mut names = FxHashSet::default();
    collect_globals_from_block(&func.body, &mut names);
    names
}

fn collect_globals_from_block(block: &ReactiveBlock, names: &mut FxHashSet<String>) {
    for stmt in block {
        match stmt {
            ReactiveStatement::Instruction(s) => {
                collect_globals_from_value(&s.instruction.value, names);
            }
            ReactiveStatement::Terminal(s) => {
                collect_globals_from_terminal(&s.terminal, names);
            }
            ReactiveStatement::Scope(s) => {
                collect_globals_from_block(&s.instructions, names);
            }
            ReactiveStatement::PrunedScope(s) => {
                collect_globals_from_block(&s.instructions, names);
            }
        }
    }
}

fn collect_globals_from_value(value: &ReactiveValue, names: &mut FxHashSet<String>) {
    match value {
        ReactiveValue::Instruction(inner) => match inner.as_ref() {
            InstructionValue::LoadGlobal(v) => {
                names.insert(v.binding.name().to_string());
            }
            InstructionValue::FunctionExpression(v) => {
                collect_globals_from_hir_function(&v.lowered_func.func, names);
            }
            InstructionValue::ObjectMethod(v) => {
                collect_globals_from_hir_function(&v.lowered_func.func, names);
            }
            _ => {}
        },
        ReactiveValue::Logical(v) => {
            collect_globals_from_value(&v.left, names);
            collect_globals_from_value(&v.right, names);
        }
        ReactiveValue::Ternary(v) => {
            collect_globals_from_value(&v.test, names);
            collect_globals_from_value(&v.consequent, names);
            collect_globals_from_value(&v.alternate, names);
        }
        ReactiveValue::Sequence(v) => {
            for instr in &v.instructions {
                collect_globals_from_value(&instr.value, names);
            }
            collect_globals_from_value(&v.value, names);
        }
        ReactiveValue::OptionalCall(v) => {
            collect_globals_from_value(&v.value, names);
        }
    }
}

fn collect_globals_from_hir_function(func: &HIRFunction, names: &mut FxHashSet<String>) {
    let block_ids = compute_rpo_order(func.body.entry, &func.body.blocks);
    for block_id in &block_ids {
        let Some(block) = func.body.blocks.get(block_id) else { continue };
        for instr in &block.instructions {
            match &instr.value {
                InstructionValue::LoadGlobal(v) => {
                    names.insert(v.binding.name().to_string());
                }
                InstructionValue::FunctionExpression(v) => {
                    collect_globals_from_hir_function(&v.lowered_func.func, names);
                }
                InstructionValue::ObjectMethod(v) => {
                    collect_globals_from_hir_function(&v.lowered_func.func, names);
                }
                _ => {}
            }
        }
    }
}

fn collect_globals_from_terminal(terminal: &ReactiveTerminal, names: &mut FxHashSet<String>) {
    match terminal {
        ReactiveTerminal::If(t) => {
            collect_globals_from_block(&t.consequent, names);
            if let Some(alt) = &t.alternate {
                collect_globals_from_block(alt, names);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &t.cases {
                if let Some(block) = &case.block {
                    collect_globals_from_block(block, names);
                }
            }
        }
        ReactiveTerminal::While(t) => {
            collect_globals_from_value(&t.test, names);
            collect_globals_from_block(&t.r#loop, names);
        }
        ReactiveTerminal::DoWhile(t) => {
            collect_globals_from_block(&t.r#loop, names);
            collect_globals_from_value(&t.test, names);
        }
        ReactiveTerminal::For(t) => {
            collect_globals_from_value(&t.init, names);
            collect_globals_from_value(&t.test, names);
            if let Some(update) = &t.update {
                collect_globals_from_value(update, names);
            }
            collect_globals_from_block(&t.r#loop, names);
        }
        ReactiveTerminal::ForOf(t) => {
            collect_globals_from_value(&t.init, names);
            collect_globals_from_value(&t.test, names);
            collect_globals_from_block(&t.r#loop, names);
        }
        ReactiveTerminal::ForIn(t) => {
            collect_globals_from_value(&t.init, names);
            collect_globals_from_block(&t.r#loop, names);
        }
        ReactiveTerminal::Label(t) => collect_globals_from_block(&t.block, names),
        ReactiveTerminal::Try(t) => {
            collect_globals_from_block(&t.block, names);
            collect_globals_from_block(&t.handler, names);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}
