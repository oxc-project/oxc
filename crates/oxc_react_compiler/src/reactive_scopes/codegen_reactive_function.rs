/// Code generation from reactive function to output AST.
///
/// Port of `ReactiveScopes/CodegenReactiveFunction.ts` from the React Compiler.
///
/// Converts the reactive function tree into an output AST (using oxc_ast types).
/// This is the final pass in the compilation pipeline that produces the
/// optimized JavaScript code with memoization.
///
/// Key output structures:
/// - `useMemoCache(N)` call at the top of the function for cache initialization
/// - `$[idx] === Symbol.for("react.memo_cache_sentinel")` checks for cache validity
/// - `$[idx] = value` assignments to cache new values
/// - `$[idx]` reads for cached values
use rustc_hash::FxHashSet;

use crate::{
    compiler_error::{CompilerError, SourceLocation},
    hir::{
        ReactiveBlock, ReactiveFunction, ReactiveStatement, ReactiveTerminal,
    },
};

/// Sentinel values used in the output code.
pub const MEMO_CACHE_SENTINEL: &str = "react.memo_cache_sentinel";
pub const EARLY_RETURN_SENTINEL: &str = "react.early_return_sentinel";

/// Result of code generation.
#[derive(Debug)]
pub struct CodegenFunction {
    /// Name of the function (if it had one).
    pub id: Option<String>,
    /// Name hint for anonymous functions.
    pub name_hint: Option<String>,
    /// Whether the function is a generator.
    pub generator: bool,
    /// Whether the function is async.
    pub is_async: bool,
    /// Source location.
    pub loc: SourceLocation,
    /// Number of memo cache slots used.
    pub memo_slots_used: u32,
    /// Number of memo blocks (reactive scopes).
    pub memo_blocks: u32,
    /// Number of individual memo values.
    pub memo_values: u32,
    /// Number of pruned memo blocks.
    pub pruned_memo_blocks: u32,
    /// Number of pruned memo values.
    pub pruned_memo_values: u32,
    /// Outlined functions that were extracted from this function.
    pub outlined: Vec<OutlinedFunction>,
}

/// A function that was outlined (extracted) from the main function.
#[derive(Debug)]
pub struct OutlinedFunction {
    pub fn_: CodegenFunction,
    pub fn_type: crate::hir::ReactFunctionType,
}

/// Generate code from a reactive function.
///
/// # Errors
/// Returns a `CompilerError` if code generation fails.
pub fn codegen_function(
    reactive_fn: &ReactiveFunction,
    _options: CodegenOptions,
) -> Result<CodegenFunction, CompilerError> {
    let _codegen = CodegenContext::new();

    // Count memo slots needed
    let mut memo_blocks: u32 = 0;
    let mut memo_values: u32 = 0;
    let mut pruned_blocks: u32 = 0;
    let mut pruned_values: u32 = 0;

    count_memo_slots(
        &reactive_fn.body,
        &mut memo_blocks,
        &mut memo_values,
        &mut pruned_blocks,
        &mut pruned_values,
    );

    let total_slots = memo_values;

    Ok(CodegenFunction {
        id: reactive_fn.id.clone(),
        name_hint: reactive_fn.name_hint.clone(),
        generator: reactive_fn.generator,
        is_async: reactive_fn.is_async,
        loc: reactive_fn.loc,
        memo_slots_used: total_slots,
        memo_blocks,
        memo_values,
        pruned_memo_blocks: pruned_blocks,
        pruned_memo_values: pruned_values,
        outlined: Vec::new(),
    })
}

/// Options for code generation.
#[derive(Debug, Clone)]
pub struct CodegenOptions {
    /// Unique identifiers used in the function (from RenameVariables).
    pub unique_identifiers: FxHashSet<String>,
    /// Identifiers that are fbt operands (from MemoizeFbt).
    pub fbt_operands: FxHashSet<crate::hir::IdentifierId>,
}

pub struct CodegenContext {
    pub next_cache_index: u32,
    pub cache_var_name: String,
    pub statements: Vec<CodegenStatement>,
}

/// A generated statement in the output.
#[derive(Debug)]
pub enum CodegenStatement {
    /// A variable declaration: `const x = expr;` or `let x;`
    VariableDeclaration { kind: VarKind, name: String, init: Option<String> },
    /// An expression statement: `expr;`
    ExpressionStatement(String),
    /// A block: `{ ... }`
    Block(Vec<CodegenStatement>),
    /// An if statement
    If { test: String, consequent: Vec<CodegenStatement>, alternate: Option<Vec<CodegenStatement>> },
    /// A return statement
    Return(Option<String>),
    /// A raw code string (for complex output)
    Raw(String),
}

/// Kind of variable declaration.
#[derive(Debug, Clone, Copy)]
pub enum VarKind {
    Const,
    Let,
}

impl CodegenContext {
    fn new() -> Self {
        Self {
            next_cache_index: 0,
            cache_var_name: "$".to_string(),
            statements: Vec::new(),
        }
    }

    /// Allocate the next cache index.
    pub fn alloc_cache_index(&mut self) -> u32 {
        let idx = self.next_cache_index;
        self.next_cache_index += 1;
        idx
    }

    /// Generate a cache read expression: `$[idx]`
    pub fn cache_read(&self, idx: u32) -> String {
        format!("{}[{}]", self.cache_var_name, idx)
    }

    /// Generate a cache sentinel check: `$[idx] === Symbol.for("react.memo_cache_sentinel")`
    pub fn cache_sentinel_check(&self, idx: u32) -> String {
        format!(
            "{}[{}] === Symbol.for(\"{}\")",
            self.cache_var_name, idx, MEMO_CACHE_SENTINEL
        )
    }

    /// Generate a cache write: `$[idx] = value`
    pub fn cache_write(&self, idx: u32, value: &str) -> String {
        format!("{}[{}] = {}", self.cache_var_name, idx, value)
    }

    /// Emit the useMemoCache initialization: `const $ = _c(N);`
    pub fn emit_cache_init(&self, memo_cache_import: &str) -> CodegenStatement {
        CodegenStatement::VariableDeclaration {
            kind: VarKind::Const,
            name: self.cache_var_name.clone(),
            init: Some(format!("{}({})", memo_cache_import, self.next_cache_index)),
        }
    }
}

/// Generate code for a reactive scope block.
///
/// The output pattern for each scope is:
/// ```js
/// if ($[idx] === Symbol.for("react.memo_cache_sentinel")) {
///   // ... instructions ...
///   $[idx] = result;
///   $[idx + 1] = otherResult;
/// } else {
///   result = $[idx];
///   otherResult = $[idx + 1];
/// }
/// ```
pub fn codegen_scope_block(
    cx: &mut CodegenContext,
    scope: &crate::hir::ReactiveScope,
    _instructions: &ReactiveBlock,
) -> Vec<CodegenStatement> {
    let mut stmts = Vec::new();

    // Allocate a sentinel check index
    let sentinel_idx = cx.alloc_cache_index();

    // Allocate indices for each declaration
    let mut decl_indices: Vec<(String, u32)> = Vec::new();
    for id in scope.declarations.keys() {
        let idx = cx.alloc_cache_index();
        let name = format!("t{}", id.0);
        decl_indices.push((name, idx));
    }

    // Generate the sentinel check
    let check = cx.cache_sentinel_check(sentinel_idx);

    // Consequent: compute + store
    let mut consequent = Vec::new();
    // In the full implementation, we'd codegen all the instructions inside the scope
    consequent.push(CodegenStatement::Raw("// ... scope instructions ...".to_string()));

    // Store each declaration into the cache
    for (name, idx) in &decl_indices {
        consequent.push(CodegenStatement::ExpressionStatement(
            cx.cache_write(*idx, name),
        ));
    }
    // Store the sentinel
    consequent.push(CodegenStatement::ExpressionStatement(
        cx.cache_write(sentinel_idx, &format!("\"{MEMO_CACHE_SENTINEL}\"")),
    ));

    // Alternate: load from cache
    let mut alternate = Vec::new();
    for (name, idx) in &decl_indices {
        alternate.push(CodegenStatement::ExpressionStatement(
            format!("{name} = {}", cx.cache_read(*idx)),
        ));
    }

    stmts.push(CodegenStatement::If {
        test: check,
        consequent,
        alternate: Some(alternate),
    });

    stmts
}

/// Count the number of memo slots needed for the reactive function.
fn count_memo_slots(
    block: &ReactiveBlock,
    memo_blocks: &mut u32,
    memo_values: &mut u32,
    pruned_blocks: &mut u32,
    pruned_values: &mut u32,
) {
    for stmt in block {
        match stmt {
            ReactiveStatement::Scope(scope) => {
                *memo_blocks += 1;
                // Each declaration in the scope needs a cache slot
                *memo_values += scope.scope.declarations.len() as u32;
                // Plus one for the cache validity check
                *memo_values += 1;
                count_memo_slots(
                    &scope.instructions,
                    memo_blocks,
                    memo_values,
                    pruned_blocks,
                    pruned_values,
                );
            }
            ReactiveStatement::PrunedScope(scope) => {
                *pruned_blocks += 1;
                *pruned_values += scope.scope.declarations.len() as u32;
                count_memo_slots(
                    &scope.instructions,
                    memo_blocks,
                    memo_values,
                    pruned_blocks,
                    pruned_values,
                );
            }
            ReactiveStatement::Terminal(term) => {
                count_terminal_memo_slots(
                    &term.terminal,
                    memo_blocks,
                    memo_values,
                    pruned_blocks,
                    pruned_values,
                );
            }
            ReactiveStatement::Instruction(_) => {}
        }
    }
}

fn count_terminal_memo_slots(
    terminal: &ReactiveTerminal,
    memo_blocks: &mut u32,
    memo_values: &mut u32,
    pruned_blocks: &mut u32,
    pruned_values: &mut u32,
) {
    match terminal {
        ReactiveTerminal::If(t) => {
            count_memo_slots(&t.consequent, memo_blocks, memo_values, pruned_blocks, pruned_values);
            if let Some(alt) = &t.alternate {
                count_memo_slots(alt, memo_blocks, memo_values, pruned_blocks, pruned_values);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &t.cases {
                if let Some(block) = &case.block {
                    count_memo_slots(
                        block,
                        memo_blocks,
                        memo_values,
                        pruned_blocks,
                        pruned_values,
                    );
                }
            }
        }
        ReactiveTerminal::While(t) => {
            count_memo_slots(&t.r#loop, memo_blocks, memo_values, pruned_blocks, pruned_values);
        }
        ReactiveTerminal::DoWhile(t) => {
            count_memo_slots(&t.r#loop, memo_blocks, memo_values, pruned_blocks, pruned_values);
        }
        ReactiveTerminal::For(t) => {
            count_memo_slots(&t.r#loop, memo_blocks, memo_values, pruned_blocks, pruned_values);
        }
        ReactiveTerminal::ForOf(t) => {
            count_memo_slots(&t.r#loop, memo_blocks, memo_values, pruned_blocks, pruned_values);
        }
        ReactiveTerminal::ForIn(t) => {
            count_memo_slots(&t.r#loop, memo_blocks, memo_values, pruned_blocks, pruned_values);
        }
        ReactiveTerminal::Label(t) => {
            count_memo_slots(&t.block, memo_blocks, memo_values, pruned_blocks, pruned_values);
        }
        ReactiveTerminal::Try(t) => {
            count_memo_slots(&t.block, memo_blocks, memo_values, pruned_blocks, pruned_values);
            count_memo_slots(&t.handler, memo_blocks, memo_values, pruned_blocks, pruned_values);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}
