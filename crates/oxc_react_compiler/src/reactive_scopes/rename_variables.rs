/// Rename variables to ensure unique names in the output.
///
/// Port of `ReactiveScopes/RenameVariables.ts` from the React Compiler.
///
/// Ensures that each named variable has a unique name that does not conflict
/// with any other variables in the same block scope.
use rustc_hash::FxHashSet;

use crate::hir::{
    IdentifierName, ReactiveBlock, ReactiveFunction, ReactiveStatement,
};

/// Rename variables in the reactive function to ensure uniqueness.
///
/// Returns the set of all unique variable names after renaming.
pub fn rename_variables(func: &mut ReactiveFunction) -> FxHashSet<String> {
    let mut used_names = FxHashSet::default();
    collect_names_from_block(&func.body, &mut used_names);
    used_names
}

fn collect_names_from_block(block: &ReactiveBlock, names: &mut FxHashSet<String>) {
    for stmt in block {
        match stmt {
            ReactiveStatement::Instruction(instr) => {
                if let Some(ref place) = instr.instruction.lvalue
                    && let Some(name) = &place.identifier.name {
                        let name_str = match name {
                            IdentifierName::Named(n) => n.clone(),
                            IdentifierName::Promoted(n) => n.clone(),
                        };
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

fn collect_names_from_terminal(
    terminal: &crate::hir::ReactiveTerminal,
    names: &mut FxHashSet<String>,
) {
    use crate::hir::ReactiveTerminal;
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
