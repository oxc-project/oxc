use oxc_allocator::Vec as OxcVec;
use oxc_ast::{
    AstKind,
    ast::{Argument, Expression, Statement, VariableDeclarationKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::AstNode;
use oxc_span::Span;
use oxc_str::CompactStr;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::LintContext,
    utils::{
        JestFnKind, JestGeneralFnKind, PossibleJestNode, get_node_name, is_type_of_jest_fn_call,
        valid_vitest_fn::is_valid_vitest_call,
    },
};

fn use_hook(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Require setup and teardown code to be within a hook.")
        .with_help("This should be done within a hook")
        .with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

This rule flags any expression that is either at the toplevel of a test file or
directly within the body of a `describe`, _except_ for the following:

- `import` statements
- `const` variables
- `let` _declarations_, and initializations to `null` or `undefined`
- Classes
- Types
- Calls to the standard Jest globals

### Why is this bad?

Having setup and teardown code outside of hooks can lead to unpredictable test
behavior. Code that runs at the top level executes when the test file is loaded,
not when tests run, which can cause issues with test isolation and make tests
dependent on execution order. Using proper hooks like `beforeEach`, `beforeAll`,
`afterEach`, and `afterAll` ensures that setup and teardown code runs at the
correct time and maintains test isolation.

### Examples

Examples of **incorrect** code for this rule:
```javascript
import { database, isCity } from '../database';
import { Logger } from '../../../src/Logger';
import { loadCities } from '../api';

jest.mock('../api');

const initializeCityDatabase = () => {
    database.addCity('Vienna');
    database.addCity('San Juan');
    database.addCity('Wellington');
};

const clearCityDatabase = () => {
    database.clear();
};

initializeCityDatabase();

test('that persists cities', () => {
    expect(database.cities.length).toHaveLength(3);
});
test('city database has Vienna', () => {
    expect(isCity('Vienna')).toBeTruthy();
});

test('city database has San Juan', () => {
    expect(isCity('San Juan')).toBeTruthy();
});

describe('when loading cities from the api', () => {
    let consoleWarnSpy = jest.spyOn(console, 'warn');
    loadCities.mockResolvedValue(['Wellington', 'London']);

    it('does not duplicate cities', async () => {
        await database.loadCities();
        expect(database.cities).toHaveLength(4);
    });
});
clearCityDatabase();
```

Examples of **correct** code for this rule:
```javascript
import { database, isCity } from '../database';
import { Logger } from '../../../src/Logger';
import { loadCities } from '../api';

jest.mock('../api');
const initializeCityDatabase = () => {
    database.addCity('Vienna');
    database.addCity('San Juan');
    database.addCity('Wellington');
};

const clearCityDatabase = () => {
    database.clear();
};

beforeEach(() => {
    initializeCityDatabase();
});

test('that persists cities', () => {
    expect(database.cities.length).toHaveLength(3);
});

test('city database has Vienna', () => {
    expect(isCity('Vienna')).toBeTruthy();
});

test('city database has San Juan', () => {
    expect(isCity('San Juan')).toBeTruthy();
});

describe('when loading cities from the api', () => {
    let consoleWarnSpy;
    beforeEach(() => {
        consoleWarnSpy = jest.spyOn(console, 'warn');
        loadCities.mockResolvedValue(['Wellington', 'London']);
    });

    it('does not duplicate cities', async () => {
        await database.loadCities();
        expect(database.cities).toHaveLength(4);
    });
});
afterEach(() => {
    clearCityDatabase();
});
```
";

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct RequireHookConfig {
    /// An array of function names that are allowed to be called outside of hooks.
    allowed_function_calls: Vec<CompactStr>,
}

impl RequireHookConfig {
    pub fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Program(program) => {
                self.check_block_body(&program.body, ctx);
            }
            AstKind::CallExpression(call_expr) => {
                if !is_type_of_jest_fn_call(
                    call_expr,
                    &PossibleJestNode { node, original: None },
                    ctx,
                    &[JestFnKind::General(JestGeneralFnKind::Describe)],
                ) || call_expr.arguments.len() < 2
                {
                    return;
                }

                match &call_expr.arguments[1] {
                    Argument::FunctionExpression(func_expr) => {
                        if let Some(func_body) = &func_expr.body {
                            self.check_block_body(&func_body.statements, ctx);
                        }
                    }
                    Argument::ArrowFunctionExpression(arrow_func_expr)
                        if !arrow_func_expr.expression =>
                    {
                        self.check_block_body(&arrow_func_expr.body.statements, ctx);
                    }
                    _ => (),
                }
            }
            _ => {}
        }
    }

    fn check_block_body<'a>(
        &self,
        statements: &'a OxcVec<'a, Statement<'_>>,
        ctx: &LintContext<'a>,
    ) {
        for stmt in statements {
            self.check(stmt, ctx);
        }
    }

    fn check<'a>(&self, stmt: &'a Statement<'_>, ctx: &LintContext<'a>) {
        if let Statement::ExpressionStatement(expr_stmt) = stmt {
            self.check_should_report_in_hook(&expr_stmt.expression, ctx);
        } else if let Statement::VariableDeclaration(var_decl) = stmt
            && var_decl.kind != VariableDeclarationKind::Const
            && var_decl.declarations.iter().any(|decl| {
                let Some(init_call) = &decl.init else {
                    return false;
                };
                !init_call.is_null_or_undefined()
            })
        {
            ctx.diagnostic(use_hook(var_decl.span));
        }
    }

    fn check_should_report_in_hook<'a>(&self, expr: &'a Expression<'a>, ctx: &LintContext<'a>) {
        if let Expression::CallExpression(call_expr) = expr {
            let name = get_node_name(&call_expr.callee);

            let node_name_split: Vec<&str> = name.split('.').collect();

            let Some(fn_type) = node_name_split.first() else {
                return;
            };

            if !(is_valid_vitest_call(&[fn_type])
                || name.starts_with("jest.")
                || name.starts_with("vi.")
                || self.allowed_function_calls.contains(&name))
            {
                ctx.diagnostic(use_hook(call_expr.span));
            }
        }
    }
}
