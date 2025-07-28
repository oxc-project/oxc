use oxc_allocator::TakeIn;
use oxc_ast::ast::*;

use crate::{CompressOptionsUnused, ctx::Ctx};

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    pub fn should_remove_unused_declarator(
        decl: &VariableDeclarator<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) -> bool {
        if ctx.state.options.unused == CompressOptionsUnused::Keep {
            return false;
        }
        if let BindingPatternKind::BindingIdentifier(ident) = &decl.id.kind {
            if Self::keep_top_level_var_in_script_mode(ctx) {
                return false;
            }
            // It is unsafe to remove if direct eval is involved.
            if ctx.scoping().root_scope_flags().contains_direct_eval() {
                return false;
            }
            if let Some(symbol_id) = ident.symbol_id.get() {
                return ctx.scoping().symbol_is_unused(symbol_id);
            }
        }
        false
    }

    pub fn remove_unused_function_declaration(
        f: &Function<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) -> Option<Statement<'a>> {
        if ctx.state.options.unused == CompressOptionsUnused::Keep {
            return None;
        }
        if Self::keep_top_level_var_in_script_mode(ctx) {
            return None;
        }
        let id = f.id.as_ref()?;
        let symbol_id = id.symbol_id.get()?;
        if !ctx.scoping().symbol_is_unused(symbol_id) {
            return None;
        }
        Some(ctx.ast.statement_empty(f.span))
    }

    pub fn remove_unused_class_declaration(
        c: &Class<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) -> Option<Statement<'a>> {
        if ctx.state.options.unused == CompressOptionsUnused::Keep {
            return None;
        }
        if Self::keep_top_level_var_in_script_mode(ctx) {
            return None;
        }
        let id = c.id.as_ref()?;
        let symbol_id = id.symbol_id.get()?;
        if !ctx.scoping().symbol_is_unused(symbol_id) {
            return None;
        }
        // TODO: keep side effectful expressions and remove the class.
        if c.super_class.is_some() {
            return None;
        }
        for e in &c.body.body {
            match e {
                ClassElement::StaticBlock(e) => {
                    if !e.body.is_empty() {
                        return None;
                    }
                }
                ClassElement::MethodDefinition(d) => {
                    if d.computed {
                        return None;
                    }
                }
                ClassElement::PropertyDefinition(_)
                | ClassElement::AccessorProperty(_)
                | ClassElement::TSIndexSignature(_) => {
                    return None;
                }
            }
        }
        Some(ctx.ast.statement_empty(c.span))
    }

    pub fn remove_unused_assignment_expression(
        &self,
        e: &mut Expression<'a>,
        ctx: &mut Ctx<'a, '_>,
    ) -> bool {
        let Expression::AssignmentExpression(assign_expr) = e else { return false };
        if matches!(
            ctx.state.options.unused,
            CompressOptionsUnused::Keep | CompressOptionsUnused::KeepAssign
        ) {
            return false;
        }
        let Some(SimpleAssignmentTarget::AssignmentTargetIdentifier(ident)) =
            assign_expr.left.as_simple_assignment_target()
        else {
            return false;
        };
        if Self::keep_top_level_var_in_script_mode(ctx) {
            return false;
        }
        let Some(reference_id) = ident.reference_id.get() else { return false };
        let Some(symbol_id) = ctx.scoping().get_reference(reference_id).symbol_id() else {
            return false;
        };
        // Keep error for assigning to `const foo = 1; foo = 2`.
        if ctx.scoping().symbol_flags(symbol_id).is_const_variable() {
            return false;
        }
        let Some(symbol_value) = ctx.state.symbol_values.get_symbol_value(symbol_id) else {
            return false;
        };
        // Cannot remove assignment to live bindings: `export let foo; foo = 1;`.
        if symbol_value.exported {
            return false;
        }
        if symbol_value.read_references_count > 0 {
            return false;
        }
        if symbol_value.for_statement_init {
            return false;
        }
        *e = assign_expr.right.take_in(ctx.ast);
        ctx.state.changed = true;
        false
    }

    /// Do remove top level vars in script mode.
    fn keep_top_level_var_in_script_mode(ctx: &Ctx<'a, '_>) -> bool {
        ctx.scoping.current_scope_id() == ctx.scoping().root_scope_id()
            && ctx.source_type().is_script()
    }
}

#[cfg(test)]
mod test {
    use oxc_span::SourceType;

    use crate::{
        CompressOptions,
        tester::{
            test_options, test_options_source_type, test_same_options,
            test_same_options_source_type,
        },
    };

    #[test]
    fn remove_unused_variable_declaration() {
        let options = CompressOptions::smallest();
        test_options("var x", "", &options);
        test_options("var x = 1", "", &options);
        test_options("var x = foo", "foo", &options);
        test_same_options("var x; foo(x)", &options);
        test_same_options("export var x", &options);
        // TypeError
        test_options(
            "var x = class extends (() => {}) {}",
            "(class extends (() => {}) {})",
            &options,
        );
    }

    #[test]
    fn remove_unused_function_declaration() {
        let options = CompressOptions::smallest();
        test_options("function foo() {}", "", &options);
        test_same_options("function foo() { bar } foo()", &options);
        test_same_options("export function foo() {} foo()", &options);
    }

    #[test]
    fn remove_unused_class_declaration() {
        let options = CompressOptions::smallest();
        test_options("class C {}", "", &options);
        test_same_options("class C extends Foo { }", &options);

        test_options("class C { static {} }", "", &options);
        test_same_options("class C { static { foo } }", &options);

        test_options("class C { foo() {} }", "", &options);
        test_same_options("class C { [foo]() {} }", &options);

        test_same_options("class C { [foo] }", &options);
        test_same_options("class C { foo = bar() }", &options);
        test_same_options("class C { foo = 1 }", &options);
        test_same_options("class C { static foo = bar }", &options);

        test_same_options("class C { accessor foo = 1}", &options);
    }

    #[test]
    fn remove_unused_assignment_expression() {
        let options = CompressOptions::smallest();
        // Vars are not handled yet due to TDZ.
        test_same_options("var x = 1; x = 2;", &options);
        test_same_options("var x = 1; x = foo();", &options);
        test_same_options("export var foo; foo = 0;", &options);
        test_same_options("var x = 1; x = 2, foo(x)", &options);
        test_same_options("function foo() { return t = x(); } foo();", &options);
        test_same_options("function foo() { var t; return t = x(); } foo();", &options);
        test_same_options("function foo(t) { return t = x(); } foo();", &options);

        test_options("let x = 1; x = 2;", "", &options);
        test_options("let x = 1; x = foo();", "foo()", &options);
        test_same_options("export let foo; foo = 0;", &options);
        test_same_options("let x = 1; x = 2, foo(x)", &options);
        test_same_options("function foo() { return t = x(); } foo();", &options);
        test_options(
            "function foo() { let t; return t = x(); } foo();",
            "function foo() { return x() } foo()",
            &options,
        );
        test_same_options("function foo(t) { return t = x(); } foo();", &options);

        test_same_options("for(let i;;) foo(i)", &options);
        test_same_options("for(let i in []) foo(i)", &options);

        test_options("var a; ({ a: a } = {})", "var a; ({ a } = {})", &options);
        test_options("var a; b = ({ a: a })", "var a; b = ({ a })", &options);

        test_options("let foo = {}; foo = 1", "", &options);

        test_same_options(
            "let bracketed = !1; for(;;) bracketed = !bracketed, log(bracketed)",
            &options,
        );

        let options = CompressOptions::smallest();
        let source_type = SourceType::cjs();
        test_same_options_source_type("var x = 1; x = 2;", source_type, &options);
        test_same_options_source_type("var x = 1; x = 2, foo(x)", source_type, &options);
        test_same_options_source_type(
            "function foo() { var x = 1; x = 2, bar() } foo()",
            source_type,
            &options,
        );
    }

    #[test]
    fn keep_in_script_mode() {
        let options = CompressOptions::smallest();
        let source_type = SourceType::cjs();
        test_same_options_source_type("var x = 1; x = 2;", source_type, &options);
        test_same_options_source_type("var x = 1; x = 2, foo(x)", source_type, &options);

        test_options_source_type("class C {}", "class C {}", source_type, &options);
    }
}
