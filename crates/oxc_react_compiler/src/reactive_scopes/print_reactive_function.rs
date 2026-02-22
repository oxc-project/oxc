/// Pretty-printer for reactive functions.
///
/// Port of `ReactiveScopes/PrintReactiveFunction.ts` from the React Compiler.
///
/// Produces a human-readable string representation of the reactive function tree,
/// useful for debugging and test output.
use crate::hir::{
    ReactiveBlock, ReactiveFunction, ReactiveInstruction, ReactiveScope, ReactiveStatement,
    ReactiveTerminal, ReactiveValue,
    print_hir::{print_instruction_value, print_place},
};

/// Print a reactive function to a string.
pub fn print_reactive_function(func: &ReactiveFunction) -> String {
    let mut writer = Writer::new();
    write_reactive_function(func, &mut writer);
    writer.complete()
}

/// Print a summary of a reactive scope (used in terminal printing).
pub fn print_reactive_scope_summary(scope: &ReactiveScope) -> String {
    let deps: Vec<String> =
        scope.dependencies.iter().map(|dep| format!("#{}", dep.identifier_id.0)).collect();
    let decls: Vec<String> = scope.declarations.keys().map(|id| format!("#{}", id.0)).collect();
    format!(
        "scope({}) deps=[{}] decls=[{}] range=[{}:{}]",
        scope.id.0,
        deps.join(", "),
        decls.join(", "),
        scope.range.start.0,
        scope.range.end.0,
    )
}

struct Writer {
    output: Vec<String>,
    indent: usize,
}

impl Writer {
    fn new() -> Self {
        Self { output: Vec::new(), indent: 0 }
    }

    fn write_line(&mut self, text: &str) {
        let indent = "  ".repeat(self.indent);
        self.output.push(format!("{indent}{text}"));
    }

    fn indented(&mut self, f: impl FnOnce(&mut Self)) {
        self.indent += 1;
        f(self);
        self.indent -= 1;
    }

    fn complete(self) -> String {
        self.output.join("\n")
    }
}

fn write_reactive_function(func: &ReactiveFunction, writer: &mut Writer) {
    let name = func.id.as_deref().unwrap_or("<unknown>");
    writer.write_line(&format!("function {name}("));
    writer.indented(|w| {
        for param in &func.params {
            match param {
                crate::hir::ReactiveParam::Place(p) => {
                    w.write_line(&format!("{},", print_place(p)));
                }
                crate::hir::ReactiveParam::Spread(s) => {
                    w.write_line(&format!("...{},", print_place(&s.place)));
                }
            }
        }
    });
    writer.write_line(") {");
    write_reactive_block(writer, &func.body);
    writer.write_line("}");
}

fn write_reactive_block(writer: &mut Writer, block: &ReactiveBlock) {
    writer.indented(|w| {
        for stmt in block {
            write_reactive_statement(w, stmt);
        }
    });
}

fn write_reactive_statement(writer: &mut Writer, stmt: &ReactiveStatement) {
    match stmt {
        ReactiveStatement::Instruction(instr) => {
            write_reactive_instruction(writer, &instr.instruction);
        }
        ReactiveStatement::Terminal(term) => {
            if let Some(label) = &term.label {
                writer
                    .write_line(&format!("label bb{} (implicit={}):", label.id.0, label.implicit));
            }
            write_reactive_terminal(writer, &term.terminal);
        }
        ReactiveStatement::Scope(scope) => {
            writer.write_line(&format!("scope {} {{", print_reactive_scope_summary(&scope.scope)));
            write_reactive_block(writer, &scope.instructions);
            writer.write_line("}");
        }
        ReactiveStatement::PrunedScope(scope) => {
            writer.write_line(&format!(
                "<pruned> scope {} {{",
                print_reactive_scope_summary(&scope.scope)
            ));
            write_reactive_block(writer, &scope.instructions);
            writer.write_line("}");
        }
    }
}

fn write_reactive_instruction(writer: &mut Writer, instr: &ReactiveInstruction) {
    let value = write_reactive_value(&instr.value);
    if let Some(lvalue) = &instr.lvalue {
        writer.write_line(&format!("[{}] {} = {value}", instr.id.0, print_place(lvalue)));
    } else {
        writer.write_line(&format!("[{}] {value}", instr.id.0));
    }
}

fn write_reactive_value(value: &ReactiveValue) -> String {
    match value {
        ReactiveValue::Instruction(instr) => print_instruction_value(instr),
        ReactiveValue::Logical(logical) => {
            let left = write_reactive_value(&logical.left);
            let right = write_reactive_value(&logical.right);
            format!("Logical {:?} ({left}) ({right})", logical.operator)
        }
        ReactiveValue::Ternary(ternary) => {
            let test = write_reactive_value(&ternary.test);
            let consequent = write_reactive_value(&ternary.consequent);
            let alternate = write_reactive_value(&ternary.alternate);
            format!("Ternary ({test}) ? ({consequent}) : ({alternate})")
        }
        ReactiveValue::Sequence(seq) => {
            let value = write_reactive_value(&seq.value);
            format!("Sequence({} instructions, value={value})", seq.instructions.len())
        }
        ReactiveValue::OptionalCall(opt) => {
            let value = write_reactive_value(&opt.value);
            format!("Optional(optional={}, {value})", opt.optional)
        }
    }
}

fn write_reactive_terminal(writer: &mut Writer, terminal: &ReactiveTerminal) {
    match terminal {
        ReactiveTerminal::Break(t) => {
            writer.write_line(&format!("[{}] Break bb{}", t.id.0, t.target.0));
        }
        ReactiveTerminal::Continue(t) => {
            writer.write_line(&format!("[{}] Continue bb{}", t.id.0, t.target.0));
        }
        ReactiveTerminal::Return(t) => {
            writer.write_line(&format!("[{}] Return {}", t.id.0, print_place(&t.value)));
        }
        ReactiveTerminal::Throw(t) => {
            writer.write_line(&format!("[{}] Throw {}", t.id.0, print_place(&t.value)));
        }
        ReactiveTerminal::If(t) => {
            writer.write_line(&format!("[{}] If ({}) {{", t.id.0, print_place(&t.test)));
            write_reactive_block(writer, &t.consequent);
            if let Some(alt) = &t.alternate {
                writer.write_line("} else {");
                write_reactive_block(writer, alt);
            }
            writer.write_line("}");
        }
        ReactiveTerminal::Switch(t) => {
            writer.write_line(&format!("[{}] Switch ({}) {{", t.id.0, print_place(&t.test)));
            for case in &t.cases {
                if let Some(test) = &case.test {
                    writer.write_line(&format!("  Case {}:", print_place(test)));
                } else {
                    writer.write_line("  Default:");
                }
                if let Some(block) = &case.block {
                    write_reactive_block(writer, block);
                }
            }
            writer.write_line("}");
        }
        ReactiveTerminal::While(t) => {
            writer.write_line(&format!("[{}] While {{", t.id.0));
            write_reactive_block(writer, &t.r#loop);
            writer.write_line("}");
        }
        ReactiveTerminal::DoWhile(t) => {
            writer.write_line(&format!("[{}] DoWhile {{", t.id.0));
            write_reactive_block(writer, &t.r#loop);
            writer.write_line("}");
        }
        ReactiveTerminal::For(t) => {
            writer.write_line(&format!("[{}] For {{", t.id.0));
            write_reactive_block(writer, &t.r#loop);
            writer.write_line("}");
        }
        ReactiveTerminal::ForOf(t) => {
            writer.write_line(&format!("[{}] ForOf {{", t.id.0));
            write_reactive_block(writer, &t.r#loop);
            writer.write_line("}");
        }
        ReactiveTerminal::ForIn(t) => {
            writer.write_line(&format!("[{}] ForIn {{", t.id.0));
            write_reactive_block(writer, &t.r#loop);
            writer.write_line("}");
        }
        ReactiveTerminal::Label(t) => {
            writer.write_line(&format!("[{}] Label {{", t.id.0));
            write_reactive_block(writer, &t.block);
            writer.write_line("}");
        }
        ReactiveTerminal::Try(t) => {
            writer.write_line(&format!("[{}] Try {{", t.id.0));
            write_reactive_block(writer, &t.block);
            writer.write_line("} catch {");
            write_reactive_block(writer, &t.handler);
            writer.write_line("}");
        }
    }
}
