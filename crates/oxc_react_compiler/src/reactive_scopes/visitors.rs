/// Visitor infrastructure for ReactiveFunction.
///
/// Port of `ReactiveScopes/visitors.ts` from the React Compiler.
///
/// Provides traversal and transformation patterns for the reactive function tree.
use crate::hir::{
    ReactiveBlock, ReactiveFunction, ReactiveInstruction, ReactiveStatement,
    ReactiveTerminal, ReactiveTerminalStatement,
};

/// Visit all statements in a reactive block.
pub fn visit_reactive_block(block: &ReactiveBlock, visitor: &mut impl ReactiveVisitor) {
    for stmt in block {
        visit_reactive_statement(stmt, visitor);
    }
}

/// Visit a single reactive statement.
pub fn visit_reactive_statement(stmt: &ReactiveStatement, visitor: &mut impl ReactiveVisitor) {
    match stmt {
        ReactiveStatement::Instruction(instr_stmt) => {
            visitor.visit_instruction(&instr_stmt.instruction);
        }
        ReactiveStatement::Terminal(term_stmt) => {
            visitor.visit_terminal(term_stmt);
            visit_terminal_children(&term_stmt.terminal, visitor);
        }
        ReactiveStatement::Scope(scope_block) => {
            visitor.visit_scope_block(&scope_block.scope);
            visit_reactive_block(&scope_block.instructions, visitor);
        }
        ReactiveStatement::PrunedScope(pruned_block) => {
            visitor.visit_pruned_scope_block(&pruned_block.scope);
            visit_reactive_block(&pruned_block.instructions, visitor);
        }
    }
}

/// Visit children blocks within a reactive terminal.
fn visit_terminal_children(terminal: &ReactiveTerminal, visitor: &mut impl ReactiveVisitor) {
    match terminal {
        ReactiveTerminal::If(t) => {
            visit_reactive_block(&t.consequent, visitor);
            if let Some(alt) = &t.alternate {
                visit_reactive_block(alt, visitor);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &t.cases {
                if let Some(block) = &case.block {
                    visit_reactive_block(block, visitor);
                }
            }
        }
        ReactiveTerminal::While(t) => {
            visit_reactive_block(&t.r#loop, visitor);
        }
        ReactiveTerminal::DoWhile(t) => {
            visit_reactive_block(&t.r#loop, visitor);
        }
        ReactiveTerminal::For(t) => {
            visit_reactive_block(&t.r#loop, visitor);
        }
        ReactiveTerminal::ForOf(t) => {
            visit_reactive_block(&t.r#loop, visitor);
        }
        ReactiveTerminal::ForIn(t) => {
            visit_reactive_block(&t.r#loop, visitor);
        }
        ReactiveTerminal::Label(t) => {
            visit_reactive_block(&t.block, visitor);
        }
        ReactiveTerminal::Try(t) => {
            visit_reactive_block(&t.block, visitor);
            visit_reactive_block(&t.handler, visitor);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}

/// Trait for visiting reactive function nodes.
pub trait ReactiveVisitor {
    fn visit_instruction(&mut self, _instr: &ReactiveInstruction) {}
    fn visit_terminal(&mut self, _stmt: &ReactiveTerminalStatement) {}
    fn visit_scope_block(&mut self, _scope: &crate::hir::ReactiveScope) {}
    fn visit_pruned_scope_block(&mut self, _scope: &crate::hir::ReactiveScope) {}
}

/// Visit a reactive function.
pub fn visit_reactive_function(func: &ReactiveFunction, visitor: &mut impl ReactiveVisitor) {
    visit_reactive_block(&func.body, visitor);
}

/// Mutable transform of a reactive block — allows replacing statements.
pub fn transform_reactive_block(
    block: &mut ReactiveBlock,
    transform: &mut impl ReactiveTransform,
) {
    let mut i = 0;
    while i < block.len() {
        let result = transform.transform_statement(&block[i]);
        match result {
            TransformResult::Keep => {
                // Also transform children
                transform_statement_children(&mut block[i], transform);
                i += 1;
            }
            TransformResult::Remove => {
                block.remove(i);
            }
            TransformResult::Replace(replacement) => {
                block[i] = replacement;
                transform_statement_children(&mut block[i], transform);
                i += 1;
            }
            TransformResult::ReplaceMany(replacements) => {
                block.remove(i);
                let count = replacements.len();
                for (j, stmt) in replacements.into_iter().enumerate() {
                    block.insert(i + j, stmt);
                }
                i += count;
            }
        }
    }
}

fn transform_statement_children(
    stmt: &mut ReactiveStatement,
    transform: &mut impl ReactiveTransform,
) {
    match stmt {
        ReactiveStatement::Terminal(term) => {
            transform_terminal_children(&mut term.terminal, transform);
        }
        ReactiveStatement::Scope(scope) => {
            transform_reactive_block(&mut scope.instructions, transform);
        }
        ReactiveStatement::PrunedScope(scope) => {
            transform_reactive_block(&mut scope.instructions, transform);
        }
        ReactiveStatement::Instruction(_) => {}
    }
}

fn transform_terminal_children(
    terminal: &mut ReactiveTerminal,
    transform: &mut impl ReactiveTransform,
) {
    match terminal {
        ReactiveTerminal::If(t) => {
            transform_reactive_block(&mut t.consequent, transform);
            if let Some(alt) = &mut t.alternate {
                transform_reactive_block(alt, transform);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &mut t.cases {
                if let Some(block) = &mut case.block {
                    transform_reactive_block(block, transform);
                }
            }
        }
        ReactiveTerminal::While(t) => {
            transform_reactive_block(&mut t.r#loop, transform);
        }
        ReactiveTerminal::DoWhile(t) => {
            transform_reactive_block(&mut t.r#loop, transform);
        }
        ReactiveTerminal::For(t) => {
            transform_reactive_block(&mut t.r#loop, transform);
        }
        ReactiveTerminal::ForOf(t) => {
            transform_reactive_block(&mut t.r#loop, transform);
        }
        ReactiveTerminal::ForIn(t) => {
            transform_reactive_block(&mut t.r#loop, transform);
        }
        ReactiveTerminal::Label(t) => {
            transform_reactive_block(&mut t.block, transform);
        }
        ReactiveTerminal::Try(t) => {
            transform_reactive_block(&mut t.block, transform);
            transform_reactive_block(&mut t.handler, transform);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}

/// Result of transforming a statement.
pub enum TransformResult {
    Keep,
    Remove,
    Replace(ReactiveStatement),
    ReplaceMany(Vec<ReactiveStatement>),
}

/// Trait for transforming reactive function statements.
pub trait ReactiveTransform {
    fn transform_statement(&mut self, _stmt: &ReactiveStatement) -> TransformResult {
        TransformResult::Keep
    }
}
