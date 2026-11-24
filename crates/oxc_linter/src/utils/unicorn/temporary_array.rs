use oxc_ast::ast::{ChainElement, Expression};
use oxc_semantic::IsGlobalReference;

use crate::context::LintContext;

/// Returns true if the expression (probably) returns a temporary array, i.e. one that can be safely
/// mutated in-place without side effects:
/// - [a, b]
/// - arr.map(fn)
/// - Array(4)
/// - Array.from(iter, fn)
/// - Object.keys(obj)
///
/// Type information is not used, so instance methods like .map(), .filter() etc. are matched by
/// name, meaning the function may have some false positives.
pub fn is_temporary_array(mut expr: &Expression<'_>, ctx: &LintContext<'_>) -> bool {
    loop {
        let call = match expr.get_inner_expression() {
            Expression::ArrayExpression(_) => return true,
            Expression::NewExpression(new_expr) => {
                return new_expr.callee.get_identifier_reference().is_some_and(|ident| {
                    ident.name == "Array" && ident.is_global_reference(ctx.scoping())
                });
            }
            Expression::CallExpression(call) => call,
            Expression::ChainExpression(chain) => match &chain.expression {
                ChainElement::CallExpression(call) => call,
                ChainElement::TSNonNullExpression(non_null) => {
                    expr = &non_null.expression;
                    continue;
                }
                _ => return false,
            },
            _ => return false,
        };

        if let Some(ident) = call.callee.get_identifier_reference() {
            return ident.name == "Array" && ident.is_global_reference(ctx.scoping());
        }

        let Some(member) = call.callee.get_member_expr() else {
            return false;
        };
        let Some(method) = member.static_property_name() else {
            return false;
        };
        let Some(method) = method.as_str() else {
            return false;
        };
        let global_callee = member
            .object()
            .get_identifier_reference()
            .filter(|ident| ident.is_global_reference(ctx.scoping()))
            .map(|ident| ident.name.as_str());

        match (global_callee, method) {
            (
                _,
                "map" | "filter" | "flatMap" | "flat" | "slice" | "concat" | "toSorted"
                | "toReversed" | "toSpliced" | "with",
            )
            | (Some("Array"), "from" | "of")
            | (
                Some("Object"),
                "keys" | "values" | "entries" | "getOwnPropertyNames" | "getOwnPropertySymbols",
            )
            | (Some("Reflect"), "ownKeys") => return true,
            // These methods return their receiver, so they return a temporary only if the receiver
            // was also a temporary.
            (_, "sort" | "reverse") => expr = member.object(),
            _ => return false,
        }
    }
}

#[cfg(test)]
mod test {
    use std::{rc::Rc, sync::Arc};

    use oxc_allocator::Allocator;
    use oxc_ast::ast::Statement;
    use oxc_parser::Parser;
    use oxc_semantic::SemanticBuilder;
    use oxc_span::SourceType;

    use super::is_temporary_array;
    use crate::{
        ContextHost, ModuleRecord,
        context::{ContextSubHost, ContextSubHostOptions},
        options::LintOptions,
    };

    fn assert_is_temporary_array(source: &str, expected: bool) {
        let allocator = Allocator::default();
        let parser_ret = Parser::new(&allocator, source, SourceType::ts()).parse();
        assert!(parser_ret.diagnostics.is_empty(), "Parse error in: {source}");
        let program = allocator.alloc(parser_ret.program);
        let Some(Statement::ExpressionStatement(statement)) = program.body.last() else {
            panic!("Expected a final expression statement in: {source}");
        };
        let semantic_ret = SemanticBuilder::new_linter().build(program);
        assert!(semantic_ret.diagnostics.is_empty(), "Semantic error in: {source}");
        let ctx = Rc::new(ContextHost::new(
            "test.ts",
            vec![ContextSubHost::new(
                semantic_ret.semantic,
                Arc::new(ModuleRecord::default()),
                0,
                ContextSubHostOptions::default(),
            )],
            &allocator,
            LintOptions::default(),
            Arc::default(),
        ))
        .spawn_for_test();

        assert_eq!(
            is_temporary_array(&statement.expression, &ctx),
            expected,
            "Failed for: {source}"
        );
    }

    #[test]
    fn test_array_creation() {
        for source in [
            "[]",
            "[1, 2]",
            "[,,]",
            "[...items]",
            "[1, ...items]",
            "new Array",
            "new Array(3)",
            "Array()",
            "Array(4)",
            "Array(1, 2)",
            "Array.from(items)",
            "Array.from(iter, mapFn)",
            "Array.from(iter, mapFn, thisArg)",
            "Array.of(1, 2)",
            "Object.keys(object)",
            "Object.values(object)",
            "Object.entries(object)",
            "Object.getOwnPropertyNames(object)",
            "Object.getOwnPropertySymbols(object)",
            "Reflect.ownKeys(object)",
            "items.map(fn)",
            "items.filter(fn)",
            "items.flatMap(fn)",
            "items.flat()",
            "items.slice()",
            "items.concat(other)",
            "items.toSorted(compareFn)",
            "items.toReversed()",
            "items.toSpliced(0, 1)",
            "items.with(0, value)",
        ] {
            assert_is_temporary_array(source, true);
        }
    }

    #[test]
    fn test_wrappers_and_chains() {
        for source in [
            "((items.map(fn)))",
            "(Array)(4)",
            "items.map(fn) as number[]",
            "items.map(fn) satisfies number[]",
            "<number[]>items.map(fn)",
            "items.map(fn)!",
            "items?.map(fn)",
            "items.map?.(fn)",
            "items?.map(fn)!",
            "items['map'](fn)",
            "Array['from'](items)",
            "Object['values'](object)",
            "Object['getOwnPropertyNames'](object)",
            "Reflect['ownKeys'](object)",
            "Array.from<number>(items)",
            "[...items].sort(compareFn)",
            "Object.values(object).reverse()",
            "(items?.map(fn)!).sort().reverse().sort(compareFn)",
        ] {
            assert_is_temporary_array(source, true);
        }
    }

    #[test]
    fn test_non_temporary_expressions() {
        for source in [
            "items",
            "items.map",
            "items?.map",
            "getArray()",
            "new CustomArray(3)",
            "Array.isArray(items)",
            "Array.fromAsync(items)",
            "Factory.from(items)",
            "items.values()",
            "items.keys()",
            "items.entries()",
            "items.getOwnPropertyNames()",
            "items.getOwnPropertySymbols()",
            "items.ownKeys()",
            "Object.ownKeys(object)",
            "Reflect.getOwnPropertyNames(object)",
            "Object.getOwnPropertyDescriptors(object)",
            "Object.fromEntries(entries)",
            "Object.groupBy(items, fn)",
            "Object.freeze(items)",
            "items[method](fn)",
            "items['\\uD800']()",
            "items.map(fn).unknown()",
            "items.sort().reverse()",
            "items.reverse().sort()",
            "(saved = items.map(fn))",
            "(saved = []).sort()",
            "(0, items.map(fn))",
            "condition ? [] : []",
            "const temporary = items.map(fn); temporary",
            "const values = Object.values; values(object)",
        ] {
            assert_is_temporary_array(source, false);
        }
    }

    #[test]
    fn test_shadowed_globals() {
        for source in [
            "const Array = custom; new Array(3)",
            "const Array = custom; Array(4)",
            "const Array = custom; Array.from(items)",
            "const Array = custom; Array.from(iter, mapFn)",
            "const Array = custom; Array.of(1, 2)",
            "const Object = custom; Object.keys(object)",
            "const Object = custom; Object.values(object)",
            "const Object = custom; Object.entries(object)",
            "const Object = custom; Object.getOwnPropertyNames(object)",
            "const Object = custom; Object.getOwnPropertySymbols(object)",
            "const Reflect = custom; Reflect.ownKeys(object)",
            "import { Array } from 'custom'; Array.from(items)",
            "import { Array } from 'custom'; Array(4)",
            "import { Object } from 'custom'; Object.values(object)",
            "import { Reflect } from 'custom'; Reflect.ownKeys(object)",
        ] {
            assert_is_temporary_array(source, false);
        }
    }

    #[test]
    fn test_method_name_heuristic() {
        // Return types and aliases are not checked for instance methods.
        assert_is_temporary_array(
            "const shared = []; const custom = { map: () => shared }; custom.map(fn)",
            true,
        );
    }
}
