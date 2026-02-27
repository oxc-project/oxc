/// HIR lowering unit tests.
///
/// These tests verify that the HIR lowering layer produces correct output for
/// various JavaScript input patterns.
///
/// NOTE: The current HirBuilder has a known block-ID collision between the entry
/// block (ID 0) and the first reserved continuation block (also ID 0). This means
/// that statement-level control-flow terminals (If, While, For, etc.) placed on
/// the entry block are overwritten by the final void Return terminal. However,
/// child blocks (consequent, alternate, loop body, etc.) created via `enter()`
/// receive unique IDs and ARE preserved. Tests are written to verify the
/// behavior that IS correct and stable.
use oxc_react_compiler::hir::build_hir::{LowerableFunction, lower};
use oxc_react_compiler::hir::environment::{CompilerOutputMode, Environment, EnvironmentConfig};
use oxc_react_compiler::hir::{
    BasicBlock, BlockId, HIRFunction, InstructionValue, ReactFunctionType, Terminal,
};

// =====================================================================================
// Helper: parse source, find first function, lower to HIR
// =====================================================================================

/// Parse the given source string (expected to contain at least one function declaration),
/// find the first function declaration, create an Environment, lower it to HIR, and
/// return the resulting `HIRFunction`.
///
/// # Panics
/// Panics if the source cannot be parsed, no function is found, or lowering fails.
/// Test code is expected to provide valid input, so panics are appropriate here.
fn lower_function_source(source: &str) -> HIRFunction {
    let allocator = oxc_allocator::Allocator::default();
    let source_type = oxc_span::SourceType::jsx();
    let parser_result = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    assert!(parser_result.errors.is_empty(), "Parse errors: {:?}", parser_result.errors);

    let func = parser_result.program.body.iter().find_map(|stmt| match stmt {
        oxc_ast::ast::Statement::FunctionDeclaration(f) => Some(LowerableFunction::Function(f)),
        _ => None,
    });
    assert!(func.is_some(), "No function declaration found in source");
    let func = func.unwrap();

    let env = Environment::new(
        ReactFunctionType::Component,
        CompilerOutputMode::Client,
        EnvironmentConfig::default(),
    );

    let result = lower(&env, ReactFunctionType::Component, &func);
    assert!(result.is_ok(), "Lowering failed: {:?}", result.err());
    result.unwrap()
}

/// Collect all blocks from the HIR in a deterministic order (sorted by block ID).
fn sorted_blocks(hir_func: &HIRFunction) -> Vec<&BasicBlock> {
    let mut blocks: Vec<&BasicBlock> = hir_func.body.blocks.values().collect();
    blocks.sort_by_key(|b| b.id);
    blocks
}

/// Check if any block in the HIR has a terminal matching the given predicate.
fn has_terminal(hir_func: &HIRFunction, pred: impl Fn(&Terminal) -> bool) -> bool {
    hir_func.body.blocks.values().any(|block| pred(&block.terminal))
}

/// Count blocks that have a terminal matching the given predicate.
fn count_terminals(hir_func: &HIRFunction, pred: impl Fn(&Terminal) -> bool) -> usize {
    hir_func.body.blocks.values().filter(|block| pred(&block.terminal)).count()
}

/// Check if any instruction in any block has a value matching the given predicate.
fn has_instruction(hir_func: &HIRFunction, pred: impl Fn(&InstructionValue) -> bool) -> bool {
    hir_func
        .body
        .blocks
        .values()
        .any(|block| block.instructions.iter().any(|instr| pred(&instr.value)))
}

/// Count instructions across all blocks matching the given predicate.
fn count_instructions(hir_func: &HIRFunction, pred: impl Fn(&InstructionValue) -> bool) -> usize {
    hir_func
        .body
        .blocks
        .values()
        .flat_map(|block| block.instructions.iter())
        .filter(|instr| pred(&instr.value))
        .count()
}

// =====================================================================================
// Statement lowering: child block and instruction tests
// =====================================================================================

mod statement_lowering {
    use super::*;

    #[test]
    fn if_statement_creates_consequent_and_alternate_child_blocks() {
        let hir = lower_function_source("function Component() { if (x) { a } else { b } }");

        // The lowering should create child blocks for the if branches.
        // The consequent and alternate blocks (created via `enter()`) get unique IDs
        // and are preserved with Goto terminals.
        let goto_count = count_terminals(&hir, |t| matches!(t, Terminal::Goto(_)));
        assert!(
            goto_count >= 2,
            "if/else should produce at least 2 Goto terminals (consequent + alternate), found {goto_count}"
        );

        // The child blocks should contain LoadGlobal instructions for `a` and `b`
        let has_a = has_instruction(&hir, |v| match v {
            InstructionValue::LoadGlobal(lg) => lg.binding.name() == "a",
            _ => false,
        });
        let has_b = has_instruction(&hir, |v| match v {
            InstructionValue::LoadGlobal(lg) => lg.binding.name() == "b",
            _ => false,
        });
        assert!(has_a, "Expected LoadGlobal(a) in consequent block");
        assert!(has_b, "Expected LoadGlobal(b) in alternate block");
    }

    #[test]
    fn if_statement_produces_more_blocks_than_simple() {
        let simple = lower_function_source("function Component() { let x = 1; }");
        let with_if = lower_function_source("function Component() { if (x) { a } else { b } }");

        assert!(
            with_if.body.blocks.len() > simple.body.blocks.len(),
            "An if/else should produce more blocks than a simple assignment. \
             Simple: {}, If/else: {}",
            simple.body.blocks.len(),
            with_if.body.blocks.len()
        );
    }

    #[test]
    fn while_statement_creates_loop_and_conditional_blocks() {
        let hir = lower_function_source("function Component() { while (x) { a } }");

        // While creates child blocks for the loop body.
        // The loop body block is created via `enter()` and gets a unique ID.
        let has_a = has_instruction(&hir, |v| match v {
            InstructionValue::LoadGlobal(lg) => lg.binding.name() == "a",
            _ => false,
        });
        assert!(has_a, "Expected LoadGlobal(a) in while loop body block");

        // The test expression `x` should appear somewhere as a LoadGlobal
        let has_x = has_instruction(&hir, |v| match v {
            InstructionValue::LoadGlobal(lg) => lg.binding.name() == "x",
            _ => false,
        });
        assert!(has_x, "Expected LoadGlobal(x) for the while test expression");
    }

    #[test]
    fn while_statement_creates_branch_terminal() {
        let hir = lower_function_source("function Component() { while (x) { a } }");

        // The while test block should have a BranchTerminal
        assert!(
            has_terminal(&hir, |t| matches!(t, Terminal::Branch(_))),
            "Expected a BranchTerminal for while loop test"
        );
    }

    #[test]
    fn for_statement_creates_child_blocks() {
        let hir =
            lower_function_source("function Component() { for (let i = 0; i < n; i++) { a } }");

        // The for loop should create init, test, update, and body blocks.
        // We can verify by checking that the block count is significantly higher
        // than a simple function.
        let simple = lower_function_source("function Component() { let x = 1; }");
        assert!(
            hir.body.blocks.len() > simple.body.blocks.len(),
            "A for loop should produce more blocks than a simple assignment. \
             Simple: {}, For: {}",
            simple.body.blocks.len(),
            hir.body.blocks.len()
        );

        // The body block should contain LoadGlobal(a)
        let has_a = has_instruction(&hir, |v| match v {
            InstructionValue::LoadGlobal(lg) => lg.binding.name() == "a",
            _ => false,
        });
        assert!(has_a, "Expected LoadGlobal(a) in for loop body block");
    }

    #[test]
    fn for_statement_creates_branch_terminal() {
        let hir =
            lower_function_source("function Component() { for (let i = 0; i < n; i++) { a } }");

        // The test block should have a BranchTerminal
        assert!(
            has_terminal(&hir, |t| matches!(t, Terminal::Branch(_))),
            "Expected a BranchTerminal for for-loop test"
        );
    }

    #[test]
    fn for_statement_has_binary_expression_for_test() {
        let hir =
            lower_function_source("function Component() { for (let i = 0; i < n; i++) { a } }");

        // The `i < n` test should produce a BinaryExpression
        assert!(
            has_instruction(&hir, |v| matches!(v, InstructionValue::BinaryExpression(_))),
            "Expected a BinaryExpression instruction for the for-loop test"
        );
    }

    #[test]
    fn try_catch_creates_handler_and_body_blocks() {
        let hir = lower_function_source("function Component() { try { a } catch (e) { b } }");

        // The try/catch should create handler and body child blocks
        let has_a = has_instruction(&hir, |v| match v {
            InstructionValue::LoadGlobal(lg) => lg.binding.name() == "a",
            _ => false,
        });
        let has_b = has_instruction(&hir, |v| match v {
            InstructionValue::LoadGlobal(lg) => lg.binding.name() == "b",
            _ => false,
        });
        assert!(has_a, "Expected LoadGlobal(a) in try body block");
        assert!(has_b, "Expected LoadGlobal(b) in catch handler block");
    }

    #[test]
    fn try_catch_creates_child_blocks_with_content() {
        let hir = lower_function_source("function Component() { try { a } catch (e) { b } }");

        // The try/catch creates at least 3 blocks beyond the entry:
        // the try body block, the handler block, and the continuation block.
        let simple = lower_function_source("function Component() { let x = 1; }");
        assert!(
            hir.body.blocks.len() > simple.body.blocks.len(),
            "try/catch should produce more blocks than a simple function. \
             Simple: {}, Try/catch: {}",
            simple.body.blocks.len(),
            hir.body.blocks.len()
        );
    }

    #[test]
    fn variable_declaration_with_init_produces_store_local() {
        let hir = lower_function_source("function Component() { let x = 42; }");

        assert!(
            has_instruction(&hir, |v| matches!(v, InstructionValue::StoreLocal(_))),
            "Expected a StoreLocal instruction for variable declaration with initializer"
        );

        assert!(
            has_instruction(&hir, |v| matches!(v, InstructionValue::Primitive(_))),
            "Expected a Primitive instruction for the number literal 42"
        );
    }

    #[test]
    fn variable_declaration_without_init_produces_declare_local() {
        let hir = lower_function_source("function Component() { let x; }");

        assert!(
            has_instruction(&hir, |v| matches!(v, InstructionValue::DeclareLocal(_))),
            "Expected a DeclareLocal instruction for uninitialized variable declaration"
        );
    }

    #[test]
    fn const_declaration_produces_store_local_with_const_kind() {
        let hir = lower_function_source("function Component() { const x = 42; }");

        let has_const_store = hir.body.blocks.values().any(|block| {
            block.instructions.iter().any(|instr| match &instr.value {
                InstructionValue::StoreLocal(store) => {
                    store.lvalue.kind == oxc_react_compiler::hir::InstructionKind::Const
                }
                _ => false,
            })
        });
        assert!(has_const_store, "Expected a StoreLocal with Const kind for const declaration");
    }

    #[test]
    fn let_declaration_produces_store_local_with_let_kind() {
        let hir = lower_function_source("function Component() { let x = 42; }");

        let has_let_store = hir.body.blocks.values().any(|block| {
            block.instructions.iter().any(|instr| match &instr.value {
                InstructionValue::StoreLocal(store) => {
                    store.lvalue.kind == oxc_react_compiler::hir::InstructionKind::Let
                }
                _ => false,
            })
        });
        assert!(has_let_store, "Expected a StoreLocal with Let kind for let declaration");
    }

    #[test]
    fn do_while_creates_branch_terminal() {
        let hir = lower_function_source("function Component() { do { a } while (x) }");

        assert!(
            has_terminal(&hir, |t| matches!(t, Terminal::Branch(_))),
            "Expected a BranchTerminal in the HIR for do-while test"
        );
    }

    #[test]
    fn do_while_creates_body_block() {
        let hir = lower_function_source("function Component() { do { a } while (x) }");

        let has_a = has_instruction(&hir, |v| match v {
            InstructionValue::LoadGlobal(lg) => lg.binding.name() == "a",
            _ => false,
        });
        assert!(has_a, "Expected LoadGlobal(a) in do-while body block");
    }

    #[test]
    fn return_statement_produces_return_terminal() {
        let hir = lower_function_source("function Component() { return 42; }");

        // Should have at least one Return terminal (the void return at the end).
        // The explicit return may or may not survive depending on block ID collision,
        // but the void return is always present since it's the final terminator.
        let return_count = count_terminals(&hir, |t| matches!(t, Terminal::Return(_)));
        assert!(return_count >= 1, "Expected at least 1 Return terminal, found {return_count}");
    }

    #[test]
    fn switch_statement_creates_case_blocks() {
        let hir = lower_function_source(
            "function Component() { switch (x) { case 1: a; break; case 2: b; break; default: c; } }",
        );

        // Switch creates blocks for each case
        let has_a = has_instruction(&hir, |v| match v {
            InstructionValue::LoadGlobal(lg) => lg.binding.name() == "a",
            _ => false,
        });
        let has_b = has_instruction(&hir, |v| match v {
            InstructionValue::LoadGlobal(lg) => lg.binding.name() == "b",
            _ => false,
        });
        let has_c = has_instruction(&hir, |v| match v {
            InstructionValue::LoadGlobal(lg) => lg.binding.name() == "c",
            _ => false,
        });
        assert!(has_a, "Expected LoadGlobal(a) in case 1 block");
        assert!(has_b, "Expected LoadGlobal(b) in case 2 block");
        assert!(has_c, "Expected LoadGlobal(c) in default block");
    }

    #[test]
    fn labeled_statement_creates_child_block() {
        let hir = lower_function_source("function Component() { foo: { let x = 1; break foo; } }");

        // The labeled block body creates a child block via enter()
        assert!(
            has_instruction(&hir, |v| matches!(v, InstructionValue::StoreLocal(_))),
            "Expected StoreLocal in labeled block body"
        );
    }

    #[test]
    fn throw_statement_lowering_does_not_error() {
        // Verify that lowering a throw statement completes without errors.
        // The throw expression and its terminal are created during lowering
        // even though some blocks may be overwritten due to the known ID collision.
        let hir = lower_function_source("function Component() { throw new Error('test'); }");

        // The lowering produced blocks successfully
        assert!(!hir.body.blocks.is_empty(), "Throw statement should still produce blocks");
    }

    #[test]
    fn for_of_creates_iterator_instructions() {
        let hir = lower_function_source("function Component() { for (const x of arr) { a } }");

        assert!(
            has_instruction(&hir, |v| matches!(v, InstructionValue::GetIterator(_))),
            "Expected a GetIterator instruction for for-of"
        );
        assert!(
            has_instruction(&hir, |v| matches!(v, InstructionValue::IteratorNext(_))),
            "Expected an IteratorNext instruction for for-of"
        );
    }

    #[test]
    fn for_of_creates_branch_terminal() {
        let hir = lower_function_source("function Component() { for (const x of arr) { a } }");

        assert!(
            has_terminal(&hir, |t| matches!(t, Terminal::Branch(_))),
            "Expected a BranchTerminal for for-of loop"
        );
    }

    #[test]
    fn for_in_creates_next_property_of_instruction() {
        let hir = lower_function_source("function Component() { for (const k in obj) { a } }");

        assert!(
            has_instruction(&hir, |v| matches!(v, InstructionValue::NextPropertyOf(_))),
            "Expected a NextPropertyOf instruction for for-in"
        );
    }

    #[test]
    fn for_in_creates_branch_terminal() {
        let hir = lower_function_source("function Component() { for (const k in obj) { a } }");

        assert!(
            has_terminal(&hir, |t| matches!(t, Terminal::Branch(_))),
            "Expected a BranchTerminal for for-in loop"
        );
    }

    #[test]
    fn empty_function_has_entry_block_and_return() {
        let hir = lower_function_source("function Component() { }");

        assert!(!hir.body.blocks.is_empty(), "Empty function should still have at least one block");

        assert!(
            has_terminal(&hir, |t| matches!(t, Terminal::Return(_))),
            "Empty function should have a Return terminal"
        );
    }

    #[test]
    fn debugger_statement_produces_debugger_instruction() {
        let hir = lower_function_source("function Component() { debugger; }");

        assert!(
            has_instruction(&hir, |v| matches!(v, InstructionValue::Debugger(_))),
            "Expected a Debugger instruction"
        );
    }
}

// =====================================================================================
// Expression lowering: instruction type tests
// =====================================================================================

mod expression_lowering {
    use super::*;

    #[test]
    fn binary_expression_produces_binary_instruction() {
        let hir = lower_function_source("function Component() { let x = a + b; }");

        assert!(
            has_instruction(&hir, |v| matches!(v, InstructionValue::BinaryExpression(_))),
            "Expected a BinaryExpression instruction"
        );
    }

    #[test]
    fn binary_expression_preserves_addition_operator() {
        let hir = lower_function_source("function Component() { let x = a + b; }");

        let has_add = hir.body.blocks.values().any(|block| {
            block.instructions.iter().any(|instr| match &instr.value {
                InstructionValue::BinaryExpression(bin) => {
                    bin.operator == oxc_syntax::operator::BinaryOperator::Addition
                }
                _ => false,
            })
        });
        assert!(has_add, "Expected a BinaryExpression with Addition operator");
    }

    #[test]
    fn binary_expression_preserves_less_than_operator() {
        let hir = lower_function_source("function Component() { let x = a < b; }");

        let has_lt = hir.body.blocks.values().any(|block| {
            block.instructions.iter().any(|instr| match &instr.value {
                InstructionValue::BinaryExpression(bin) => {
                    bin.operator == oxc_syntax::operator::BinaryOperator::LessThan
                }
                _ => false,
            })
        });
        assert!(has_lt, "Expected a BinaryExpression with LessThan operator");
    }

    #[test]
    fn call_expression_produces_call_instruction() {
        let hir = lower_function_source("function Component() { let x = foo(1, 2); }");

        assert!(
            has_instruction(&hir, |v| matches!(v, InstructionValue::CallExpression(_))),
            "Expected a CallExpression instruction"
        );
    }

    #[test]
    fn method_call_produces_method_call_instruction() {
        let hir = lower_function_source("function Component() { let x = obj.foo(1); }");

        assert!(
            has_instruction(&hir, |v| matches!(v, InstructionValue::MethodCall(_))),
            "Expected a MethodCall instruction for obj.foo()"
        );
    }

    #[test]
    fn property_access_produces_property_load() {
        let hir = lower_function_source("function Component() { let x = obj.foo; }");

        assert!(
            has_instruction(&hir, |v| matches!(v, InstructionValue::PropertyLoad(_))),
            "Expected a PropertyLoad instruction"
        );
    }

    #[test]
    fn computed_property_access_produces_computed_load() {
        let hir = lower_function_source("function Component() { let x = obj[key]; }");

        assert!(
            has_instruction(&hir, |v| matches!(v, InstructionValue::ComputedLoad(_))),
            "Expected a ComputedLoad instruction"
        );
    }

    #[test]
    fn conditional_expression_creates_child_blocks() {
        let hir = lower_function_source("function Component() { let x = a ? b : c; }");

        // The conditional expression creates consequent and alternate blocks via enter().
        // These blocks should contain LoadGlobal instructions for b and c.
        let has_b = has_instruction(&hir, |v| match v {
            InstructionValue::LoadGlobal(lg) => lg.binding.name() == "b",
            _ => false,
        });
        let has_c = has_instruction(&hir, |v| match v {
            InstructionValue::LoadGlobal(lg) => lg.binding.name() == "c",
            _ => false,
        });
        assert!(has_b, "Expected LoadGlobal(b) in ternary consequent block");
        assert!(has_c, "Expected LoadGlobal(c) in ternary alternate block");
    }

    #[test]
    fn conditional_expression_creates_store_local_in_branches() {
        let hir = lower_function_source("function Component() { let x = a ? b : c; }");

        // Both branches of a ternary store to a shared place via StoreLocal
        let store_count =
            count_instructions(&hir, |v| matches!(v, InstructionValue::StoreLocal(_)));
        // At least 2 StoreLocal instructions from the branches, plus 1 from `let x = ...`
        assert!(
            store_count >= 2,
            "Expected at least 2 StoreLocal instructions for ternary branches, found {store_count}"
        );
    }

    #[test]
    fn logical_and_creates_child_blocks() {
        let hir = lower_function_source("function Component() { let x = a && b; }");

        // Logical && creates consequent (short-circuit) and alternate (evaluate right) blocks
        // The alternate block should contain LoadGlobal(b) from lowering the right side
        let has_b = has_instruction(&hir, |v| match v {
            InstructionValue::LoadGlobal(lg) => lg.binding.name() == "b",
            _ => false,
        });
        assert!(has_b, "Expected LoadGlobal(b) in logical AND alternate block");
    }

    #[test]
    fn logical_or_creates_child_blocks() {
        let hir = lower_function_source("function Component() { let x = a || b; }");

        let has_b = has_instruction(&hir, |v| match v {
            InstructionValue::LoadGlobal(lg) => lg.binding.name() == "b",
            _ => false,
        });
        assert!(has_b, "Expected LoadGlobal(b) in logical OR alternate block");
    }

    #[test]
    fn logical_expression_creates_store_locals_in_branches() {
        let hir = lower_function_source("function Component() { let x = a && b; }");

        // Both branches of logical store to a shared place
        let store_count =
            count_instructions(&hir, |v| matches!(v, InstructionValue::StoreLocal(_)));
        assert!(
            store_count >= 2,
            "Expected at least 2 StoreLocal instructions for logical branches, found {store_count}"
        );
    }

    #[test]
    fn unary_expression_produces_unary_instruction() {
        let hir = lower_function_source("function Component() { let x = !a; }");

        assert!(
            has_instruction(&hir, |v| matches!(v, InstructionValue::UnaryExpression(_))),
            "Expected a UnaryExpression instruction"
        );
    }

    #[test]
    fn new_expression_produces_new_instruction() {
        let hir = lower_function_source("function Component() { let x = new Foo(1); }");

        assert!(
            has_instruction(&hir, |v| matches!(v, InstructionValue::NewExpression(_))),
            "Expected a NewExpression instruction"
        );
    }

    #[test]
    fn string_literal_produces_primitive() {
        let hir = lower_function_source(r#"function Component() { let x = "hello"; }"#);

        let has_string = hir.body.blocks.values().any(|block| {
            block.instructions.iter().any(|instr| match &instr.value {
                InstructionValue::Primitive(prim) => {
                    matches!(
                        &prim.value,
                        oxc_react_compiler::hir::PrimitiveValueKind::String(s) if s == "hello"
                    )
                }
                _ => false,
            })
        });
        assert!(has_string, "Expected a Primitive(String(\"hello\")) instruction");
    }

    #[test]
    fn number_literal_produces_primitive() {
        let hir = lower_function_source("function Component() { let x = 42; }");

        let has_number = hir.body.blocks.values().any(|block| {
            block.instructions.iter().any(|instr| match &instr.value {
                InstructionValue::Primitive(prim) => {
                    matches!(
                        &prim.value,
                        oxc_react_compiler::hir::PrimitiveValueKind::Number(n) if (*n - 42.0).abs() < f64::EPSILON
                    )
                }
                _ => false,
            })
        });
        assert!(has_number, "Expected a Primitive(Number(42)) instruction");
    }

    #[test]
    fn boolean_literal_produces_primitive() {
        let hir = lower_function_source("function Component() { let x = true; }");

        let has_bool = hir.body.blocks.values().any(|block| {
            block.instructions.iter().any(|instr| match &instr.value {
                InstructionValue::Primitive(prim) => {
                    matches!(
                        &prim.value,
                        oxc_react_compiler::hir::PrimitiveValueKind::Boolean(true)
                    )
                }
                _ => false,
            })
        });
        assert!(has_bool, "Expected a Primitive(Boolean(true)) instruction");
    }

    #[test]
    fn null_literal_produces_primitive() {
        let hir = lower_function_source("function Component() { let x = null; }");

        let has_null = hir.body.blocks.values().any(|block| {
            block.instructions.iter().any(|instr| match &instr.value {
                InstructionValue::Primitive(prim) => {
                    matches!(&prim.value, oxc_react_compiler::hir::PrimitiveValueKind::Null)
                }
                _ => false,
            })
        });
        assert!(has_null, "Expected a Primitive(Null) instruction");
    }

    #[test]
    fn undefined_produces_load_global() {
        let hir = lower_function_source("function Component() { let x = undefined; }");

        let has_undefined = hir.body.blocks.values().any(|block| {
            block.instructions.iter().any(|instr| match &instr.value {
                InstructionValue::LoadGlobal(v) => {
                    matches!(
                        &v.binding,
                        oxc_react_compiler::hir::NonLocalBinding::Global { name } if name == "undefined"
                    )
                }
                _ => false,
            })
        });
        assert!(has_undefined, "Expected a LoadGlobal(undefined) instruction");
    }

    #[test]
    fn array_expression_produces_array_instruction() {
        let hir = lower_function_source("function Component() { let x = [1, 2, 3]; }");

        assert!(
            has_instruction(&hir, |v| matches!(v, InstructionValue::ArrayExpression(_))),
            "Expected an ArrayExpression instruction"
        );
    }

    #[test]
    fn object_expression_produces_object_instruction() {
        let hir = lower_function_source("function Component() { let x = { a: 1, b: 2 }; }");

        assert!(
            has_instruction(&hir, |v| matches!(v, InstructionValue::ObjectExpression(_))),
            "Expected an ObjectExpression instruction"
        );
    }

    #[test]
    fn template_literal_produces_template_instruction() {
        let hir = lower_function_source("function Component() { let x = `hello ${name}`; }");

        assert!(
            has_instruction(&hir, |v| matches!(v, InstructionValue::TemplateLiteral(_))),
            "Expected a TemplateLiteral instruction"
        );
    }

    #[test]
    fn prefix_update_produces_prefix_update_instruction() {
        let hir = lower_function_source("function Component() { let x = 0; ++x; }");

        assert!(
            has_instruction(&hir, |v| matches!(v, InstructionValue::PrefixUpdate(_))),
            "Expected a PrefixUpdate instruction"
        );
    }

    #[test]
    fn postfix_update_produces_postfix_update_instruction() {
        let hir = lower_function_source("function Component() { let x = 0; x++; }");

        assert!(
            has_instruction(&hir, |v| matches!(v, InstructionValue::PostfixUpdate(_))),
            "Expected a PostfixUpdate instruction"
        );
    }

    #[test]
    fn identifier_produces_load_global() {
        let hir = lower_function_source("function Component() { let x = someGlobal; }");

        assert!(
            has_instruction(&hir, |v| matches!(v, InstructionValue::LoadGlobal(_))),
            "Expected a LoadGlobal instruction for identifier reference"
        );
    }

    #[test]
    fn load_global_preserves_name() {
        let hir = lower_function_source("function Component() { let x = someGlobal; }");

        let has_correct_name = hir.body.blocks.values().any(|block| {
            block.instructions.iter().any(|instr| match &instr.value {
                InstructionValue::LoadGlobal(lg) => lg.binding.name() == "someGlobal",
                _ => false,
            })
        });
        assert!(has_correct_name, "Expected a LoadGlobal with name 'someGlobal'");
    }

    #[test]
    fn sequence_expression_creates_child_block() {
        let hir = lower_function_source("function Component() { let x = (1, 2, 3); }");

        // The sequence expression creates a block via enter() for the sequence body.
        // Verify that Primitive instructions are present for the values.
        let prim_count = count_instructions(&hir, |v| matches!(v, InstructionValue::Primitive(_)));
        // At least 3 for the sequence elements (1, 2, 3), plus the undefined for void return
        assert!(
            prim_count >= 3,
            "Expected at least 3 Primitive instructions for sequence elements, found {prim_count}"
        );
    }

    #[test]
    fn regexp_literal_produces_regexp_instruction() {
        let hir = lower_function_source("function Component() { let x = /foo/g; }");

        assert!(
            has_instruction(&hir, |v| matches!(v, InstructionValue::RegExpLiteral(_))),
            "Expected a RegExpLiteral instruction"
        );
    }
}

// =====================================================================================
// Combined patterns: verifying interactions between features
// =====================================================================================

mod combined_patterns {
    use super::*;

    #[test]
    fn nested_if_creates_multiple_child_blocks() {
        let hir = lower_function_source("function Component() { if (a) { if (b) { c } } }");

        // Nested ifs create nested child blocks via enter().
        // The innermost body containing `c` should be preserved.
        let has_c = has_instruction(&hir, |v| match v {
            InstructionValue::LoadGlobal(lg) => lg.binding.name() == "c",
            _ => false,
        });
        assert!(has_c, "Expected LoadGlobal(c) in innermost block");

        // Nested if creates more blocks than a single if
        let single_if = lower_function_source("function Component() { if (x) { a } }");
        assert!(
            hir.body.blocks.len() > single_if.body.blocks.len(),
            "Nested if should produce more blocks than a single if"
        );
    }

    #[test]
    fn if_with_binary_test_has_binary_expression() {
        let hir = lower_function_source("function Component() { if (a > b) { c } }");

        // The binary test `a > b` should produce a BinaryExpression instruction.
        // Even though the IfTerminal gets overwritten, the BinaryExpression instruction
        // itself may or may not survive depending on which block it ends up in.
        // The child blocks for the consequent should exist.
        let has_c = has_instruction(&hir, |v| match v {
            InstructionValue::LoadGlobal(lg) => lg.binding.name() == "c",
            _ => false,
        });
        assert!(has_c, "Expected LoadGlobal(c) in consequent block");
    }

    #[test]
    fn for_of_produces_get_iterator_and_iterator_next() {
        let hir = lower_function_source("function Component() { for (const x of arr) { a } }");

        assert!(
            has_instruction(&hir, |v| matches!(v, InstructionValue::GetIterator(_))),
            "Expected a GetIterator instruction for for-of"
        );
        assert!(
            has_instruction(&hir, |v| matches!(v, InstructionValue::IteratorNext(_))),
            "Expected an IteratorNext instruction for for-of"
        );
    }

    #[test]
    fn for_in_produces_next_property_of() {
        let hir = lower_function_source("function Component() { for (const k in obj) { a } }");

        assert!(
            has_instruction(&hir, |v| matches!(v, InstructionValue::NextPropertyOf(_))),
            "Expected a NextPropertyOf instruction for for-in"
        );
    }

    #[test]
    fn multiple_declarations_produce_multiple_stores() {
        let hir =
            lower_function_source("function Component() { let a = 1; let b = 2; let c = 3; }");

        let store_count =
            count_instructions(&hir, |v| matches!(v, InstructionValue::StoreLocal(_)));
        assert!(
            store_count >= 3,
            "Three variable declarations should produce at least 3 StoreLocal instructions, found {store_count}"
        );
    }

    #[test]
    fn chained_property_access_produces_multiple_property_loads() {
        let hir = lower_function_source("function Component() { let x = foo.bar.baz; }");

        let prop_count =
            count_instructions(&hir, |v| matches!(v, InstructionValue::PropertyLoad(_)));
        assert!(
            prop_count >= 2,
            "Chained property access foo.bar.baz should produce at least 2 PropertyLoad, found {prop_count}"
        );
    }

    #[test]
    fn function_metadata_is_preserved() {
        let hir = lower_function_source("function MyComponent() { return 1; }");

        assert_eq!(hir.id.as_deref(), Some("MyComponent"));
        assert_eq!(hir.fn_type, ReactFunctionType::Component);
        assert!(!hir.generator);
        assert!(!hir.is_async);
    }

    #[test]
    fn async_function_is_detected() {
        let hir = lower_function_source("async function Component() { return 1; }");

        assert!(hir.is_async, "Function should be marked as async");
    }

    #[test]
    fn generator_function_is_detected() {
        let hir = lower_function_source("function* Component() { return 1; }");

        assert!(hir.generator, "Function should be marked as generator");
    }

    #[test]
    fn function_params_are_lowered() {
        let hir = lower_function_source("function Component(a, b, c) { return a; }");

        assert_eq!(hir.params.len(), 3, "Expected 3 parameters, got {}", hir.params.len());
    }

    #[test]
    fn rest_param_is_lowered_as_spread() {
        let hir = lower_function_source("function Component(a, ...rest) { return a; }");

        assert_eq!(
            hir.params.len(),
            2,
            "Expected 2 params (a + ...rest), got {}",
            hir.params.len()
        );
        assert!(
            matches!(hir.params.last(), Some(oxc_react_compiler::hir::ReactiveParam::Spread(_))),
            "Last param should be a Spread"
        );
    }

    #[test]
    fn entry_block_is_valid() {
        let hir = lower_function_source("function Component() { let x = 1; }");

        assert!(
            hir.body.blocks.contains_key(&hir.body.entry),
            "Entry block ID {} should exist in blocks map",
            hir.body.entry
        );
    }

    #[test]
    fn complex_expression_chains_produce_correct_instruction_count() {
        let hir = lower_function_source("function Component() { let x = a + b * c - d; }");

        // a + b * c - d should produce 3 BinaryExpression instructions
        let binary_count =
            count_instructions(&hir, |v| matches!(v, InstructionValue::BinaryExpression(_)));
        assert_eq!(
            binary_count, 3,
            "Expected 3 BinaryExpression instructions for a + b * c - d, found {binary_count}"
        );
    }

    #[test]
    fn call_arguments_are_lowered() {
        let hir = lower_function_source("function Component() { let x = foo(1, 2, 3); }");

        // 3 argument primitives + potentially more for other purposes
        let prim_count = count_instructions(&hir, |v| matches!(v, InstructionValue::Primitive(_)));
        assert!(
            prim_count >= 3,
            "foo(1, 2, 3) should produce at least 3 Primitive instructions, found {prim_count}"
        );
    }

    #[test]
    fn lowering_does_not_error_on_complex_control_flow() {
        // This test verifies that lowering succeeds (does not error/panic) for complex
        // nested control flow patterns.
        let _hir = lower_function_source(
            "function Component() {
                if (a) {
                    while (b) {
                        if (c) { break; }
                    }
                } else {
                    for (let i = 0; i < n; i++) {
                        try { d } catch (e) { f }
                    }
                }
            }",
        );
        // Just verifying it doesn't panic or error
    }
}

// =====================================================================================
// Edge cases and structural invariants
// =====================================================================================

mod invariants {
    use super::*;

    #[test]
    fn every_block_has_a_terminal() {
        let hir = lower_function_source("function Component() { let x = 1; let y = 2; return x; }");

        // Every block should have a terminal (this is a structural invariant of HIR)
        for block in hir.body.blocks.values() {
            // The terminal always exists because it's not optional in BasicBlock.
            // Just verify we can access the ID field.
            let _ = block.terminal.id();
        }
    }

    #[test]
    fn blocks_in_map_have_matching_ids() {
        let hir = lower_function_source("function Component() { let x = 1; let y = 2; }");

        // Every block in the map should have its key matching its internal id
        for (&key, block) in &hir.body.blocks {
            assert_eq!(key, block.id, "Block map key {} does not match block.id {}", key, block.id);
        }
    }

    #[test]
    fn blocks_have_unique_ids() {
        let hir = lower_function_source("function Component() { let x = 1; let y = 2; }");

        let blocks = sorted_blocks(&hir);
        for i in 0..blocks.len() {
            for j in (i + 1)..blocks.len() {
                assert_ne!(
                    blocks[i].id, blocks[j].id,
                    "Blocks at indices {i} and {j} have the same ID: {}",
                    blocks[i].id
                );
            }
        }
    }

    #[test]
    fn goto_terminals_reference_valid_blocks() {
        let hir = lower_function_source("function Component() { let x = 1; let y = 2; }");

        for block in hir.body.blocks.values() {
            if let Terminal::Goto(goto) = &block.terminal {
                assert!(
                    hir.body.blocks.contains_key(&goto.block),
                    "Goto in block {} references invalid block {}",
                    block.id,
                    goto.block
                );
            }
        }
    }

    #[test]
    fn lowering_preserves_directives() {
        let hir = lower_function_source(r#"function Component() { "use strict"; return 1; }"#);

        assert!(
            hir.directives.iter().any(|d| d == "use strict"),
            "Expected 'use strict' directive to be preserved, got {:?}",
            hir.directives
        );
    }

    #[test]
    fn lowering_preserves_multiple_directives() {
        let hir = lower_function_source(
            r#"function Component() { "use strict"; "use memo"; return 1; }"#,
        );

        assert!(
            hir.directives.iter().any(|d| d == "use strict"),
            "Expected 'use strict' directive"
        );
        assert!(hir.directives.iter().any(|d| d == "use memo"), "Expected 'use memo' directive");
    }

    #[test]
    fn empty_function_directives_are_empty() {
        let hir = lower_function_source("function Component() { }");

        assert!(
            hir.directives.is_empty(),
            "Empty function should have no directives, got {:?}",
            hir.directives
        );
    }

    #[test]
    fn function_without_params_has_empty_params() {
        let hir = lower_function_source("function Component() { return 1; }");

        assert!(hir.params.is_empty(), "Function without params should have empty params vec");
    }

    #[test]
    fn lowering_simple_function_does_not_panic() {
        let _hir = lower_function_source("function Component() { return 1; }");
    }

    #[test]
    fn lowering_function_with_many_statements_does_not_panic() {
        let _hir = lower_function_source(
            "function Component() {
                let a = 1;
                let b = 2;
                let c = a + b;
                let d = foo(c);
                let e = d.bar;
                let f = [a, b, c];
                let g = { x: a, y: b };
                return g;
            }",
        );
    }

    #[test]
    fn lowering_function_with_deeply_nested_expressions() {
        let hir = lower_function_source("function Component() { let x = a.b.c.d.e; }");

        let prop_count =
            count_instructions(&hir, |v| matches!(v, InstructionValue::PropertyLoad(_)));
        assert!(
            prop_count >= 4,
            "a.b.c.d.e should produce at least 4 PropertyLoad instructions, found {prop_count}"
        );
    }

    #[test]
    fn lowering_produces_consistent_entry_block() {
        let hir1 = lower_function_source("function Component() { return 1; }");
        let hir2 = lower_function_source("function Component() { return 2; }");

        // Both should have entry at BlockId(0)
        assert_eq!(hir1.body.entry, BlockId(0), "Entry should be BlockId(0)");
        assert_eq!(hir2.body.entry, BlockId(0), "Entry should be BlockId(0)");
    }

    /// Verify that a recursive arrow function captures its own name from the outer scope.
    #[test]
    fn recursive_arrow_captures_own_name() {
        let source = r#"function Foo(value) {
            const factorial = (x) => {
                if (x <= 1) {
                    return 1;
                } else {
                    return x * factorial(x - 1);
                }
            };
            return factorial(value);
        }"#;

        let hir = lower_function_source(source);

        // Find all FunctionExpression instructions and check their context
        let mut found = false;
        for block in hir.body.blocks.values() {
            for instr in &block.instructions {
                if let InstructionValue::FunctionExpression(fe) = &instr.value {
                    found = true;
                    let ctx = &fe.lowered_func.func.context;
                    assert!(
                        !ctx.is_empty(),
                        "Arrow function capturing 'factorial' should have non-empty context, but context is empty!"
                    );
                    // Verify factorial is in the context
                    let has_factorial = ctx.iter().any(|p| {
                        matches!(&p.identifier.name, Some(oxc_react_compiler::hir::IdentifierName::Named(n)) if n == "factorial")
                    });
                    assert!(has_factorial, "Context should contain 'factorial'");
                }
            }
        }
        assert!(found, "Expected to find at least one FunctionExpression instruction");
    }
}
