use std::rc::Rc;

use bitflags::bitflags;
use oxc_ast::{ast::*, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{ScopeId, SymbolId};
use oxc_span::{Atom, Span};
use regex::Regex;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule, AstNode};

// const ARGUMENTS_STR: Atom = Atom::from("arguments");
const DECLARED_STR: &'static Atom = &Atom::new_inline("declared");
const EMPTY_STR: &'static Atom = &Atom::new_inline("");

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-unused-vars): Unused variables are not allowed")]
#[diagnostic(severity(warning), help("{0} is {1} but never used{2}"))]
struct NoUnusedVarsDiagnostic(
    /* varName*/ pub Atom,
    /* action */ pub &'static Atom,
    /* additional */ pub Atom,
    #[label] pub Span,
);
impl NoUnusedVarsDiagnostic {
    /// Diagnostic for unused declaration, with no additional message.
    pub fn decl(var: Atom, span: Span) -> Self {
        Self(var, &DECLARED_STR, EMPTY_STR.clone(), span)
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum VarsOption {
    /// All variables are checked for usage, including those in the global scope.
    #[default]
    All,
    /// Checks only that locally-declared variables are used but will allow
    /// global variables to be unused.
    Local,
}

impl TryFrom<&String> for VarsOption {
    type Error = String;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "all" => Ok(Self::All),
            "local" => Ok(Self::Local),
            _ => Err(format!("Expected 'all' or 'local', got {value}")),
        }
    }
}

impl TryFrom<&Value> for VarsOption {
    type Error = String;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => Self::try_from(s),
            _ => Err(format!("Expected a string, got {value}")),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub enum ArgsOption {
    /// Unused positional arguments that occur before the last used argument
    /// will not be checked, but all named arguments and all positional
    /// arguments after the last used argument will be checked.
    #[default]
    AfterUsed,
    /// All named arguments must be used
    All,
    /// Do not check arguments
    None,
}

impl TryFrom<&Value> for ArgsOption {
    type Error = String;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::String(s) => match s.as_str() {
                "after-used" => Ok(Self::AfterUsed),
                "all" => Ok(Self::All),
                "none" => Ok(Self::None),
                _ => Err(format!("Expected 'after-used', 'all', or 'none', got '{s}")),
            },
            _ => Err(format!("Expected a string, got {value}")),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct NoUnusedVars {
    /// Controls how usage of a variable in the global scope is checked.
    ///
    /// This option has two settings:
    /// 1. `all` checks all variables for usage, including those in the global
    ///    scope. This is the default setting.
    /// 2. `local` checks only that locally-declared variables are used but will
    ///    allow global variables to be unused.
    vars: VarsOption,
    /// Specifies exceptions to this rule for unused variables. Variables whose
    /// names match this pattern will be ignored.
    ///
    /// ## Example
    ///
    /// Examples of **correct** code for this option when the pattern is `^_`:
    /// ```javascript
    /// var _a = 10;
    /// var b = 10;
    /// console.log(b);
    /// ```
    vars_ignore_pattern: Option<Regex>,
    /// Controls how unused arguments are checked.
    ///
    /// This option has three settings:
    /// 1. `after-used` - Unused positional arguments that occur before the last
    ///    used argument will not be checked, but all named arguments and all
    ///    positional arguments after the last used argument will be checked.
    /// 2. `all` - All named arguments must be used.
    /// 3. `none` - Do not check arguments.
    args: ArgsOption,
    /// Specifies exceptions to this rule for unused arguments. Arguments whose
    /// names match this pattern will be ignored.
    ///
    /// ## Example
    ///
    /// Examples of **correct** code for this option when the pattern is `^_`:
    ///
    /// ```javascript
    /// function foo(_a, b) {
    ///    console.log(b);
    /// }
    /// foo(1, 2);
    /// ```
    args_ignore_pattern: Option<Regex>,
    /// Used for `catch` block validation.
    /// It has two settings:
    /// * `none` - do not check error objects. This is the default setting
    /// * `all` - all named arguments must be used`
    ///
    #[doc(hidden)]
    /// `none` corresponds to `false`, while `all` corresponds to `true`.
    caught_errors: bool,
    /// Specifies exceptions to this rule for errors caught within a `catch` block.
    /// Variables declared within a `catch` block whose names match this pattern
    /// will be ignored.
    ///
    /// ## Example
    ///
    /// Examples of **correct** code when the pattern is `^ignore`:
    ///
    /// ```javascript
    /// try {
    ///   // ...
    /// } catch (ignoreErr) {
    ///   console.error("Error caught in catch block");
    /// }
    /// ```
    caught_errors_ignore_pattern: Option<Regex>,
    /// This option specifies exceptions within destructuring patterns that will
    /// not be checked for usage. Variables declared within array destructuring
    /// whose names match this pattern will be ignored.
    ///
    /// ## Example
    ///
    /// Examples of **correct** code for this option, when the pattern is `^_`:
    /// ```javascript
    /// const [a, _b, c] = ["a", "b", "c"];
    /// console.log(a + c);
    ///
    /// const { x: [_a, foo] } = bar;
    /// console.log(foo);
    ///
    /// let _m, n;
    /// foo.forEach(item => {
    ///     [_m, n] = item;
    ///     console.log(n);
    /// });
    /// ```
    destructured_array_ignore_pattern: Option<Regex>,
    /// Using a Rest property it is possible to "omit" properties from an
    /// object, but by default the sibling properties are marked as "unused".
    /// With this option enabled the rest property's siblings are ignored.
    ignore_rest_siblings: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows variable declarations or imports that are not used in code.
    ///
    /// ### Why is this bad?
    ///
    /// Variables that are declared and not used anywhere in the code are most
    /// likely an error due to incomplete refactoring. Such variables take up
    /// space in the code and can lead to confusion by readers.
    ///
    /// A variable `foo` is considered to be used if any of the following are
    /// true:
    ///
    /// * It is called (`foo()`) or constructed (`new foo()`)
    /// * It is read (`var bar = foo`)
    /// * It is passed into a function as an argument (`doSomething(foo)`)
    /// * It is read inside of a function that is passed to another function
    ///   (`doSomething(function() { foo(); })`)
    ///
    /// A variable is _not_ considered to be used if it is only ever declared
    /// (`var foo = 5`) or assigned to (`foo = 7`).
    ///
    /// #### Exported
    ///
    /// In environments outside of CommonJS or ECMAScript modules, you may use
    /// `var` to create a global variable that may be used by other scripts. You
    /// can use the `/* exported variableName */` comment block to indicate that
    /// this variable is being exported and therefore should not be considered
    /// unused.
    ///
    /// Note that `/* exported */` has no effect for any of the following:
    /// * when the environment is `node` or `commonjs`
    /// * when `parserOptions.sourceType` is `module`
    /// * when `ecmaFeatures.globalReturn` is `true`
    ///
    /// The line comment `//exported variableName` will not work as `exported`
    /// is not line-specific.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    /// ```javascript
    /// /*eslint no-unused-vars: "error"*/
    /// /*global some_unused_var*/
    ///
    /// // It checks variables you have defined as global
    /// some_unused_var = 42;
    ///
    /// var x;
    ///
    /// // Write-only variables are not considered as used.
    /// var y = 10;
    /// y = 5;
    ///
    /// // A read for a modification of itself is not considered as used.
    /// var z = 0;
    /// z = z + 1;
    ///
    /// // By default, unused arguments cause warnings.
    /// (function(foo) {
    ///     return 5;
    /// })();
    ///
    /// // Unused recursive functions also cause warnings.
    /// function fact(n) {
    ///     if (n < 2) return 1;
    ///     return n * fact(n - 1);
    /// }
    ///
    /// // When a function definition destructures an array, unused entries from
    /// // the array also cause warnings.
    /// function getY([x, y]) {
    ///     return y;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// /*eslint no-unused-vars: "error"*/
    ///
    /// var x = 10;
    /// alert(x);
    ///
    /// // foo is considered used here
    /// myFunc(function foo() {
    ///     // ...
    /// }.bind(this));
    ///
    /// (function(foo) {
    ///     return foo;
    /// })();
    ///
    /// var myFunc;
    /// myFunc = setTimeout(function() {
    ///     // myFunc is considered used
    ///     myFunc();
    /// }, 50);
    ///
    /// // Only the second argument from the destructured array is used.
    /// function getY([, y]) {
    ///     return y;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for `/* exported variableName */` operation:
    /// ```javascript
    /// /* exported global_var */
    ///
    /// var global_var = 42;
    /// ```
    NoUnusedVars,
    correctness
);

/// Parses a potential pattern into a [`Regex`] that accepts unicode characters.
fn parse_unicode_rule(value: Option<&Value>, name: &str) -> Option<Regex> {
    // fn parse_unicode_rule(config: &serde_json::map::Map<String, &Value>, name: &str) -> Option<Regex>
    value
        .and_then(Value::as_str)
        .map(|pattern| regex::RegexBuilder::new(pattern).unicode(true).build())
        .transpose()
        .map_err(|err| panic!("Invalid '{}' option for no-unused-vars: {}", name, err))
        .unwrap()
}

impl NoUnusedVars {
    fn check_unused_module_declaration<'a>(
        &self,
        symbol_id: SymbolId,
        module: &'a ModuleDeclaration<'a>,
        ctx: &LintContext<'a>,
    ) {
        if module.is_export() {
            // skip exported variables
            return;
        }
        if let ModuleDeclaration::ImportDeclaration(import) = module {
            if import.specifiers.is_empty() {
                // skip imports without specifiers
                return;
            }
            todo!()
        }
    }

    fn is_ignored_var(&self, name: &str) -> bool {
        if let Some(pat) = &self.vars_ignore_pattern { pat.is_match(name) } else { false }
    }

    fn is_ignored_arg(&self, name: &str) -> bool {
        if let Some(pat) = &self.args_ignore_pattern { pat.is_match(name) } else { false }
        // let Some(identifier) = arg.pattern.kind.identifier() else { return };
        // match &self.args_ignore_pattern {
        //     Some(pat) => if pat.is_match(identifier.name.as_str()) {
        //         return
        //     },
        //     None => {}
        // }
    }

    fn is_ignored_array_destructured(&self, name: &str) -> bool {
        if let Some(pat) = &self.destructured_array_ignore_pattern {
            pat.is_match(name)
        } else {
            false
        }
    }

    fn is_ignored_catch_err(&self, name: &str) -> bool {
        if let Some(pat) = &self.caught_errors_ignore_pattern { pat.is_match(name) } else { false }
    }

    // fn is_ignored(&self, node: &AstNode<'_>) -> bool {
    //     match node.kind() {
    //         AstKind::ModuleDeclaration(decl) => match &decl {
    //             ModuleDeclaration::ImportDeclaration(import) => self.is_ignored_var(import..local())
    //         }
    //         // AstKind::VariableDeclarator(decl) => decl.
    //         // AstKind::Function(f)
    //         // AstKind::Class(class)
    //         // AstKind::FormalParameters(params)
    //         // AstKind::FormalParameter(param)
    //     }
    // }
    fn check_unused_binding_pattern<'a>(
        &self,
        symbol_id: SymbolId,
        name: &Atom,
        id: &BindingPattern<'a>,
        ctx: &LintContext<'a>
    ) -> Option<Span> {
        match &id.kind {
            | BindingPatternKind::BindingIdentifier(id) => {
                // debug_assert!(id.name == name, "Expected BindingIdentifier to
                // have name '{name}', but it had name '{}'", id.name);
                // id might not be name if we're in a recursive call from an
                // array or object pattern
                if &id.name != name || self.is_ignored_var(name) {
                    None
                } else {
                    Some(id.span)
                }
            },
            | BindingPatternKind::AssignmentPattern(id) => self.check_unused_binding_pattern(
                symbol_id,
                name,
                &id.left,
                ctx
            ),
            | BindingPatternKind::ArrayPattern(arr) => {
                for el in arr.elements.iter() {
                    let Some(el) = el else { continue };
                    if let Some(id) = el.kind.identifier() {
                        if &id.name != name {
                            continue
                        }
                        // let _id_name = id.name.as_str();
                        if !self.is_ignored_array_destructured(&id.name) {
                            return Some(id.span)
                        }
                    } else {
                        return self.check_unused_binding_pattern(symbol_id, name, el, ctx);
                    }
                }
                None
            },
            | BindingPatternKind::ObjectPattern(obj) => {
                for el in obj.properties.iter() {
                    let maybe_span = self.check_unused_binding_pattern(symbol_id, name, &el.value, ctx);
                    if maybe_span.is_some() {
                        return maybe_span
                    }
                    // match el.key {
                    //     PropertyKey::Identifier(id) => {
                    //         return Some(id.span)
                    //     },
                    //     _ => todo!()
                    // }
                    // let maybe_span = self.check_unused_binding_pattern(symbol_id, name, id, ctx)
                    // if el.name
                }
                None
            }
        }
    }

    fn check_unused_variable_declarator<'a>(
        &self,
        symbol_id: SymbolId,
        decl: &'a VariableDeclarator<'a>,
        ctx: &LintContext<'a>,
    ) {
        let name = ctx.symbols().get_name(symbol_id);

        // Allow `var x` for "vars": "local" b/c var keyword has side effects
        if self.vars == VarsOption::Local && decl.kind.is_var() {
            return;
        }

        // skip unused variable declarations
        if self.is_ignored_var(name) {
            return;
        }

        // allow ignored args
        let Some(span) = self.check_unused_binding_pattern(symbol_id, name, &decl.id, ctx) else { return };

        // ignore exported vars
        let var_decl_node = ctx.semantic().symbol_declaration(symbol_id);
        match ctx.nodes().parent_node(var_decl_node.id()) {
            Some(parent) if Self::is_exported(parent, ctx) => return,
            _ => { /* noop */ }
        }

        ctx.diagnostic(NoUnusedVarsDiagnostic::decl(name.clone(), span));
    }

    fn check_unused_catch_clause<'a>(
        &self,
        symbol_id: SymbolId,
        catch: &'a CatchClause,
        ctx: &LintContext<'a>,
    ) {
        let name = ctx.symbols().get_name(symbol_id);
        if self.caught_errors && !self.is_ignored_catch_err(name.as_str()) {
            match &catch.param {
                Some(error) => {
                    if let Some(id) = error.kind.identifier() {
                        ctx.diagnostic(NoUnusedVarsDiagnostic::decl(id.name.clone(), id.span))
                    }
                }
                None => {
                    debug_assert!(
                        false,
                        "Found unused caught error but CatchClause AST node has no param"
                    );
                }
            }
        }
    }

    fn check_unused_function<'a>(
        &self,
        symbol_id: SymbolId,
        f: &'a Function,
        ctx: &LintContext<'a>,
    ) {
        // skip exported functions
        // if Self::is_exported(f, ctx) {
        //     return
        // }
        // if f.modifiers.contains(ModifierKind::Export) {
        //     return
        // }

        // skip ignored functions
        let Some(name) = f.id.as_ref().map(|binding| &binding.name) else { return };
        if self.is_ignored_var(name.as_str()) {
            return;
        }

        ctx.diagnostic(NoUnusedVarsDiagnostic::decl(name.clone(), f.span))
    }

    fn check_unused_class<'a>(&self, symbol_id: SymbolId, class: &'a Class, ctx: &LintContext<'a>) {
        let Some(name) = class.id.as_ref().map(|binding| &binding.name) else { return };
        if self.is_ignored_var(name.as_str()) {
            return;
        }

        ctx.diagnostic(NoUnusedVarsDiagnostic::decl(name.clone(), class.span))
    }

    fn check_unused_arguments<'a>(
        &self,
        symbol_id: SymbolId,
        scope_id: ScopeId,
        args: &FormalParameters<'a>,
        ctx: &LintContext<'a>,
    ) {
        // let semantic = ctx.semantic().clone();
        match self.args {
            ArgsOption::All => {
                let name = ctx.semantic().symbols().get_name(symbol_id);
                if self.is_ignored_arg(name) {
                    return;
                }
                let arg = args
                    .items
                    .iter()
                    .find(|arg| arg.pattern.kind.identifier().is_some_and(|id| id.name == name));
                match arg {
                    Some(arg) => {
                        ctx.diagnostic(NoUnusedVarsDiagnostic::decl(name.clone(), arg.span))
                    }
                    None => {
                        debug_assert!(
                            false,
                            "Could not find FormalArgument in FormalArguments AST node even though Semantic said we would find it here. This is a bug."
                        );
                    }
                }
                //     args.items.iter().for_each(|arg| self.check_unused_argument(arg, ctx))
            }
            ArgsOption::AfterUsed => {
                let name = ctx.semantic().symbols().get_name(symbol_id);
                if self.is_ignored_arg(name) {
                    return;
                }

                // set to true when a arg defined before the current one is
                // found to be used
                let mut has_prev_used = false;
                for arg in args.items.iter().rev() {
                    if has_prev_used {
                        break;
                    }

                    let Some(binding) = arg.pattern.kind.identifier() else { continue };
                    let Some(arg_symbol_id) = ctx.scopes().get_binding(scope_id, &binding.name)
                    else {
                        continue;
                    };

                    // we've reached the current argument, break
                    if arg_symbol_id == symbol_id {
                        break;
                    }

                    {
                        let refs =
                            ctx.semantic().symbols().get_resolved_reference_ids(arg_symbol_id);
                        let refs_count = refs
                            .iter()
                            .filter(|r| !ctx.semantic().symbols().references[**r].is_write())
                            .count();
                        if refs_count > 0 {
                            has_prev_used = true;
                        }
                    }
                }

                if !has_prev_used {
                    let arg = args.items.iter().find(|arg| {
                        arg.pattern.kind.identifier().is_some_and(|id| id.name == name)
                    });
                    debug_assert!(
                        arg.is_some(),
                        "Expected {name} to be in FormalParameters.items but it wasn't"
                    );
                    let Some(arg) = arg else { return };
                    ctx.diagnostic(NoUnusedVarsDiagnostic::decl(name.clone(), arg.span))
                }
            }
            ArgsOption::None => {
                // noop
            }
        }
    }

    fn check_unused_argument<'a>(
        &self,
        symbol_id: SymbolId,
        arg: &'a FormalParameter,
        ctx: &LintContext<'a>,
    ) {
        let name = ctx.semantic().symbols().get_name(symbol_id);
        if self.is_ignored_arg(name.as_str()) {
            return;
        }
        let Some(identifier) = arg.pattern.kind.identifier() else {
            debug_assert!(
                false,
                "No binding identifier found for FormalParameter with name {}",
                name.as_str()
            );
            return;
        };
        // match &self.args_ignore_pattern {
        //     Some(pat) => if pat.is_match(identifier.name.as_str()) {
        //         return
        //     },
        //     None => {}
        // }

        ctx.diagnostic(NoUnusedVarsDiagnostic(
            identifier.name.clone(),
            DECLARED_STR,
            "".into(),
            arg.span,
        ))
    }

    fn is_exported(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
        let Some(parent_kind) = ctx.nodes().parent_kind(node.id()) else { return false };
        let AstKind::ModuleDeclaration(module) = parent_kind else { return false };
        // return module.is_export()
        match module {
            ModuleDeclaration::ExportAllDeclaration(_)
            | ModuleDeclaration::ExportNamedDeclaration(_)
            | ModuleDeclaration::ExportDefaultDeclaration(_) => true,
            // todo: should we include ts exports?
            _ => false,
        }
    }
}

impl Rule for NoUnusedVars {
    fn from_configuration(value: Value) -> Self {
        let Some(config) = value.get(0) else { return Self::default() };
        match config {
            Value::String(vars) => {
                let vars: VarsOption = vars
                    .try_into()
                    .map_err(|err| format!("Invalid 'vars' option for no-unused-vars: {:}", err))
                    .unwrap();
                Self { vars, ..Default::default() }
            }
            Value::Object(config) => {
                let vars = config
                    .get("vars")
                    .map(|vars| {
                        let vars: VarsOption = vars
                            .try_into()
                            .map_err(|err| {
                                format!("Invalid 'vars' option for no-unused-vars: {:}", err)
                            })
                            .unwrap();
                        vars
                    })
                    .unwrap_or_default();

                let vars_ignore_pattern: Option<Regex> =
                    parse_unicode_rule(config.get("varsIgnorePattern"), "varsIgnorePattern");

                let args: ArgsOption = config
                    .get("args")
                    .map(|args| {
                        let args: ArgsOption = args
                            .try_into()
                            .map_err(|err| {
                                format!("Invalid 'args' option for no-unused-vars: {:}", err)
                            })
                            .unwrap();
                        args
                    })
                    .unwrap_or_default();

                let args_ignore_pattern: Option<Regex> =
                    parse_unicode_rule(config.get("argsIgnorePattern"), "argsIgnorePattern");

                let caught_errors: bool = config
                    .get("caughtErrors")
                    .map(|caught_errors| {
                        match caught_errors {
                            Value::String(s) => match s.as_str() {
                                "all" => true,
                                "none" => false,
                                _ => panic!("Invalid 'caughtErrors' option for no-unused-vars: Expected 'all' or 'none', got {}", s),
                            },
                            _ => panic!("Invalid 'caughtErrors' option for no-unused-vars: Expected a string, got {}", caught_errors),
                            }
                        }).unwrap_or_default();

                let caught_errors_ignore_pattern = parse_unicode_rule(
                    config.get("caughtErrorsIgnorePattern"),
                    "caughtErrorsIgnorePattern",
                );

                let destructured_array_ignore_pattern: Option<Regex> = parse_unicode_rule(
                    config.get("destructuredArrayIgnorePattern"),
                    "destructuredArrayIgnorePattern",
                );

                let ignore_rest_siblings: bool = config
                    .get("ignoreRestSiblings")
                    .map_or(Some(false), Value::as_bool)
                    .unwrap_or(false);

                Self {
                    vars,
                    vars_ignore_pattern,
                    args,
                    args_ignore_pattern,
                    caught_errors,
                    caught_errors_ignore_pattern,
                    destructured_array_ignore_pattern,
                    ignore_rest_siblings,
                }
            }
            Value::Null => Self::default(),
            _ => panic!(
                "Invalid 'vars' option for no-unused-vars: Expected a string or an object, got {config}"
            ),
        }
    }

    fn run_on_symbol(&self, symbol_id: SymbolId, ctx: &LintContext<'_>) {
        let semantic = ctx.semantic();
        let symbols = ctx.symbols();
        let nodes = ctx.nodes();

        // Find all references that count as a usage
        let references: Vec<_> = symbols
            .get_resolved_references(symbol_id)
            .filter(|reference| reference.is_read())
            .collect();

        let name = symbols.get_name(symbol_id);
        let _name_str: &str = name.as_str();

        // Symbol is used, rule doesn't apply
        if references.len() > 0 {
            return;
        }

        // let declaration =
        // ctx.nodes().get_node(ctx.symbols().get_declaration(symbol_id));
        let declaration = semantic.symbol_declaration(symbol_id);
        if Self::is_exported(declaration, ctx) {
            return;
        }
        // let parent_kind = nodes.parent_kind(declaration.id());

        match declaration.kind() {
            AstKind::ModuleDeclaration(decl) => {
                self.check_unused_module_declaration(symbol_id, decl, ctx)
            }
            AstKind::VariableDeclarator(decl) => {
                self.check_unused_variable_declarator(symbol_id, decl, ctx)
            }
            AstKind::Function(f) => self.check_unused_function(symbol_id, f, ctx),
            AstKind::Class(class) => self.check_unused_class(symbol_id, class, ctx),
            AstKind::CatchClause(catch) => self.check_unused_catch_clause(symbol_id, catch, ctx),
            AstKind::FormalParameters(params) => {
                self.check_unused_arguments(symbol_id, declaration.scope_id(), params, ctx)
            }
            AstKind::FormalParameter(param) => self.check_unused_argument(symbol_id, param, ctx),
            s => todo!("handle decl kind {:?}", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};

    use super::*;
    use crate::tester::Tester;

    // test-only trait implementations to make testing easier

    impl PartialEq for ArgsOption {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::AfterUsed, Self::AfterUsed) => true,
                (Self::All, Self::All) => true,
                (Self::None, Self::None) => true,
                _ => false,
            }
        }
    }
    impl From<Value> for NoUnusedVars {
        fn from(value: Value) -> Self {
            Self::from_configuration(value)
        }
    }

    #[test]
    fn test_options_default() {
        let rule = NoUnusedVars::default();
        assert_eq!(rule.vars, VarsOption::All);
        assert!(rule.vars_ignore_pattern.is_none());
        assert_eq!(rule.args, ArgsOption::AfterUsed);
        assert!(rule.args_ignore_pattern.is_none());
        assert!(!rule.caught_errors);
        assert!(rule.caught_errors_ignore_pattern.is_none());
        assert!(rule.destructured_array_ignore_pattern.is_none());
        assert!(!rule.ignore_rest_siblings)
    }

    #[test]
    fn test_options_from_string() {
        let rule: NoUnusedVars = json!(["all"]).into();
        assert_eq!(rule.vars, VarsOption::All);

        let rule: NoUnusedVars = json!(["local"]).into();
        assert_eq!(rule.vars, VarsOption::Local);
    }

    #[test]
    fn test_options_from_object() {
        let rule: NoUnusedVars = json!([
            {
                "vars": "local",
                "varsIgnorePattern": "^_",
                "args": "all",
                "argsIgnorePattern": "^_",
                "caughtErrors": "all",
                "caughtErrorsIgnorePattern": "^_",
                "destructuredArrayIgnorePattern": "^_",
                "ignoreRestSiblings": true
            }
        ])
        .into();

        assert_eq!(rule.vars, VarsOption::Local);
        assert_eq!(rule.vars_ignore_pattern.unwrap().as_str(), "^_");
        assert_eq!(rule.args, ArgsOption::All);
        assert_eq!(rule.args_ignore_pattern.unwrap().as_str(), "^_");
        assert!(rule.caught_errors);
        assert_eq!(rule.caught_errors_ignore_pattern.unwrap().as_str(), "^_");
        assert_eq!(rule.destructured_array_ignore_pattern.unwrap().as_str(), "^_");
        assert!(rule.ignore_rest_siblings);
    }

    #[test]
    fn test_var_simple() {
        let pass = vec![
            "let a = 1; console.log(a);",
            "let a = 1; let b = a + 1; console.log(b);",
            "export var foo = 123;",
        ];
        let fail = vec!["let a", "let a = 1;", "let a = 1; a += 2;"];

        Tester::new_without_config(NoUnusedVars::NAME, pass, fail).test();
    }

    #[test]
    fn test_var_simple_scoped() {
        let pass = vec!["let a = 1; function foo(b) { return a + b }; console.log(foo(1));"];
        let fail =
            vec!["let a = 1; function foo(b) { let a = 1; return a + b }; console.log(foo(1));"];

        Tester::new_without_config(NoUnusedVars::NAME, pass, fail).test();
    }

    #[test]
    fn test_var_ignored() {
        let local = Some(json!(["local"]));
        let ignore_underscore = Some(json!([{ "vars": "all", "varsIgnorePattern": "^_" }]));
        let pass = vec![
            // does const count?
            ("var a", local.clone()),
            ("var _a", ignore_underscore.clone()),
            ("var a = 1; var _b = a", ignore_underscore.clone()),
        ];
        let fail = vec![
            ("var a_", ignore_underscore.clone()),
            // ("let a = 1;", None),
            // ("let a = 1; a += 2;", None)
        ];

        Tester::new(NoUnusedVars::NAME, pass, fail).test();
    }

    #[test]
    fn test_spread_arr_simple() {
        let pass = vec![
            ("let [b] = a; console.log(b);", None),
        ];
        let fail = vec![("let [b] = a", None), ("let [b, c] = a; console.log(b);", None)];

        Tester::new(NoUnusedVars::NAME, pass, fail).test();
    }

    #[test]
    fn test_spread_arr_ignored() {
        let ignore_underscore = Some(json!([{ "destructuredArrayIgnorePattern": "^_" }]));
        let pass = vec![
            ("let [_b] = a;", ignore_underscore),
        ];
        let fail = vec![];

        Tester::new(NoUnusedVars::NAME, pass, fail).test();
    }

    #[test]
    fn test_spread_obj_simple() {
        let pass = vec![
            "let { a } = x; console.log(a)",
            "let { a: { b } } = x; console.log(b)"
        ];
        let fail = vec![
            "let { a } = x;",
            "let { a: { b } } = x;",
            "let { a: { b, c, d } } = x; console.log(b + d)",
        ];
        Tester::new_without_config(NoUnusedVars::NAME, pass, fail).test();
    }

    #[test]
    fn test_spread_compound() {
        let pass = vec![
            ("let { a: [b, c, { e }] } = x; console.log(b + c + e)", None),
            ("function foo({ bar = 1 }) { return bar }; foo()", None)
        ];
        let fail = vec![

            ("let { a: [b, c, d] } = x", None)
        ];
        Tester::new(NoUnusedVars::NAME, pass, fail).test()
    }

    #[test]
    fn test_function_simple() {
        let pass = vec![
            "function foo() { return }; foo()",
            "export function foo() { return }",
            "export default function foo() { return }",
        ];
        let fail = vec!["function foo() { return }"];
        Tester::new_without_config(NoUnusedVars::NAME, pass, fail).test();
    }

    #[test]
    fn test_class_simple() {
        let pass = vec![
            ("class Foo {}; const f = new Foo(); console.log(f)"),
            ("export class Foo {}"),
            ("export default class Foo {}"),
        ];
        let fail = vec![("class Foo {}")];
        Tester::new_without_config(NoUnusedVars::NAME, pass, fail).test()
    }

    #[test]
    fn test_args_simple() {
        let all = Some(json!([{ "args": "all" }]));
        let none = Some(json!([{ "args": "none" }]));
        let after_used = Some(json!([{ "args": "after-used" }]));
        let pass = vec![
            ("function foo(a) { return a }; foo()", all.clone()),
            ("function foo(a) { return }; foo()", none),
            // after used
            ("function foo(a, b) { return b }; foo()", None),
            ("function foo(a, b) { return b }; foo()", after_used),
        ];
        let fail = vec![
            ("function foo(a) { return }; foo()", None),
            ("function foo(a = 1) { return }; foo()", None),
            ("function foo() { return }", None),
            ("function foo(a, b) { return a }; foo()", None),
            ("function foo(a, b) { return b }; foo()", all.clone()),
        ];
        Tester::new(NoUnusedVars::NAME, pass, fail).test();
    }

    #[test]
    fn test_catch_clause_simple() {
        let all = Some(json!([{"caughtErrors": "all"}]));
        let allow_underscore =
            Some(json!([{ "caughtErrors": "all", "caughtErrorsIgnorePattern": "^_" }]));
        let pass = vec![
            ("try {} catch (e) { }", None),
            ("try {} catch (_e) { }", allow_underscore.clone()),
            ("try {} catch (e) { console.error(e); }", all.clone()),
        ];
        let fail = vec![("try {} catch (e) { }", all), ("try {} catch (e) { }", allow_underscore)];
        Tester::new(NoUnusedVars::NAME, pass, fail).test();
    }

    #[test]
    fn test_modules_simple() {
        let pass = vec![
            ("export const foo = 1;", None),
            ("export default { a: true };", None),
            ("export function foo() {}", None),
            ("export default function foo() {}", None),
            ("export class Foo {}", None),
            ("export default class Foo {}", None),
            // todo
            // ("export const Foo = class Foo {}", None),
        ];
        let fail = vec![];
        Tester::new(NoUnusedVars::NAME, pass, fail).test();
    }

    #[test]
    fn test() {
        let pass = vec![
            ("var foo = 5;\n\nlabel: while (true) {\n  console.log(foo);\n  break label;\n}", None),
            ("var foo = 5;\n\nwhile (true) {\n  console.log(foo);\n  break;\n}", None),
            ("for (let prop in box) {\n        box[prop] = parseInt(box[prop]);\n}", None),
            (
                "var box = {a: 2};\n    for (var prop in box) {\n        box[prop] = parseInt(box[prop]);\n}",
                None,
            ),
            // todo
            // ("f({ set foo(a) { return; } });", None),
            ("a; var a;", Some(serde_json::json!(["all"]))),
            ("var a=10; alert(a);", Some(serde_json::json!(["all"]))),
            ("var a=10; (function() { alert(a); })();", Some(serde_json::json!(["all"]))),
            (
                "var a=10; (function() { setTimeout(function() { alert(a); }, 0); })();",
                Some(serde_json::json!(["all"])),
            ),
            ("var a=10; d[a] = 0;", Some(serde_json::json!(["all"]))),
            ("(function() { var a=10; return a; })();", Some(serde_json::json!(["all"]))),
            ("(function g() {})()", Some(serde_json::json!(["all"]))),
            ("function f(a) {alert(a);}; f();", Some(serde_json::json!(["all"]))),
            (
                "var c = 0; function f(a){ var b = a; return b; }; f(c);",
                Some(serde_json::json!(["all"])),
            ),
            // todo
            ("function a(x, y){ return y; }; a();", Some(serde_json::json!(["all"]))),
            (
                "var arr1 = [1, 2]; var arr2 = [3, 4]; for (var i in arr1) { arr1[i] = 5; } for (var i in arr2) { arr2[i] = 10; }",
                Some(serde_json::json!(["all"])),
            ),
            // todo
            ("var a=10;", Some(serde_json::json!(["local"]))),
            ("var min = \"min\"; Math[min];", Some(serde_json::json!(["all"]))),
            ("Foo.bar = function(baz) { return baz; };", Some(serde_json::json!(["all"]))),
            ("myFunc(function foo() {}.bind(this))", None),
            ("myFunc(function foo(){}.toString())", None),
            (
                "function foo(first, second) {\ndoStuff(function() {\nconsole.log(second);});}; foo()",
                None,
            ),
            ("(function() { var doSomething = function doSomething() {}; doSomething() }())", None),
            ("try {} catch(e) {}", None),
            ("/* global a */ a;", None),
            (
                "var a=10; (function() { alert(a); })();",
                Some(serde_json::json!([{ "vars": "all" }])),
            ),
            (
                "function g(bar, baz) { return baz; }; g();",
                Some(serde_json::json!([{ "vars": "all" }])),
            ),
            (
                "function g(bar, baz) { return baz; }; g();",
                Some(serde_json::json!([{ "vars": "all", "args": "after-used" }])),
            ),
            (
                "function g(bar, baz) { return bar; }; g();",
                Some(serde_json::json!([{ "vars": "all", "args": "none" }])),
            ),
            (
                "function g(bar, baz) { return 2; }; g();",
                Some(serde_json::json!([{ "vars": "all", "args": "none" }])),
            ),
            (
                "function g(bar, baz) { return bar + baz; }; g();",
                Some(serde_json::json!([{ "vars": "local", "args": "all" }])),
            ),
            (
                "var g = function(bar, baz) { return 2; }; g();",
                Some(serde_json::json!([{ "vars": "all", "args": "none" }])),
            ),
            ("(function z() { z(); })();", None),
            (" ", None),
            ("var who = \"Paul\";\nmodule.exports = `Hello ${who}!`;", None),
            ("export var foo = 123;", None),
            ("export function foo () {}", None),
            // FIXME
            // ("let toUpper = (partial) => partial.toUpperCase; export {toUpper}", None),
            ("export class foo {}", None),
            ("class Foo{}; var x = new Foo(); x.foo()", None),
            (
                "const foo = \"hello!\";function bar(foobar = foo) {  foobar.replace(/!$/, \" world!\");}\nbar();",
                None,
            ),
            ("function Foo(){}; var x = new Foo(); x.foo()", None),
            ("function foo() {var foo = 1; return foo}; foo();", None),
            ("function foo(foo) {return foo}; foo(1);", None),
            ("function foo() {function foo() {return 1;}; return foo()}; foo();", None),
            ("function foo() {var foo = 1; return foo}; foo();", None),
            ("function foo(foo) {return foo}; foo(1);", None),
            ("function foo() {function foo() {return 1;}; return foo()}; foo();", None),
            ("const x = 1; const [y = x] = []; foo(y);", None),
            ("const x = 1; const {y = x} = {}; foo(y);", None),
            ("const x = 1; const {z: [y = x]} = {}; foo(y);", None),
            ("const x = []; const {z: [y] = x} = {}; foo(y);", None),
            ("const x = 1; let y; [y = x] = []; foo(y);", None),
            ("const x = 1; let y; ({z: [y = x]} = {}); foo(y);", None),
            // ("const x = []; let y; ({z: [y] = x} = {}); foo(y);", None),
            // ("const x = 1; function foo(y = x) { bar(y); } foo();", None),
            // ("const x = 1; function foo({y = x} = {}) { bar(y); } foo();", None),
            // ("const x = 1; function foo(y = function(z = x) { bar(z); }) { y(); } foo();", None),
            // ("const x = 1; function foo(y = function() { bar(x); }) { y(); } foo();", None),
            // ("var x = 1; var [y = x] = []; foo(y);", None),
            // ("var x = 1; var {y = x} = {}; foo(y);", None),
            // ("var x = 1; var {z: [y = x]} = {}; foo(y);", None),
            // ("var x = []; var {z: [y] = x} = {}; foo(y);", None),
            // ("var x = 1, y; [y = x] = []; foo(y);", None),
            // ("var x = 1, y; ({z: [y = x]} = {}); foo(y);", None),
            // ("var x = [], y; ({z: [y] = x} = {}); foo(y);", None),
            ("var x = 1; function foo(y = x) { bar(y); } foo();", None),
            // ("var x = 1; function foo({y = x} = {}) { bar(y); } foo();", None),
            ("var x = 1; function foo(y = function(z = x) { bar(z); }) { y(); } foo();", None),
            ("var x = 1; function foo(y = function() { bar(x); }) { y(); } foo();", None),
            // ("/*exported toaster*/ var toaster = 'great'", None),
            // ("/*exported toaster, poster*/ var toaster = 1; poster = 0;", None),
            // ("/*exported x*/ var { x } = y", None),
            // ("/*exported x, y*/  var { x, y } = z", None),
            // ("/*eslint use-every-a:1*/ var a;", None),
            // ("/*eslint use-every-a:1*/ !function(a) { return 1; }", None),
            // ("/*eslint use-every-a:1*/ !function() { var a; return 1 }", None),
            ("var _a;", Some(serde_json::json!([{ "vars": "all", "varsIgnorePattern": "^_" }]))),
            // todo
            (
                "var a; function foo() { var _b; } foo();",
                Some(serde_json::json!([{ "vars": "local", "varsIgnorePattern": "^_" }])),
            ),
            (
                "function foo(_a) { } foo();",
                Some(serde_json::json!([{ "args": "all", "argsIgnorePattern": "^_" }])),
            ),
            (
                "function foo(a, _b) { return a; } foo();",
                Some(serde_json::json!([{ "args": "after-used", "argsIgnorePattern": "^_" }])),
            ),
            (
                "var [ firstItemIgnored, secondItem ] = items;\nconsole.log(secondItem);",
                Some(serde_json::json!([{ "vars": "all", "varsIgnorePattern": "[iI]gnored" }])),
            ),
            // (
            //     "const [ a, _b, c ] = items;\nconsole.log(a+c);",
            //     Some(serde_json::json!([{ "destructuredArrayIgnorePattern": "^_" }])),
            // ),
            // (
            //     "const [ [a, _b, c] ] = items;\nconsole.log(a+c);",
            //     Some(serde_json::json!([{ "destructuredArrayIgnorePattern": "^_" }])),
            // ),
            // (
            //     "const { x: [_a, foo] } = bar;\nconsole.log(foo);",
            //     Some(serde_json::json!([{ "destructuredArrayIgnorePattern": "^_" }])),
            // ),
            // (
            //     "function baz([_b, foo]) { foo; };\nbaz()",
            //     Some(serde_json::json!([{ "destructuredArrayIgnorePattern": "^_" }])),
            // ),
            // (
            //     "function baz({x: [_b, foo]}) {foo};\nbaz()",
            //     Some(serde_json::json!([{ "destructuredArrayIgnorePattern": "^_" }])),
            // ),
            // (
            //     "function baz([{x: [_b, foo]}]) {foo};\nbaz()",
            //     Some(serde_json::json!([{ "destructuredArrayIgnorePattern": "^_" }])),
            // ),
            // // ,
            // // ,
            // // ,
            // FIXME(don): failing, this looks like a semantic analysis bug
            // ("(function(obj) { var name; for ( name in obj ) return; })({});", None),
            // ("(function(obj) { var name; for ( name in obj ) { return; } })({});", None),
            // ("(function(obj) { for ( var name in obj ) { return true } })({})", None),
            // ("(function(obj) { for ( var name in obj ) return true })({})", None),
            // ("(function(obj) { let name; for ( name in obj ) return; })({});", None),
            // ("(function(obj) { let name; for ( name in obj ) { return; } })({});", None),
            // ("(function(obj) { for ( let name in obj ) { return true } })({})", None),
            // ("(function(obj) { for ( let name in obj ) return true })({})", None),
            // ("(function(obj) { for ( const name in obj ) { return true } })({})", None),
            // ("(function(obj) { for ( const name in obj ) return true })({})", None),
            // ("(function(iter) { let name; for ( name of iter ) return; })({});", None),
            // ("(function(iter) { let name; for ( name of iter ) { return; } })({});", None),
            // ("(function(iter) { for ( let name of iter ) { return true } })({})", None),
            // ("(function(iter) { for ( let name of iter ) return true })({})", None),
            // ("(function(iter) { for ( const name of iter ) { return true } })({})", None),
            // ("(function(iter) { for ( const name of iter ) return true })({})", None),
            // FIXME
            // ("let x = 0; foo = (0, x++);", None),
            // ("let x = 0; foo = (0, x += 1);", None),
            // ("let x = 0; foo = (0, x = x + 1);", None),
            (
                "try{}catch(err){console.error(err);}",
                Some(serde_json::json!([{ "caughtErrors": "all" }])),
            ),
            ("try{}catch(err){}", Some(serde_json::json!([{ "caughtErrors": "none" }]))),
            (
                "try{}catch(ignoreErr){}",
                Some(
                    serde_json::json!([{ "caughtErrors": "all", "caughtErrorsIgnorePattern": "^ignore" }]),
                ),
            ),
            ("try{}catch(err){}", Some(serde_json::json!([{ "vars": "all", "args": "all" }]))),
            // (
            //     "const data = { type: 'coords', x: 1, y: 2 };\nconst { type, ...coords } = data;\n console.log(coords);",
            //     Some(serde_json::json!([{ "ignoreRestSiblings": true }])),
            // ),
            // ("var a = 0, b; b = a = a + 1; foo(b);", None),
            // ("var a = 0, b; b = a += a + 1; foo(b);", None),
            // ("var a = 0, b; b = a++; foo(b);", None),
            // ("function foo(a) { var b = a = a + 1; bar(b) } foo();", None),
            // ("function foo(a) { var b = a += a + 1; bar(b) } foo();", None),
            // ("function foo(a) { var b = a++; bar(b) } foo();", None),
            // (
            //     "var unregisterFooWatcher;\n// ...\nunregisterFooWatcher = $scope.$watch( \"foo\", function() {\n    // ...some code..\n    unregisterFooWatcher();\n});\n",
            //     None,
            // ),
            // (
            //     "var ref;\nref = setInterval(\n    function(){\n        clearInterval(ref);\n    }, 10);\n",
            //     None,
            // ),
            // (
            //     "var _timer;\nfunction f() {\n    _timer = setTimeout(function () {}, _timer ? 100 : 0);\n}\nf();\n",
            //     None,
            // ),
            // (
            //     "function foo(cb) { cb = function() { function something(a) { cb(1 + a); } register(something); }(); } foo();",
            //     None,
            // ),
            // ("function* foo(cb) { cb = yield function(a) { cb(1 + a); }; } foo();", None),
            // ("function foo(cb) { cb = tag`hello${function(a) { cb(1 + a); }}`; } foo();", None),
            // ("function foo(cb) { var b; cb = b = function(a) { cb(1 + a); }; b(); } foo();", None),
            // (
            //     "function someFunction() {\n    var a = 0, i;\n    for (i = 0; i < 2; i++) {\n        a = myFunction(a);\n    }\n}\nsomeFunction();\n",
            //     None,
            // ),
            // (
            //     "(function(a, b, {c, d}) { d })",
            //     Some(serde_json::json!([{ "argsIgnorePattern": "c" }])),
            // ),
            // (
            //     "(function(a, b, {c, d}) { c })",
            //     Some(serde_json::json!([{ "argsIgnorePattern": "d" }])),
            // ),
            // ("(function(a, b, c) { c })", Some(serde_json::json!([{ "argsIgnorePattern": "c" }]))),
            // (
            //     "(function(a, b, {c, d}) { c })",
            //     Some(serde_json::json!([{ "argsIgnorePattern": "[cd]" }])),
            // ),
            // ("(class { set foo(UNUSED) {} })", None),
            // ("class Foo { set bar(UNUSED) {} } console.log(Foo)", None),
            // (
            //     "(({a, ...rest}) => rest)",
            //     Some(serde_json::json!([{ "args": "all", "ignoreRestSiblings": true }])),
            // ),
            // (
            //     "let foo, rest;\n({ foo, ...rest } = something);\nconsole.log(rest);",
            //     Some(serde_json::json!([{ "ignoreRestSiblings": true }])),
            // ),
            // ("/*eslint use-every-a:1*/ !function(b, a) { return 1 }", None),
            // ("var a = function () { a(); }; a();", None),
            // ("var a = function(){ return function () { a(); } }; a();", None),
            ("const a = () => { a(); }; a();", None),
            ("const a = () => () => { a(); }; a();", None),
            ("export * as ns from \"source\"", None),
            // ("import.meta", None),
            // FIXME
            // ("var a; a ||= 1;", None),
            // ("var a; a &&= 1;", None),
            // ("var a; a ??= 1;", None),
        ];

        let fail = vec![
            // ("function foox() { return foox(); }", None),
            // ("(function() { function foox() { if (true) { return foox(); } } }())", None),
            ("var a=10", None),
            ("function f() { var a = 1; return function(){ f(a *= 2); }; }", None),
            ("function f() { var a = 1; return function(){ f(++a); }; }", None),
            // ("/*global a */", None),
            // (
            //     "function foo(first, second) {\ndoStuff(function() {\nconsole.log(second);});};",
            //     None,
            // ),
            // ("var a=10;", Some(serde_json::json!(["all"]))),
            // ("var a=10; a=20;", Some(serde_json::json!(["all"]))),
            // (
            //     "var a=10; (function() { var a = 1; alert(a); })();",
            //     Some(serde_json::json!(["all"])),
            // ),
            // ("var a=10, b=0, c=null; alert(a+b)", Some(serde_json::json!(["all"]))),
            // (
            //     "var a=10, b=0, c=null; setTimeout(function() { var b=2; alert(a+b+c); }, 0);",
            //     Some(serde_json::json!(["all"])),
            // ),
            // (
            //     "var a=10, b=0, c=null; setTimeout(function() { var b=2; var c=2; alert(a+b+c); }, 0);",
            //     Some(serde_json::json!(["all"])),
            // ),
            // (
            //     "function f(){var a=[];return a.map(function(){});}",
            //     Some(serde_json::json!(["all"])),
            // ),
            // (
            //     "function f(){var a=[];return a.map(function g(){});}",
            //     Some(serde_json::json!(["all"])),
            // ),
            // (
            //     "function foo() {function foo(x) {\nreturn x; }; return function() {return foo; }; }",
            //     None,
            // ),
            // (
            //     "function f(){var x;function a(){x=42;}function b(){alert(x);}}",
            //     Some(serde_json::json!(["all"])),
            // ),
            // ("function f(a) {}; f();", Some(serde_json::json!(["all"]))),
            // ("function a(x, y, z){ return y; }; a();", Some(serde_json::json!(["all"]))),
            // ("var min = Math.min", Some(serde_json::json!(["all"]))),
            // ("var min = {min: 1}", Some(serde_json::json!(["all"]))),
            // ("Foo.bar = function(baz) { return 1; };", Some(serde_json::json!(["all"]))),
            // ("var min = {min: 1}", Some(serde_json::json!([{ "vars": "all" }]))),
            // (
            //     "function gg(baz, bar) { return baz; }; gg();",
            //     Some(serde_json::json!([{ "vars": "all" }])),
            // ),
            // (
            //     "(function(foo, baz, bar) { return baz; })();",
            //     Some(serde_json::json!([{ "vars": "all", "args": "after-used" }])),
            // ),
            // (
            //     "(function(foo, baz, bar) { return baz; })();",
            //     Some(serde_json::json!([{ "vars": "all", "args": "all" }])),
            // ),
            // (
            //     "(function z(foo) { var bar = 33; })();",
            //     Some(serde_json::json!([{ "vars": "all", "args": "all" }])),
            // ),
            // ("(function z(foo) { z(); })();", Some(serde_json::json!([{}]))),
            // (
            //     "function f() { var a = 1; return function(){ f(a = 2); }; }",
            //     Some(serde_json::json!([{}])),
            // ),
            // ("import x from \"y\";", None),
            // ("export function fn2({ x, y }) {\n console.log(x); \n};", None),
            // ("export function fn2( x, y ) {\n console.log(x); \n};", None),
            // ("/*exported max*/ var max = 1, min = {min: 1}", None),
            // ("/*exported x*/ var { x, y } = z", None),
            // (
            //     "var _a; var b;",
            //     Some(serde_json::json!([{ "vars": "all", "varsIgnorePattern": "^_" }])),
            // ),
            // (
            //     "var a; function foo() { var _b; var c_; } foo();",
            //     Some(serde_json::json!([{ "vars": "local", "varsIgnorePattern": "^_" }])),
            // ),
            // (
            //     "function foo(a, _b) { } foo();",
            //     Some(serde_json::json!([{ "args": "all", "argsIgnorePattern": "^_" }])),
            // ),
            // (
            //     "function foo(a, _b, c) { return a; } foo();",
            //     Some(serde_json::json!([{ "args": "after-used", "argsIgnorePattern": "^_" }])),
            // ),
            // (
            //     "function foo(_a) { } foo();",
            //     Some(serde_json::json!([{ "args": "all", "argsIgnorePattern": "[iI]gnored" }])),
            // ),
            // (
            //     "var [ firstItemIgnored, secondItem ] = items;",
            //     Some(serde_json::json!([{ "vars": "all", "varsIgnorePattern": "[iI]gnored" }])),
            // ),
            // /*
            //       {
            //        code: "const [ a, _b, c ] = items;\nconsole.log(a+c);",
            //        options: [{ destructuredArrayIgnorePattern: "^_" }],
            //        parserOptions: { ecmaVersion: 6 }
            //    },
            //    {
            //        code: "const [ [a, _b, c] ] = items;\nconsole.log(a+c);",
            //        options: [{ destructuredArrayIgnorePattern: "^_" }],
            //        parserOptions: { ecmaVersion: 6 }
            //    },
            //    {
            //        code: "const { x: [_a, foo] } = bar;\nconsole.log(foo);",
            //        options: [{ destructuredArrayIgnorePattern: "^_" }],
            //        parserOptions: { ecmaVersion: 6 }
            //    },
            //    {
            //        code: "function baz([_b, foo]) { foo; };\nbaz()",
            //        options: [{ destructuredArrayIgnorePattern: "^_" }],
            //        parserOptions: { ecmaVersion: 6 }
            //    },
            //    {
            //        code: "function baz({x: [_b, foo]}) {foo};\nbaz()",
            //        options: [{ destructuredArrayIgnorePattern: "^_" }],
            //        parserOptions: { ecmaVersion: 6 }
            //    },
            //    {
            //        code: "function baz([{x: [_b, foo]}]) {foo};\nbaz()",
            //        options: [{ destructuredArrayIgnorePattern: "^_" }],
            //        parserOptions: { ecmaVersion: 6 }
            //    },
            //    {
            //        code: `
            //        let _a, b;
            //        foo.forEach(item => {
            //            [_a, b] = item;
            //            doSomething(b);
            //        });
            //        `,
            //        options: [{ destructuredArrayIgnorePattern: "^_" }],
            //        parserOptions: { ecmaVersion: 6 }
            //    },
            //    {
            //        code: `
            //        // doesn't report _x
            //        let _x, y;
            //        _x = 1;
            //        [_x, y] = foo;
            //        y;

            //        // doesn't report _a
            //        let _a, b;
            //        [_a, b] = foo;
            //        _a = 1;
            //        b;
            //        `,
            //        options: [{ destructuredArrayIgnorePattern: "^_" }],
            //        parserOptions: { ecmaVersion: 2018 }
            //    },
            //    {
            //        code: `
            //        // doesn't report _x
            //        let _x, y;
            //        _x = 1;
            //        [_x, y] = foo;
            //        y;

            //        // doesn't report _a
            //        let _a, b;
            //        _a = 1;
            //        ({_a, ...b } = foo);
            //        b;
            //        `,
            //        options: [{ destructuredArrayIgnorePattern: "^_", ignoreRestSiblings: true }],
            //        parserOptions: { ecmaVersion: 2018 }
            //    },
            // */
            // ("(function(obj) { var name; for ( name in obj ) { i(); return; } })({});", None),
            // ("(function(obj) { var name; for ( name in obj ) { } })({});", None),
            // ("(function(obj) { for ( var name in obj ) { } })({});", None),
            // ("(function(iter) { var name; for ( name of iter ) { i(); return; } })({});", None),
            // ("(function(iter) { var name; for ( name of iter ) { } })({});", None),
            // ("(function(iter) { for ( var name of iter ) { } })({});", None),
            // ("\n/* global foobar, foo, bar */\nfoobar;", None),
            // ("\n/* global foobar,\n   foo,\n   bar\n */\nfoobar;", None),
            // (
            //     "const data = { type: 'coords', x: 1, y: 2 };\nconst { type, ...coords } = data;\n console.log(coords);",
            //     None,
            // ),
            // (
            //     "const data = { type: 'coords', x: 2, y: 2 };\nconst { type, ...coords } = data;\n console.log(type)",
            //     Some(serde_json::json!([{ "ignoreRestSiblings": true }])),
            // ),
            // (
            //     "let type, coords;\n({ type, ...coords } = data);\n console.log(type)",
            //     Some(serde_json::json!([{ "ignoreRestSiblings": true }])),
            // ),
            // (
            //     "const data = { type: 'coords', x: 3, y: 2 };\nconst { type, ...coords } = data;\n console.log(type)",
            //     None,
            // ),
            // (
            //     "const data = { vars: ['x','y'], x: 1, y: 2 };\nconst { vars: [x], ...coords } = data;\n console.log(coords)",
            //     None,
            // ),
            // (
            //     "const data = { defaults: { x: 0 }, x: 1, y: 2 };\nconst { defaults: { x }, ...coords } = data;\n console.log(coords)",
            //     None,
            // ),
            // (
            //     "(({a, ...rest}) => {})",
            //     Some(serde_json::json!([{ "args": "all", "ignoreRestSiblings": true }])),
            // ),
            // ("/* global a$fooz,$foo */\na$fooz;", None),
            // ("/* globals a$fooz, $ */\na$fooz;", None),
            // ("/*globals $foo*/", None),
            // ("/* global global*/", None),
            // ("/*global foo:true*/", None),
            // ("/*global 変数, 数*/\n変数;", None),
            // ("/*global 𠮷𩸽, 𠮷*/\n\\u{20BB7}\\u{29E3D};", None),
            // ("export default function(a) {}", None),
            // ("export default function(a, b) { console.log(a); }", None),
            // ("export default (function(a) {});", None),
            // ("export default (function(a, b) { console.log(a); });", None),
            // ("export default (a) => {};", None),
            // ("export default (a, b) => { console.log(a); };", None),
            // ("try{}catch(err){};", Some(serde_json::json!([{ "caughtErrors": "all" }]))),
            // (
            //     "try{}catch(err){};",
            //     Some(
            //         serde_json::json!([{ "caughtErrors": "all", "caughtErrorsIgnorePattern": "^ignore" }]),
            //     ),
            // ),
            // (
            //     "try{}catch(ignoreErr){}try{}catch(err){};",
            //     Some(
            //         serde_json::json!([{ "caughtErrors": "all", "caughtErrorsIgnorePattern": "^ignore" }]),
            //     ),
            // ),
            // (
            //     "try{}catch(error){}try{}catch(err){};",
            //     Some(
            //         serde_json::json!([{ "caughtErrors": "all", "caughtErrorsIgnorePattern": "^ignore" }]),
            //     ),
            // ),
            // (
            //     "try{}catch(err){};",
            //     Some(serde_json::json!([{ "vars": "all", "args": "all", "caughtErrors": "all" }])),
            // ),
            // (
            //     "try{}catch(err){};",
            //     Some(serde_json::json!([
            //         {
            //             "vars": "all",
            //             "args": "all",
            //             "caughtErrors": "all",
            //             "argsIgnorePattern": "^er"
            //         }
            //     ])),
            // ),
            // ("var a = 0; a = a + 1;", None),
            // ("var a = 0; a = a + a;", None),
            // ("var a = 0; a += a + 1;", None),
            // ("var a = 0; a++;", None),
            // ("function foo(a) { a = a + 1 } foo();", None),
            // ("function foo(a) { a += a + 1 } foo();", None),
            // ("function foo(a) { a++ } foo();", None),
            // ("var a = 3; a = a * 5 + 6;", None),
            // ("var a = 2, b = 4; a = a * 2 + b;", None),
            // ("function foo(cb) { cb = function(a) { cb(1 + a); }; bar(not_cb); } foo();", None),
            // ("function foo(cb) { cb = function(a) { return cb(1 + a); }(); } foo();", None),
            // ("function foo(cb) { cb = (function(a) { cb(1 + a); }, cb); } foo();", None),
            // ("function foo(cb) { cb = (0, function(a) { cb(1 + a); }); } foo();", None),
            // /*
            // // https://github.com/eslint/eslint/issues/6646
            // {
            //     code: [
            //         "while (a) {",
            //         "    function foo(b) {",
            //         "        b = b + 1;",
            //         "    }",
            //         "    foo()",
            //         "}"
            //     ].join("\n"),
            //     errors: [assignedError("b")]
            // },
            //  */
            // ("(function(a, b, c) {})", Some(serde_json::json!([{ "argsIgnorePattern": "c" }]))),
            // (
            //     "(function(a, b, {c, d}) {})",
            //     Some(serde_json::json!([{ "argsIgnorePattern": "[cd]" }])),
            // ),
            // (
            //     "(function(a, b, {c, d}) {})",
            //     Some(serde_json::json!([{ "argsIgnorePattern": "c" }])),
            // ),
            // (
            //     "(function(a, b, {c, d}) {})",
            //     Some(serde_json::json!([{ "argsIgnorePattern": "d" }])),
            // ),
            // ("/*global\rfoo*/", None),
            // ("(function ({ a }, b ) { return b; })();", None),
            // ("(function ({ a }, { b, c } ) { return b; })();", None),
            // /*
            //   // https://github.com/eslint/eslint/issues/6646
            // {
            //     code: [
            //         "while (a) {",
            //         "    function foo(b) {",
            //         "        b = b + 1;",
            //         "    }",
            //         "    foo()",
            //         "}"
            //     ].join("\n"),
            //     errors: [assignedError("b")]
            // },

            // // https://github.com/eslint/eslint/issues/7124
            // {
            //     code: "(function(a, b, c) {})",
            //     options: [{ argsIgnorePattern: "c" }],
            //     errors: [
            //         definedError("a", ". Allowed unused args must match /c/u"),
            //         definedError("b", ". Allowed unused args must match /c/u")
            //     ]
            // },
            //  */
            ("let x = 0; x++, 0;", None),
            ("let x = 0; 0, x++;", None),
            ("let x = 0; 0, (1, x++);", None),
            ("let x = 0; foo = (x++, 0);", None),
            ("let x = 0; foo = ((0, x++), 0);", None),
            ("let x = 0; x += 1, 0;", None),
            ("let x = 0; 0, x += 1;", None),
            ("let x = 0; 0, (1, x += 1);", None),
            ("let x = 0; foo = (x += 1, 0);", None),
            ("let x = 0; foo = ((0, x += 1), 0);", None),
            // /*
            //        {
            //        code: `let z = 0;
            //        z = z + 1, z = 2;
            //        `,
            //        parserOptions: { ecmaVersion: 2020 },
            //        errors: [{ ...assignedError("z"), line: 2, column: 24 }]
            //    },
            //    {
            //        code: `let z = 0;
            //        z = z+1, z = 2;
            //        z = 3;`,
            //        parserOptions: { ecmaVersion: 2020 },
            //        errors: [{ ...assignedError("z"), line: 3, column: 13 }]
            //    },
            //    {
            //        code: `let z = 0;
            //        z = z+1, z = 2;
            //        z = z+3;
            //        `,
            //        parserOptions: { ecmaVersion: 2020 },
            //        errors: [{ ...assignedError("z"), line: 3, column: 13 }]
            //    },
            // */
            // ("let x = 0; 0, x = x+1;", None),
            // ("let x = 0; x = x+1, 0;", None),
            // ("let x = 0; foo = ((0, x = x + 1), 0);", None),
            // ("let x = 0; foo = (x = x+1, 0);", None),
            // ("let x = 0; 0, (1, x=x+1);", None),
            // ("(function ({ a, b }, { c } ) { return b; })();", None),
            // ("(function ([ a ], b ) { return b; })();", None),
            // ("(function ([ a ], [ b, c ] ) { return b; })();", None),
            // ("(function ([ a, b ], [ c ] ) { return b; })();", None),
            // (
            //     "(function(_a) {})();",
            //     Some(serde_json::json!([{ "args": "all", "varsIgnorePattern": "^_" }])),
            // ),
            // (
            //     "(function(_a) {})();",
            //     Some(serde_json::json!([{ "args": "all", "caughtErrorsIgnorePattern": "^_" }])),
            // ),
            // ("var a = function() { a(); };", None),
            // ("var a = function(){ return function() { a(); } };", None),
            // ("const a = () => { a(); };", None),
            // ("const a = () => () => { a(); };", None),
            // /*
            //         {
            //         code: `let myArray = [1,2,3,4].filter((x) => x == 0);
            // myArray = myArray.filter((x) => x == 1);`,
            //         parserOptions: { ecmaVersion: 2015 },
            //         errors: [{ ...assignedError("myArray"), line: 2, column: 5 }]
            //     },
            //  */
            // ("const a = 1; a += 1;", None),
            // ("var a = function() { a(); };", None),
            // ("var a = function(){ return function() { a(); } };", None),
            // ("const a = () => { a(); };", None),
            // ("const a = () => () => { a(); };", None),
            // ("let x = [];\nx = x.concat(x);", None),
            // /*
            //             {

            //             code: `let a = 'a';
            //             a = 10;
            //             function foo(){
            //                 a = 11;
            //                 a = () => {
            //                     a = 13
            //                 }
            //             }`,
            //             parserOptions: { ecmaVersion: 2020 },
            //             errors: [{ ...assignedError("a"), line: 2, column: 13 }, { ...definedError("foo"), line: 3, column: 22 }]
            //         },
            //         {
            //             code: `let foo;
            //             init();
            //             foo = foo + 2;
            //             function init() {
            //                 foo = 1;
            //             }`,
            //             parserOptions: { ecmaVersion: 2020 },
            //             errors: [{ ...assignedError("foo"), line: 3, column: 13 }]
            //         },
            //         {
            //             code: `function foo(n) {
            //                 if (n < 2) return 1;
            //                 return n * foo(n - 1);
            //             }`,
            //             parserOptions: { ecmaVersion: 2020 },
            //             errors: [{ ...definedError("foo"), line: 1, column: 10 }]
            //         },
            //         {
            //             code: `let c = 'c'
            // c = 10
            // function foo1() {
            //   c = 11
            //   c = () => {
            //     c = 13
            //   }
            // }

            // c = foo1`,
            //             parserOptions: { ecmaVersion: 2020 },
            //             errors: [{ ...assignedError("c"), line: 10, column: 1 }]
            //         }
            //      */
        ];

        Tester::new(NoUnusedVars::NAME, pass, fail).test_and_snapshot();
    }
}
