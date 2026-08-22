use std::fmt::Write as _;

use rand::{RngExt, SeedableRng, rngs::StdRng};

const MAX_EXPRESSION_DEPTH: usize = 3;

/// Generate a deterministic, self-contained JavaScript program for a seed.
///
/// Like Terser's ufuzz generator, programs have bounded loops and a shared call
/// budget. They only observe behavior through the final `console.log` call.
///
/// <https://github.com/terser/terser/blob/v5.50.0/test/ufuzz.js>
pub fn generate(seed: u64) -> String {
    Generator::new(seed).program()
}

struct Generator {
    rng: StdRng,
    next_var: usize,
    next_brake: usize,
    next_effect: usize,
}

#[derive(Clone, Copy)]
struct StatementContext {
    in_function: bool,
    loop_depth: usize,
}

impl Generator {
    fn new(seed: u64) -> Self {
        Self { rng: StdRng::seed_from_u64(seed), next_var: 0, next_brake: 0, next_effect: 0 }
    }

    fn program(mut self) -> String {
        let mut source = String::from(
            "(function () {\n\
             \"use strict\";\n\
             var _calls_ = 10, a = 100, b = 10, c = 0, x = 1, y = 2, z = 3;\n\
             var obj = { p: 0, q: 1 }, arr = [0, 1, 2], trace = [];\n",
        );

        for function_index in 0..2 {
            let _ = writeln!(source, "function f{function_index}(p0, p1) {{");
            source.push_str("if (--_calls_ < 0) return 0;\n");
            let context = StatementContext { in_function: true, loop_depth: 0 };
            for _ in 0..self.range(1..=3) {
                source.push_str(&self.statement(3, context));
            }
            let return_expression = self.expression(MAX_EXPRESSION_DEPTH);
            let _ = writeln!(source, "return {return_expression};\n}}");
        }

        let context = StatementContext { in_function: false, loop_depth: 0 };
        for _ in 0..self.range(4..=7) {
            source.push_str(&self.statement(4, context));
        }
        source.push_str("console.log(null, a, b, c, x, y, z, _calls_, obj, arr, trace);\n");
        source.push_str("})();\n");
        source
    }

    fn statement(&mut self, depth: usize, context: StatementContext) -> String {
        if depth == 0 {
            return self.simple_statement();
        }

        match self.range(0..12) {
            0..=2 => self.simple_statement(),
            3 => {
                let condition = self.expression(2);
                let consequent = self.statement(depth - 1, context);
                let alternate = self.statement(depth - 1, context);
                format!("if ({condition}) {{\n{consequent}}} else {{\n{alternate}}}\n")
            }
            4 => self.block(depth - 1, context),
            5 => self.while_statement(depth - 1, context),
            6 => self.for_statement(depth - 1, context),
            7 => self.switch_statement(depth - 1, context),
            8 => self.try_statement(depth - 1, context),
            9 if context.in_function => {
                let expression = self.expression(2);
                format!("return {expression};\n")
            }
            10 if context.loop_depth > 0 => {
                if self.chance(0.5) {
                    "break;\n".into()
                } else {
                    "continue;\n".into()
                }
            }
            _ => self.do_while_statement(depth - 1, context),
        }
    }

    fn simple_statement(&mut self) -> String {
        if self.chance(0.25) {
            let name = self.fresh_var();
            let expression = self.expression(MAX_EXPRESSION_DEPTH);
            format!("var {name} = {expression};\n")
        } else {
            let expression = self.expression(MAX_EXPRESSION_DEPTH);
            format!("{expression};\n")
        }
    }

    fn block(&mut self, depth: usize, context: StatementContext) -> String {
        let mut block = String::from("{\n");
        for _ in 0..self.range(1..=3) {
            block.push_str(&self.statement(depth, context));
        }
        block.push_str("}\n");
        block
    }

    fn while_statement(&mut self, depth: usize, context: StatementContext) -> String {
        let brake = self.fresh_brake();
        let condition = self.expression(2);
        let body = self
            .statement(depth, StatementContext { loop_depth: context.loop_depth + 1, ..context });
        format!("{{ var {brake} = 4; while (--{brake} > 0 && ({condition})) {{\n{body}}} }}\n")
    }

    fn do_while_statement(&mut self, depth: usize, context: StatementContext) -> String {
        let brake = self.fresh_brake();
        let condition = self.expression(2);
        let body = self
            .statement(depth, StatementContext { loop_depth: context.loop_depth + 1, ..context });
        format!("{{ var {brake} = 4; do {{\n{body}}} while (--{brake} > 0 && ({condition})); }}\n")
    }

    fn for_statement(&mut self, depth: usize, context: StatementContext) -> String {
        let brake = self.fresh_brake();
        let condition = self.expression(2);
        let body = self
            .statement(depth, StatementContext { loop_depth: context.loop_depth + 1, ..context });
        format!("for (var {brake} = 4; {brake}-- > 0 && ({condition}); ) {{\n{body}}}\n")
    }

    fn switch_statement(&mut self, depth: usize, context: StatementContext) -> String {
        let discriminant = self.expression(2);
        let first = self.statement(depth, context);
        let second = self.statement(depth, context);
        let default = self.statement(depth, context);
        format!(
            "switch ({discriminant}) {{\ncase 0:\n{first}break;\ncase 1:\n{second}default:\n{default}}}\n"
        )
    }

    fn try_statement(&mut self, depth: usize, context: StatementContext) -> String {
        let condition = self.expression(1);
        let thrown = self.expression(1);
        let try_body = self.statement(depth, context);
        let catch_body = self.statement(depth, context);
        let effect = self.fresh_effect();
        format!(
            "try {{\nif ({condition}) throw {thrown};\n{try_body}}} catch (caught) {{\ntrace.push({effect});\n{catch_body}}}\n"
        )
    }

    fn expression(&mut self, depth: usize) -> String {
        if depth == 0 {
            return self.primitive();
        }

        match self.range(0..13) {
            0 | 1 => self.primitive(),
            2 => {
                let left = self.expression(depth - 1);
                let right = self.expression(depth - 1);
                let operator = self.pick(&[
                    "+", "-", "*", "/", "%", "&", "|", "^", "<<", ">>", ">>>", "<", "<=", ">",
                    ">=", "==", "===", "!=", "!==",
                ]);
                format!("({left} {operator} {right})")
            }
            3 => {
                let left = self.expression(depth - 1);
                let right = self.expression(depth - 1);
                let operator = self.pick(&["&&", "||", "??"]);
                format!("({left} {operator} {right})")
            }
            4 => {
                let condition = self.expression(depth - 1);
                let consequent = self.expression(depth - 1);
                let alternate = self.expression(depth - 1);
                format!("({condition} ? {consequent} : {alternate})")
            }
            5 => {
                let argument = self.expression(depth - 1);
                let operator = self.pick(&["+", "-", "!", "~", "void "]);
                format!("({operator} {argument})")
            }
            6 => {
                let effect = self.fresh_effect();
                let value = self.expression(depth - 1);
                format!("(trace.push({effect}), {value})")
            }
            7 => {
                let target = self.mutable_target();
                let value = self.expression(depth - 1);
                let operator = self.pick(&["=", "+=", "-=", "*=", "|=", "^="]);
                format!("({target} {operator} {value})")
            }
            8 => {
                let target = self.mutable_target();
                let operator = self.pick(&["++", "--"]);
                if self.chance(0.5) {
                    format!("({target}{operator})")
                } else {
                    format!("({operator}{target})")
                }
            }
            9 => {
                let function_index = self.range(0..2);
                let first = self.primitive();
                let second = self.primitive();
                format!("f{function_index}({first}, {second})")
            }
            10 => {
                let first = self.expression(depth - 1);
                let second = self.expression(depth - 1);
                format!("[{first}, {second}].length")
            }
            11 => {
                let value = self.expression(depth - 1);
                format!("({{ p: {value} }}).p")
            }
            _ => {
                let left = self.expression(depth - 1);
                let right = self.expression(depth - 1);
                format!("({left}, {right})")
            }
        }
    }

    fn primitive(&mut self) -> String {
        self.pick(&[
            "-3",
            "-1",
            "0",
            "1",
            "2",
            "3",
            "10",
            "42",
            "true",
            "false",
            "null",
            "undefined",
            "NaN",
            "Infinity",
            "''",
            "'x'",
            "a",
            "b",
            "c",
            "x",
            "y",
            "z",
            "obj.p",
            "obj.q",
            "arr[0]",
            "arr[1]",
        ])
        .into()
    }

    fn mutable_target(&mut self) -> &'static str {
        self.pick(&["a", "b", "c", "x", "y", "z", "obj.p", "obj.q", "arr[0]", "arr[1]"])
    }

    fn fresh_var(&mut self) -> String {
        let value = self.next_var;
        self.next_var += 1;
        format!("v{value}")
    }

    fn fresh_brake(&mut self) -> String {
        let value = self.next_brake;
        self.next_brake += 1;
        format!("brake{value}")
    }

    fn fresh_effect(&mut self) -> usize {
        let value = self.next_effect;
        self.next_effect += 1;
        value
    }

    fn chance(&mut self, probability: f64) -> bool {
        self.rng.random_bool(probability)
    }

    fn range<T, R>(&mut self, range: R) -> T
    where
        T: rand::distr::uniform::SampleUniform,
        R: rand::distr::uniform::SampleRange<T>,
    {
        self.rng.random_range(range)
    }

    fn pick<'a>(&mut self, choices: &'a [&'a str]) -> &'a str {
        choices[self.range(0..choices.len())]
    }
}
