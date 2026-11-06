use oxc_ast::{
    AstKind,
    ast::{
        Argument, BindingPattern, CallExpression, Expression, IdentifierReference,
        ImportDeclarationSpecifier, ObjectExpression, ObjectPropertyKind, PropertyKind,
        UnaryOperator, VariableDeclarationKind,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;

use crate::{AstNode, context::LintContext, rule::Rule};

fn missing_timeout_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`{name}()` is called without a timeout."))
        .with_help(format!(
            "Pass a positive `timeout` option to `{name}()` to bound its execution."
        ))
        .with_label(span)
}

fn missing_timeout_or_signal_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`{name}()` is called without a timeout or an abort signal."))
        .with_help(format!(
            "Pass a positive `timeout` option or an `AbortSignal` as the `signal` option to `{name}()` to bound its execution."
        ))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireChildProcessTimeout;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires a timeout or an abort signal for child processes started with
    /// `node:child_process`.
    ///
    /// `execSync`, `execFileSync` and `spawnSync` need a positive `timeout`.
    /// `exec`, `execFile`, `spawn` and `fork` need a positive `timeout` or a `signal`.
    ///
    /// ### Why is this bad?
    ///
    /// A child process can wait indefinitely for a daemon, a network connection or input.
    /// Callers waiting for that process can get stuck too. A timeout or an abort signal
    /// lets the application bound how long it waits.
    ///
    /// The rule only reports options that are provably unbounded. Dynamic options,
    /// unknown spreads, unknown arguments and reassigned bindings are skipped.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import { execSync, execFile, spawn } from "node:child_process";
    /// import { promisify } from "node:util";
    ///
    /// execSync("docker ps");
    /// promisify(execFile)("gh", ["api", "user"]);
    /// spawn("docker", ["logs", "-f", "container"], { timeout: 0 });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import { execSync, execFile, spawn } from "node:child_process";
    /// import { promisify } from "node:util";
    ///
    /// execSync("docker ps", { timeout: 10_000 });
    /// promisify(execFile)("gh", ["api", "user"], { timeout: 60_000 });
    /// spawn("docker", ["logs", "-f", "container"], {
    ///   signal: AbortSignal.timeout(60_000),
    /// });
    /// ```
    RequireChildProcessTimeout,
    node,
    restriction,
    version = "next",
    short_description = "Require a timeout or signal for child processes.",
);

const METHODS: [&str; 7] =
    ["exec", "execFile", "execSync", "execFileSync", "spawn", "spawnSync", "fork"];

impl Rule for RequireChildProcessTimeout {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call) = node.kind() else {
            return;
        };
        let resolver = Resolver { ctx };
        if let Some(Value::Method(name) | Value::Promisified(name)) =
            resolver.value_of(&call.callee, &mut FxHashSet::default())
        {
            resolver.check_call(call, name);
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.source_text().contains("child_process")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Module {
    ChildProcess,
    Util,
}

/// What an expression refers to, as far as this rule is concerned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Value {
    Module(Module),
    /// A `child_process` method, such as `exec`.
    Method(&'static str),
    /// `util.promisify`.
    Promisify,
    /// The result of `util.promisify(exec)` or `util.promisify(execFile)`.
    Promisified(&'static str),
}

/// A statically known value, as far as this rule needs to know it.
#[derive(Debug, Clone, PartialEq)]
enum Constant {
    Undefined,
    Null,
    Number(f64),
    String(String),
    /// Any other primitive: a boolean, a bigint, ...
    OtherPrimitive,
    /// An array or object literal.
    Object {
        array: bool,
    },
}

impl Constant {
    fn is_nullish(&self) -> bool {
        matches!(self, Self::Undefined | Self::Null)
    }
}

/// An expression with local `const` aliases and function names followed.
enum Resolved<'a> {
    Expression(&'a Expression<'a>),
    FunctionDeclaration,
}

/// Whether a `timeout` or `signal` option bounds the call.
/// `None` means it cannot be determined statically.
type Bound = Option<bool>;

struct Resolver<'c, 'a> {
    ctx: &'c LintContext<'a>,
}

impl<'a> Resolver<'_, 'a> {
    fn symbol_of(&self, ident: &IdentifierReference) -> Option<SymbolId> {
        self.ctx.scoping().get_reference(ident.reference_id()).symbol_id()
    }

    fn is_global(&self, ident: &IdentifierReference, name: &str) -> bool {
        ident.name == name && self.ctx.is_reference_to_global_variable(ident)
    }

    /// `require("child_process")` or `process.getBuiltinModule("child_process")`.
    fn module_of_call(&self, call: &'a CallExpression<'a>) -> Option<Module> {
        if call.optional || call.arguments.len() != 1 {
            return None;
        }
        let callee = call.callee.without_parentheses();
        let is_loader = match callee {
            Expression::Identifier(ident) => self.is_global(ident, "require"),
            Expression::StaticMemberExpression(member) => {
                member.property.name == "getBuiltinModule"
                    && matches!(&member.object, Expression::Identifier(object) if self.is_global(object, "process"))
            }
            _ => false,
        };
        if !is_loader {
            return None;
        }
        let Some(Constant::String(source)) = call.arguments[0]
            .as_expression()
            .and_then(|argument| self.constant_of(argument, &mut FxHashSet::default()))
        else {
            return None;
        };
        module_from_source(&source)
    }

    fn value_of(&self, expr: &'a Expression<'a>, seen: &mut FxHashSet<SymbolId>) -> Option<Value> {
        match expr.without_parentheses() {
            Expression::Identifier(ident) => {
                let symbol_id = self.symbol_of(ident)?;
                if !seen.insert(symbol_id)
                    || self.is_symbol_modified(symbol_id, &mut FxHashSet::default())
                {
                    return None;
                }
                self.value_of_symbol(symbol_id, seen)
            }
            Expression::CallExpression(call) => {
                if let Some(module) = self.module_of_call(call) {
                    return Some(Value::Module(module));
                }
                if call.optional
                    || call.arguments.len() != 1
                    || self.is_modified(&call.callee)
                    || self.value_of(&call.callee, seen)? != Value::Promisify
                {
                    return None;
                }
                let argument = call.arguments[0].as_expression()?;
                if self.is_modified(argument) {
                    return None;
                }
                match self.value_of(argument, seen)? {
                    Value::Method(name @ ("exec" | "execFile")) => Some(Value::Promisified(name)),
                    _ => None,
                }
            }
            member @ (Expression::StaticMemberExpression(_)
            | Expression::ComputedMemberExpression(_)) => {
                let member = member.as_member_expression()?;
                let name = member.static_property_name()?;
                let Value::Module(module) = self.value_of(member.object(), seen)? else {
                    return None;
                };
                member_of(module, name.as_str()?)
            }
            _ => None,
        }
    }

    fn value_of_symbol(
        &self,
        symbol_id: SymbolId,
        seen: &mut FxHashSet<SymbolId>,
    ) -> Option<Value> {
        let declaration =
            self.ctx.nodes().get_node(self.ctx.scoping().symbol_declaration(symbol_id));
        match declaration.kind() {
            AstKind::ImportSpecifier(_)
            | AstKind::ImportDefaultSpecifier(_)
            | AstKind::ImportNamespaceSpecifier(_) => {
                let AstKind::ImportDeclaration(import) =
                    self.ctx.nodes().parent_kind(declaration.id())
                else {
                    return None;
                };
                if import.import_kind.is_type() {
                    return None;
                }
                let module = module_from_source(import.source.value.as_str()?)?;
                let specifier = import
                    .specifiers
                    .as_ref()?
                    .iter()
                    .find(|specifier| specifier.local().symbol_id() == symbol_id)?;
                match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                        if specifier.import_kind.is_type() {
                            return None;
                        }
                        member_of(module, specifier.imported.name().as_str())
                    }
                    // A default import is the module's `module.exports`, as in CommonJS.
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(_)
                    | ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                        Some(Value::Module(module))
                    }
                }
            }
            AstKind::VariableDeclarator(declarator) => {
                let init = declarator.init.as_ref()?;
                match &declarator.id {
                    BindingPattern::BindingIdentifier(binding)
                        if binding.symbol_id() == symbol_id =>
                    {
                        self.value_of(init, seen)
                    }
                    BindingPattern::ObjectPattern(pattern) => {
                        let property = pattern
                            .properties
                            .iter()
                            .find(|property| binding_symbol(&property.value) == Some(symbol_id))?;
                        let key = if property.computed {
                            match self.constant_of(
                                property.key.as_expression()?,
                                &mut FxHashSet::default(),
                            )? {
                                Constant::String(key) => key,
                                _ => return None,
                            }
                        } else {
                            property.key.static_name()?.as_str()?.to_string()
                        };
                        let Value::Module(module) = self.value_of(init, seen)? else {
                            return None;
                        };
                        member_of(module, &key)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Whether the binding behind `expr` may no longer refer to what it was initialized with.
    fn is_modified(&self, expr: &Expression<'a>) -> bool {
        let mut expr = expr.without_parentheses();
        while let Some(member) = expr.as_member_expression() {
            expr = member.object().without_parentheses();
        }
        match expr {
            Expression::Identifier(ident) => self.symbol_of(ident).is_some_and(|symbol_id| {
                self.is_symbol_modified(symbol_id, &mut FxHashSet::default())
            }),
            _ => false,
        }
    }

    fn is_symbol_modified(&self, symbol_id: SymbolId, seen: &mut FxHashSet<SymbolId>) -> bool {
        if !seen.insert(symbol_id) {
            return false;
        }
        let nodes = self.ctx.nodes();
        for reference in self.ctx.symbol_references(symbol_id) {
            if reference.is_write() {
                return true;
            }
            // Walk up to the outermost member expression with this reference as its object,
            // so that `options.timeout = 1` and `cp.exec = custom` count as modifications.
            let mut target = nodes.get_node(reference.node_id());
            loop {
                let parent = nodes.parent_node(target.id());
                let is_object = match parent.kind() {
                    AstKind::StaticMemberExpression(member) => {
                        member.object.span() == target.span()
                    }
                    AstKind::ComputedMemberExpression(member) => {
                        member.object.span() == target.span()
                    }
                    AstKind::PrivateFieldExpression(member) => {
                        member.object.span() == target.span()
                    }
                    _ => false,
                };
                if !is_object {
                    break;
                }
                target = parent;
            }
            let target_span = target.span();
            match nodes.parent_kind(target.id()) {
                AstKind::AssignmentExpression(assignment)
                    if assignment.left.span() == target_span =>
                {
                    return true;
                }
                AstKind::UpdateExpression(_) => return true,
                AstKind::UnaryExpression(unary) if unary.operator == UnaryOperator::Delete => {
                    return true;
                }
                // A writable alias can modify the same object through its properties.
                AstKind::VariableDeclarator(declarator)
                    if declarator.init.as_ref().is_some_and(|init| init.span() == target_span) =>
                {
                    if let BindingPattern::BindingIdentifier(alias) = &declarator.id
                        && self.is_symbol_modified(alias.symbol_id(), seen)
                    {
                        return true;
                    }
                }
                _ => {}
            }
        }
        let declaration = nodes.get_node(self.ctx.scoping().symbol_declaration(symbol_id));
        if let AstKind::VariableDeclarator(declarator) = declaration.kind()
            && let Some(init) = &declarator.init
        {
            let mut init = init.without_parentheses();
            while let Some(member) = init.as_member_expression() {
                init = member.object().without_parentheses();
            }
            if let Expression::Identifier(ident) = init
                && let Some(init_symbol_id) = self.symbol_of(ident)
            {
                return self.is_symbol_modified(init_symbol_id, seen);
            }
        }
        false
    }

    /// Follows unchanged local `const` aliases, without evaluating code.
    fn resolve(
        &self,
        expr: &'a Expression<'a>,
        seen: &mut FxHashSet<SymbolId>,
    ) -> Option<Resolved<'a>> {
        let Expression::Identifier(ident) = expr.without_parentheses() else {
            return Some(Resolved::Expression(expr.without_parentheses()));
        };
        let symbol_id = self.symbol_of(ident)?;
        if !seen.insert(symbol_id) || self.is_symbol_modified(symbol_id, &mut FxHashSet::default())
        {
            return None;
        }
        let declaration =
            self.ctx.nodes().get_node(self.ctx.scoping().symbol_declaration(symbol_id));
        match declaration.kind() {
            AstKind::Function(function) if function.is_declaration() => {
                Some(Resolved::FunctionDeclaration)
            }
            AstKind::VariableDeclarator(declarator) => match (&declarator.id, &declarator.init) {
                (BindingPattern::BindingIdentifier(binding), Some(init))
                    if binding.symbol_id() == symbol_id =>
                {
                    self.resolve(init, seen)
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn is_callback(&self, expr: &'a Expression<'a>) -> bool {
        match self.resolve(expr, &mut FxHashSet::default()) {
            Some(Resolved::FunctionDeclaration) => true,
            Some(Resolved::Expression(expr)) => matches!(
                expr,
                Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_)
            ),
            None => false,
        }
    }

    fn resolve_object(&self, expr: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
        match self.resolve(expr, &mut FxHashSet::default())? {
            Resolved::Expression(Expression::ObjectExpression(object)) => Some(object),
            _ => None,
        }
    }

    fn resolves_to_array(&self, expr: &'a Expression<'a>) -> bool {
        matches!(
            self.resolve(expr, &mut FxHashSet::default()),
            Some(Resolved::Expression(Expression::ArrayExpression(_)))
        )
    }

    /// A small subset of constant evaluation: literals, `undefined`, unary signs,
    /// `void`, array and object literals with constant contents, and `const` bindings.
    fn constant_of(
        &self,
        expr: &'a Expression<'a>,
        seen: &mut FxHashSet<SymbolId>,
    ) -> Option<Constant> {
        match expr.without_parentheses() {
            Expression::NullLiteral(_) => Some(Constant::Null),
            Expression::BooleanLiteral(_) | Expression::BigIntLiteral(_) => {
                Some(Constant::OtherPrimitive)
            }
            Expression::NumericLiteral(number) => Some(Constant::Number(number.value)),
            Expression::StringLiteral(string) => {
                Some(Constant::String(string.value.as_str()?.to_string()))
            }
            Expression::TemplateLiteral(template) => {
                Some(Constant::String(template.single_quasi()?.as_str()?.to_string()))
            }
            Expression::UnaryExpression(unary) => match unary.operator {
                UnaryOperator::Void => Some(Constant::Undefined),
                UnaryOperator::UnaryNegation | UnaryOperator::UnaryPlus => {
                    let Constant::Number(number) = self.constant_of(&unary.argument, seen)? else {
                        return None;
                    };
                    Some(Constant::Number(if unary.operator == UnaryOperator::UnaryNegation {
                        -number
                    } else {
                        number
                    }))
                }
                _ => None,
            },
            Expression::ArrayExpression(array) => {
                for element in &array.elements {
                    let expression = element.as_expression()?;
                    self.constant_of(expression, seen)?;
                }
                Some(Constant::Object { array: true })
            }
            Expression::ObjectExpression(object) => {
                for property in &object.properties {
                    let ObjectPropertyKind::ObjectProperty(property) = property else {
                        return None;
                    };
                    if property.kind != PropertyKind::Init || property.method || property.computed {
                        return None;
                    }
                    self.constant_of(&property.value, seen)?;
                }
                Some(Constant::Object { array: false })
            }
            Expression::Identifier(ident) => {
                if self.is_global(ident, "undefined") {
                    return Some(Constant::Undefined);
                }
                let symbol_id = self.symbol_of(ident)?;
                if !seen.insert(symbol_id) {
                    return None;
                }
                let declaration =
                    self.ctx.nodes().get_node(self.ctx.scoping().symbol_declaration(symbol_id));
                let AstKind::VariableDeclarator(declarator) = declaration.kind() else {
                    return None;
                };
                if !matches!(
                    self.ctx.nodes().parent_kind(declaration.id()),
                    AstKind::VariableDeclaration(declaration)
                        if declaration.kind == VariableDeclarationKind::Const
                ) {
                    return None;
                }
                match (&declarator.id, &declarator.init) {
                    (BindingPattern::BindingIdentifier(binding), Some(init))
                        if binding.symbol_id() == symbol_id =>
                    {
                        self.constant_of(init, seen)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn check_call(&self, call: &'a CallExpression<'a>, name: &'static str) {
        if call.arguments.is_empty()
            || call.arguments.iter().any(Argument::is_spread)
            || self.is_modified(&call.callee)
        {
            return;
        }
        let argument = |index: usize| call.arguments.get(index).and_then(Argument::as_expression);
        let mut options = argument(1);
        if !matches!(name, "exec" | "execSync")
            && let Some(second) = options
        {
            // The second argument of these methods is either the argument list or the options.
            let constant = self.constant_of(second, &mut FxHashSet::default());
            if self.resolves_to_array(second)
                || constant.as_ref().is_some_and(|constant| {
                    matches!(constant, Constant::Object { array: true }) || constant.is_nullish()
                })
            {
                options = argument(2);
            } else if self.resolve_object(second).is_none() && !self.is_callback(second) {
                // An unknown second argument might be either the arguments or the options.
                return;
            }
        }
        let sync = name.ends_with("Sync");
        if self.is_unbounded(options, sync) {
            ctx_report(self.ctx, call.span, name, sync);
        }
    }

    /// Only report options that are provably unbounded. Unknown spreads or computed
    /// properties can supply either option, in property order.
    fn is_unbounded(&self, options: Option<&'a Expression<'a>>, sync: bool) -> bool {
        let Some(options) = options else {
            return true;
        };
        if self.is_callback(options)
            || self.constant_of(options, &mut FxHashSet::default()).is_some_and(|c| c.is_nullish())
        {
            return true;
        }
        let Some(object) = self.resolve_object(options) else {
            return false;
        };
        let mut timeout: Bound = Some(false);
        let mut signal: Bound = Some(false);
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                timeout = None;
                signal = None;
                continue;
            };
            let key = if property.computed {
                match property
                    .key
                    .as_expression()
                    .and_then(|key| self.constant_of(key, &mut FxHashSet::default()))
                {
                    Some(Constant::String(key)) => Some(key),
                    _ => None,
                }
            } else {
                property.key.static_name().and_then(|key| key.as_str().map(str::to_string))
            };
            let Some(key) = key else {
                timeout = None;
                signal = None;
                continue;
            };
            if key != "timeout" && key != "signal" {
                continue;
            }
            let constant = if property.kind == PropertyKind::Init && !property.method {
                self.constant_of(&property.value, &mut FxHashSet::default())
            } else {
                None
            };
            let bound = constant.map(|constant| {
                if key == "timeout" {
                    matches!(constant, Constant::Number(number) if number.is_finite() && number > 0.0)
                } else {
                    matches!(constant, Constant::Object { .. })
                }
            });
            if key == "timeout" {
                timeout = bound;
            } else {
                signal = bound;
            }
        }
        timeout == Some(false) && (sync || signal == Some(false))
    }
}

fn ctx_report(ctx: &LintContext, span: Span, name: &str, sync: bool) {
    ctx.diagnostic(if sync {
        missing_timeout_diagnostic(span, name)
    } else {
        missing_timeout_or_signal_diagnostic(span, name)
    });
}

fn module_from_source(source: &str) -> Option<Module> {
    match source.strip_prefix("node:").unwrap_or(source) {
        "child_process" => Some(Module::ChildProcess),
        "util" => Some(Module::Util),
        _ => None,
    }
}

fn member_of(module: Module, name: &str) -> Option<Value> {
    match module {
        Module::ChildProcess => {
            METHODS.iter().find(|method| **method == name).map(|m| Value::Method(m))
        }
        Module::Util => (name == "promisify").then_some(Value::Promisify),
    }
}

/// The symbol bound directly by a destructuring property value, such as `run` in
/// `{ exec: run }` or `{ exec: run = fallback }`.
fn binding_symbol(pattern: &BindingPattern) -> Option<SymbolId> {
    match pattern {
        BindingPattern::BindingIdentifier(binding) => Some(binding.symbol_id()),
        BindingPattern::AssignmentPattern(assignment) => binding_symbol(&assignment.left),
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    const ASYNC_METHODS: [&str; 4] = ["exec", "execFile", "spawn", "fork"];
    const SYNC_METHODS: [&str; 3] = ["execSync", "execFileSync", "spawnSync"];
    // Methods whose second argument may be the argument list.
    let with_args = METHODS.iter().filter(|name| !matches!(**name, "exec" | "execSync"));

    let mut pass: Vec<String> = Vec::new();
    for name in with_args.clone() {
        pass.push(format!(
            "import {{ {name} }} from 'child_process'; {name}('cmd', undefined, {{ timeout: 1000 }})"
        ));
    }
    for name in METHODS {
        pass.push(format!(
            "import {{ {name} as run }} from 'node:child_process'; run('command', {{ timeout: 1000 }})"
        ));
        pass.push(format!(
            "const cp = require('child_process'); cp.{name}('command', {{ timeout: duration }})"
        ));
        pass.push(format!("import * as cp from 'child_process'; cp.{name}('command', options)"));
    }
    for name in ASYNC_METHODS {
        pass.push(format!(
            "import {{ {name} }} from 'child_process'; {name}('command', {{ signal: controller.signal }})"
        ));
    }
    pass.extend(
        [
            "import { exec } from 'child_process'; exec('cmd', { timeout: 0, signal: controller.signal })",
            "import { exec } from 'child_process'; exec('cmd', { ...options })",
            "import { exec } from 'child_process'; exec('cmd', { timeout: 0, ...options })",
            "import { exec } from 'child_process'; exec('cmd', { ...options, timeout: 0 })",
            "import { exec } from 'child_process'; exec('cmd', { [key]: 1000 })",
            "import { exec } from 'child_process'; exec('cmd', { get timeout() { return 1000 } })",
            "import { exec } from 'child_process'; exec('cmd', { timeout: 1000 }, callback)",
            "import { exec } from 'child_process'; exec(...args)",
            "import { exec } from 'child_process'; exec('cmd', callback)",
            "import { spawn } from 'child_process'; spawn('cmd', args, options)",
            "import { spawn } from 'child_process'; spawn('cmd', [], { timeout: 1000 })",
            "import { exec } from 'child_process'; const options = { signal: controller.signal }; exec('cmd', options)",
            "import { exec } from 'child_process'; const options = {}; options.timeout = 1000; exec('cmd', options)",
            "import { exec } from 'child_process'; const options = {}; const alias = options; alias.timeout = 1000; exec('cmd', options)",
            "import { exec } from 'child_process'; const options = {}; const alias = options; const next = alias; next.timeout = 1000; exec('cmd', options)",
            "import { exec } from 'child_process'; let options = {}; options = getOptions(); exec('cmd', options)",
            "const exec = command => command; exec('cmd')",
            "const cp = { exec() {} }; cp.exec('cmd')",
            "function run(require) { require('child_process').exec('cmd') }",
            "import { exec } from 'somewhere-else'; exec('cmd')",
            "const { exec } = require('child_process'); function run(exec) { exec('cmd') }",
            "let { exec } = require('child_process'); exec = customExec; exec('cmd')",
            "const cp = require('child_process'); cp.exec = customExec; cp.exec('cmd')",
            "const cp = require('child_process'); const run = cp.exec; cp.exec = customExec; run('cmd')",
            "import { execFile } from 'child_process'; import { promisify } from 'util'; promisify(execFile)('cmd', [], { timeout: 1000 })",
            "import { exec } from 'child_process'; import { promisify } from 'node:util'; const run = promisify(exec); run('cmd', { signal: controller.signal })",
            "import { exec } from 'child_process'; function promisify(fn) { return customExec } promisify(exec)('cmd')",
            "import { exec } from 'child_process'; const { promisify } = require('custom-util'); promisify(exec)('cmd')",
            "import { exec } from 'child_process'; import { promisify } from 'util'; let run = promisify(exec); run = customExec; run('cmd')",
            // Not part of the upstream tests.
            "import type { exec } from 'child_process'; exec('cmd')",
        ]
        .map(String::from),
    );

    let mut fail: Vec<String> = Vec::new();
    for name in with_args.clone() {
        fail.push(format!("import {{ {name} }} from 'child_process'; {name}('cmd', undefined)"));
    }
    fail.extend(
        [
            "import { exec } from 'child_process'; exec('cmd', { signal: false })",
            "import { exec } from 'child_process'; exec('cmd', { signal: 0 })",
        ]
        .map(String::from),
    );
    for name in METHODS {
        fail.push(format!("import {{ {name} }} from 'node:child_process'; {name}('command')"));
        fail.push(format!("import * as cp from 'child_process'; cp.{name}('command', {{}})"));
        fail.push(format!(
            "const {{ {name}: run }} = require('node:child_process'); run('command', {{ timeout: 0 }})"
        ));
        fail.push(format!("require('child_process').{name}('command', {{ timeout: undefined }})"));
        fail.push(format!(
            "import cp from 'node:child_process'; cp.{name}('command', {{ timeout: null }})"
        ));
    }
    for name in with_args {
        fail.push(format!("const cp = require('child_process'); cp.{name}('command', [])"));
        fail.push(format!(
            "const cp = require('child_process'); cp.{name}('command', ['arg'], {{}})"
        ));
    }
    for name in ASYNC_METHODS {
        fail.push(format!(
            "import {{ {name} }} from 'child_process'; {name}('cmd', {{ signal: null }})"
        ));
    }
    for name in SYNC_METHODS {
        fail.push(format!(
            "import {{ {name} }} from 'child_process'; {name}('cmd', {{ signal: controller.signal }})"
        ));
    }
    fail.extend(
        [
            "import { exec } from 'child_process'; exec('cmd', () => {})",
            "import { exec } from 'child_process'; exec('cmd', {}, () => {})",
            "import { execFile } from 'child_process'; execFile('cmd', [], () => {})",
            "import { exec } from 'child_process'; function done() {} exec('cmd', done)",
            "import { exec } from 'child_process'; const options = {}; const alias = options; exec('cmd', alias)",
            "import { exec } from 'child_process'; exec('cmd', { ['timeout']: 0 })",
            "import { exec } from 'child_process'; const key = 'signal'; exec('cmd', { [key]: undefined })",
            "import { exec } from 'child_process'; exec('cmd', { timeout: -1 })",
            "import { exec } from 'child_process'; exec('cmd', { timeout: 1000, timeout: 0 })",
            "import { exec } from 'child_process'; exec('cmd', { signal: controller.signal, signal: null })",
            "import { execSync } from 'child_process'; execSync('cmd', { ...options, timeout: 0 })",
            "const cp = process.getBuiltinModule('node:child_process'); cp.exec('cmd')",
            "import { exec as run } from 'child_process'; const alias = run; alias('cmd')",
            "import { exec } from 'child_process'; import { promisify } from 'util'; promisify(exec)('cmd')",
            "import { execFile as run } from 'node:child_process'; import { promisify as promise } from 'node:util'; const execute = promise(run); execute('cmd', [])",
            "const { exec } = require('child_process'); const { promisify } = require('util'); const run = promisify(exec); run('cmd')",
            "const cp = require('node:child_process'); const util = require('node:util'); util.promisify(cp.exec)('cmd')",
            "import * as cp from 'child_process'; import util from 'util'; const run = util.promisify(cp.exec); run('cmd')",
            "const cp = require('child_process'); const run = cp.exec; require('util').promisify(run)('cmd')",
        ]
        .map(String::from),
    );

    Tester::new(RequireChildProcessTimeout::NAME, RequireChildProcessTimeout::PLUGIN, pass, fail)
        .test_and_snapshot();
}
