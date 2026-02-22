/// Rename variables to ensure unique names in the output.
///
/// Port of `ReactiveScopes/RenameVariables.ts` from the React Compiler.
///
/// Ensures that each named variable has a unique name that does not conflict
/// with any other variables in the same block scope.
use rustc_hash::FxHashSet;

use crate::hir::{
    HIRFunction, InstructionValue, ReactiveBlock, ReactiveFunction, ReactiveStatement,
    ReactiveTerminal, ReactiveValue,
};

/// Rename variables in the reactive function to ensure uniqueness.
///
/// Returns the set of all unique variable names after renaming.
pub fn rename_variables(func: &ReactiveFunction) -> FxHashSet<String> {
    // Seed with all globally-referenced names (matches TS: collectReferencedGlobals(fn))
    let mut used_names = collect_referenced_globals(func);
    // Then add all locally declared names
    collect_names_from_block(&func.body, &mut used_names);
    used_names
}

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
    for block in func.body.blocks.values() {
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

fn collect_names_from_block(block: &ReactiveBlock, names: &mut FxHashSet<String>) {
    for stmt in block {
        match stmt {
            ReactiveStatement::Instruction(instr) => {
                if let Some(ref place) = instr.instruction.lvalue
                    && let Some(name) = &place.identifier.name
                {
                    let name_str = name.value().to_string();
                    names.insert(name_str);
                }
            }
            ReactiveStatement::Terminal(term) => {
                collect_names_from_terminal(&term.terminal, names);
            }
            ReactiveStatement::Scope(scope) => {
                collect_names_from_block(&scope.instructions, names);
            }
            ReactiveStatement::PrunedScope(scope) => {
                collect_names_from_block(&scope.instructions, names);
            }
        }
    }
}

fn collect_names_from_terminal(terminal: &ReactiveTerminal, names: &mut FxHashSet<String>) {
    match terminal {
        ReactiveTerminal::If(t) => {
            collect_names_from_block(&t.consequent, names);
            if let Some(alt) = &t.alternate {
                collect_names_from_block(alt, names);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &t.cases {
                if let Some(block) = &case.block {
                    collect_names_from_block(block, names);
                }
            }
        }
        ReactiveTerminal::While(t) => collect_names_from_block(&t.r#loop, names),
        ReactiveTerminal::DoWhile(t) => collect_names_from_block(&t.r#loop, names),
        ReactiveTerminal::For(t) => collect_names_from_block(&t.r#loop, names),
        ReactiveTerminal::ForOf(t) => collect_names_from_block(&t.r#loop, names),
        ReactiveTerminal::ForIn(t) => collect_names_from_block(&t.r#loop, names),
        ReactiveTerminal::Label(t) => collect_names_from_block(&t.block, names),
        ReactiveTerminal::Try(t) => {
            collect_names_from_block(&t.block, names);
            collect_names_from_block(&t.handler, names);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}
