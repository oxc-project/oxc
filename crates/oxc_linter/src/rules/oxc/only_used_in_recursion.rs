use oxc_ast::{
    AstKind,
    ast::{
        Argument, AssignmentTarget, BindingIdentifier, BindingPattern, BindingProperty,
        CallExpression, Expression, FormalParameters, JSXAttributeItem, JSXElementName,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{NodeId, SymbolId};
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode, ast_util::get_function_like_declaration, context::LintContext, fixer::Fix, rule::Rule,
};

fn only_used_in_recursion_diagnostic(span: Span, param_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Parameter `{param_name}` is only used in recursive calls"
    ))
    .with_help(
        "Remove the argument and its usage. Alternatively, use the argument in the function body.",
    )
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct OnlyUsedInRecursion;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Checks for arguments that are only used in recursion with no side-effects.
    ///
    /// Inspired by [the `only_used_in_recursion` rule in Clippy](https://rust-lang.github.io/rust-clippy/master/#only_used_in_recursion).
    ///
    /// ### Why is this bad?
    ///
    /// Supplying an argument that is only used in recursive calls is likely a mistake.
    ///
    /// It increases cognitive complexity and may impact performance.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// function test(onlyUsedInRecursion) {
    ///     return test(onlyUsedInRecursion);
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// function f(a: number): number {
    ///    if (a == 0) {
    ///        return 1
    ///    } else {
    ///        return f(a - 1)
    ///    }
    /// }
    /// ```
    OnlyUsedInRecursion,
    oxc,
    correctness,
    dangerous_fix,
    version = "0.1.1",
    short_description = "Checks for arguments that are only used in recursion with no side effects.",
);

fn is_exported(id: &BindingIdentifier<'_>, ctx: &LintContext<'_>) -> bool {
    let module_record = ctx.module_record();
    module_record.exported_bindings.contains_key(id.name.as_str())
        || module_record.export_default.is_some_and(|default| default == id.span)
}

impl Rule for OnlyUsedInRecursion {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (function_id, function_parameters, function_span) = match node.kind() {
            AstKind::Function(function) => {
                if function.is_typescript_syntax() {
                    return;
                }

                if let Some(binding_ident) = get_function_like_declaration(node, ctx) {
                    (binding_ident, &function.params, function.span)
                } else if let Some(function_id) = &function.id {
                    (function_id, &function.params, function.span)
                } else {
                    return;
                }
            }
            AstKind::ArrowFunctionExpression(arrow_function) => {
                if let Some(binding_ident) = get_function_like_declaration(node, ctx) {
                    (binding_ident, &arrow_function.params, arrow_function.span)
                } else {
                    return;
                }
            }
            _ => return,
        };

        if is_function_reassigned(function_id, ctx) {
            return;
        }

        for (parameter_index, formal_parameter) in function_parameters.items.iter().enumerate() {
            match &formal_parameter.pattern {
                BindingPattern::BindingIdentifier(parameter)
                    if is_parameter_only_used_in_recursion(
                        function_id,
                        parameter,
                        parameter_index,
                        ctx,
                    ) =>
                {
                    report_parameter(
                        ctx,
                        function_id,
                        function_parameters,
                        parameter,
                        parameter_index,
                        function_span,
                    );
                }
                BindingPattern::ObjectPattern(pattern) => {
                    for property in &pattern.properties {
                        if let Some(ident) = property.value.get_binding_identifier()
                            && let Some(name) = property.key.name()
                            && is_jsx_property_only_used_in_recursion(
                                ident,
                                &name,
                                function_id,
                                ctx,
                            )
                        {
                            report_jsx_property(ctx, function_id, property, ident, &name);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn report_parameter(
    ctx: &LintContext,
    function_id: &BindingIdentifier,
    function_parameters: &FormalParameters,
    parameter: &BindingIdentifier,
    parameter_index: usize,
    function_span: Span,
) {
    let diagnostic = || only_used_in_recursion_diagnostic(parameter.span, parameter.name.as_str());
    let can_fix =
        parameter_index + 1 == function_parameters.items.len() && !is_exported(function_id, ctx);

    if !can_fix {
        ctx.diagnostic(diagnostic());
        return;
    }

    ctx.diagnostic_with_dangerous_fix(diagnostic(), |fixer| {
        let mut fix = fixer.new_fix_with_capacity(
            ctx.semantic().symbol_references(parameter.symbol_id()).count() + 1,
        );

        // Delete the parameter, including the comma before it
        fix.push(delete_with_preceding_separator(ctx.source_text(), parameter.span));

        for reference in ctx.semantic().symbol_references(parameter.symbol_id()) {
            let node = ctx.nodes().get_node(reference.node_id());
            // Delete the argument reference, including the comma before it
            fix.push(delete_with_preceding_separator(ctx.source_text(), node.span()));
        }

        // Search for references to the function and remove the argument.
        for reference in ctx.semantic().symbol_references(function_id.symbol_id()) {
            let node = ctx.nodes().get_node(reference.node_id());

            let AstKind::CallExpression(call_expr) = ctx.nodes().parent_kind(node.id()) else {
                continue;
            };
            if call_expr.arguments.len() != function_parameters.items.len()
                || function_span.contains_inclusive(call_expr.span)
            {
                continue;
            }

            fix.push(delete_with_preceding_separator(
                ctx.source_text(),
                call_expr.arguments[parameter_index].span(),
            ));
        }

        fix.with_message("Remove unused argument")
    });
}

fn report_jsx_property(
    ctx: &LintContext,
    function_id: &BindingIdentifier,
    property: &BindingProperty,
    property_ident: &BindingIdentifier,
    property_name: &str,
) {
    let diagnostic = || only_used_in_recursion_diagnostic(property.span, property_name);

    if is_exported(function_id, ctx) {
        ctx.diagnostic(diagnostic());
        return;
    }

    let property_symbol_id = property_ident.symbol_id();
    if ctx
        .semantic()
        .symbol_references(property_symbol_id)
        .any(|reference| is_used_with_spread_attribute(reference.node_id(), ctx))
    {
        // If the JSXElement has a spread attribute, we cannot apply a fix safely,
        // as the same property name could exist within the spread attribute.
        ctx.diagnostic(diagnostic());
        return;
    }

    ctx.diagnostic_with_dangerous_fix(diagnostic(), |fixer| {
        let reference_count = ctx.semantic().symbol_references(property_symbol_id).count();
        let mut fix = fixer.new_fix_with_capacity(reference_count + 1);
        fix.push(delete_with_surrounding_separators(ctx.source_text(), property.span));

        // Search for references to the property and remove the JSX attribute.
        for reference in ctx.semantic().symbol_references(property_symbol_id) {
            if let Some(attr) = ctx
                .nodes()
                .ancestors(reference.node_id())
                .find_map(|node| node.kind().as_jsx_attribute())
            {
                fix.push(Fix::delete(attr.span()));
            }
        }

        fix.with_message("Remove unused property")
    });
}

fn is_used_with_spread_attribute(node_id: NodeId, ctx: &LintContext) -> bool {
    ctx.nodes().ancestor_kinds(node_id).any(|kind| match kind {
        AstKind::JSXOpeningElement(opening_element) => opening_element
            .attributes
            .iter()
            .any(|attr| matches!(attr, JSXAttributeItem::SpreadAttribute(_))),
        _ => false,
    })
}

fn is_parameter_only_used_in_recursion(
    function_id: &BindingIdentifier,
    parameter: &BindingIdentifier,
    parameter_index: usize,
    ctx: &LintContext,
) -> bool {
    let mut references = ctx.semantic().symbol_references(parameter.symbol_id()).peekable();

    // Avoid returning true for an empty iterator
    if references.peek().is_none() {
        return false;
    }

    let function_symbol_id = function_id.symbol_id();

    for reference in references {
        let AstKind::CallExpression(call_expr) = ctx.nodes().parent_kind(reference.node_id())
        else {
            return false;
        };

        let Some(call_arg) = call_expr.arguments.get(parameter_index) else {
            return false;
        };

        if let Argument::Identifier(ident) = call_arg
            && ident.name != parameter.name
        {
            return false;
        }

        if !is_recursive_call(call_expr, function_symbol_id, ctx) {
            return false;
        }
    }

    true
}

fn is_jsx_property_only_used_in_recursion(
    ident: &BindingIdentifier,
    property_name: &str,
    function_ident: &BindingIdentifier,
    ctx: &LintContext,
) -> bool {
    let mut references = ctx.semantic().symbol_references(ident.symbol_id()).peekable();
    if references.peek().is_none() {
        return false;
    }

    let function_symbol_id = function_ident.symbol_id();
    for reference in references {
        // Conditions:
        // 1. The reference is inside a JSXExpressionContainer.
        // 2. The JSXElement calls the recursive function itself.
        // 3. The reference is in a JSXAttribute, and the attribute name has the same name as the function.
        let expression_container = ctx.nodes().parent_node(reference.node_id());
        let AstKind::JSXExpressionContainer(_) = expression_container.kind() else {
            // In this case, we simply ignore the references inside JSXExpressionContainer that are not single-node expression.
            //   e.g. <Increment count={count+1} />
            //
            // To support this case, we need to check whether expression contains side-effect like ++val
            return false;
        };

        let Some(attr) = ctx
            .nodes()
            .ancestors(expression_container.id())
            .find_map(|node| node.kind().as_jsx_attribute())
        else {
            return false;
        };

        let Some(attr_name) = attr.name.as_identifier() else {
            return false;
        };
        if attr_name.name != property_name {
            return false;
        }

        let Some(opening_element) = ctx
            .nodes()
            .ancestors(reference.node_id())
            .find_map(|node| node.kind().as_jsx_opening_element())
        else {
            return false;
        };

        let Some(jsx_ident_symbol_id) = get_jsx_element_symbol_id(&opening_element.name, ctx)
        else {
            return false;
        };
        if jsx_ident_symbol_id != function_symbol_id {
            return false;
        }
    }

    true
}

fn is_recursive_call(
    call_expr: &CallExpression,
    function_symbol_id: SymbolId,
    ctx: &LintContext,
) -> bool {
    let Expression::Identifier(identifier) = &call_expr.callee else { return false };
    ctx.scoping().get_reference(identifier.reference_id()).symbol_id() == Some(function_symbol_id)
}

fn is_function_reassigned(function_id: &BindingIdentifier, ctx: &LintContext) -> bool {
    ctx.semantic().symbol_references(function_id.symbol_id()).any(|reference| {
        let reference_node = ctx.nodes().get_node(reference.node_id());

        // Check if this reference is on the left side of an assignment
        let AstKind::AssignmentExpression(assignment) =
            ctx.nodes().parent_kind(reference.node_id())
        else {
            return false;
        };
        let AssignmentTarget::AssignmentTargetIdentifier(ident) = &assignment.left else {
            return false;
        };
        ident.span == reference_node.span()
    })
}

fn get_jsx_element_symbol_id(node: &JSXElementName<'_>, ctx: &LintContext<'_>) -> Option<SymbolId> {
    let identifier = match node {
        JSXElementName::IdentifierReference(ident) => Some(ident.as_ref()),
        JSXElementName::MemberExpression(expr) => expr.get_identifier(),
        JSXElementName::Identifier(_)
        | JSXElementName::NamespacedName(_)
        | JSXElementName::ThisExpression(_) => None,
    }?;

    ctx.scoping().get_reference(identifier.reference_id()).symbol_id()
}

#[derive(Clone, Copy)]
enum Direction {
    Forward,
    Backward,
}

// Skips whitespace and commas in a given direction and
// returns the byte offset of the next non-skipped character if found.
#[expect(clippy::cast_possible_truncation)]
fn skip_to_next_char(s: &str, start: u32, direction: Direction) -> Option<u32> {
    let start = start as usize;
    match direction {
        Direction::Forward => {
            let slice = s.get(start..)?;
            for (offset, c) in slice.char_indices() {
                if !c.is_whitespace() && c != ',' {
                    return Some((start + offset) as u32);
                }
            }
            None
        }
        Direction::Backward => {
            let slice = s.get(..start)?;
            let mut result = None;
            for (i, c) in slice.char_indices().rev() {
                if c.is_whitespace() || c == ',' {
                    result = Some(i as u32);
                } else {
                    break;
                }
            }
            result
        }
    }
}

fn delete_with_preceding_separator(source: &str, span: Span) -> Fix {
    let start = skip_to_next_char(source, span.start, Direction::Backward).unwrap_or(span.start);
    Fix::delete(Span::new(start, span.end))
}

fn delete_with_surrounding_separators(source: &str, span: Span) -> Fix {
    let start = skip_to_next_char(source, span.start, Direction::Backward).unwrap_or(span.start);
    let end = skip_to_next_char(source, span.end, Direction::Forward).unwrap_or(span.end);
    Fix::delete(Span::new(start, end))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // no args, no recursion
        "
            function test() {
                // some code
            }
        ",
        "
            function test(arg0) {
                test(arg0+1)
            }
        ",
        // unused arg, no recursion
        "
            function test(arg0) {
                // arg0 not used
            }
        ",
        // no recursion, assignment pattern
        r"
            function test({ arg0 = 10 }) {
                return arg0;
            }
        ",
        "
            function test(arg0) {
                anotherTest(arg0);
            }

            function anotherTest(arg) { }
        ",
        // conditional recursion
        "
            function test(arg0) {
                if (arg0 > 0) {
                    test(arg0 - 1);
                }
            }
        ",
        "
            function test(arg0, arg1) {
                // only arg0 used in recursion
                arg0
                test(arg0);
            }
        ",
        // allowed case
        "
            function test() {
                test()
            }
        ",
        // arg not passed to recursive call
        "
            function test(arg0) {
                arg0()
            }
        ",
        // arg not passed to recursive call (arrow)
        "
            const test = (arg0) => {
                test();
            };
        ",
        "function test(arg0) { }",
        // args in wrong order
        "
            function test(arg0, arg1) {
                test(arg1, arg0)
            }
        ",
        // arguments swapped in recursion
        r"
            function test(arg0, arg1) {
                test(arg1, arg0);
            }
        ",
        // arguments swapped in recursion (arrow)
        r"
            const test = (arg0, arg1) => {
                test(arg1, arg0);
            };
        ",
        // https://github.com/swc-project/swc/blob/3ca954b9f9622ed400308f2af35242583a4bdc3d/crates/swc_ecma_transforms_base/src/helpers/_get.js#L1-L16
        r#"
        function _get(target, property, receiver) {
            if (typeof Reflect !== "undefined" && Reflect.get) {
                _get = Reflect.get;
            } else {
                _get = function get(target, property, receiver) {
                    var base = _super_prop_base(target, property);
                    if (!base) return;
                    var desc = Object.getOwnPropertyDescriptor(base, property);
                    if (desc.get) {
                        return desc.get.call(receiver || target);
                    }
                    return desc.value;
                };
            }
            return _get(target, property, receiver || target);
        }
        "#,
        r#"
        var validator = function validator(node, key, val) {
            var validator = node.operator === "in" ? inOp : expression;
            validator(node, key, val);
        };
        validator()
        "#,
        // no params, no recursion
        "
           function Test() {
                return <Test1 />;
            }
        ",
        // allowed case: The parameter 'depth' is used outside of the recursive call.
        // it is logged and reassigned, so the linter does not flag it as only used in recursion.
        "
            function Listitem({ depth }) {
                 console.log(depth);
                 depth = depth + 1;
                 return <Listitem depth={depth}/>;
             }
        ",
        "
            function Listitem({ depth }) {
                console.log(depth);
                return <Listitem depth={depth}/>;
            }
        ",
        // allowed case
        // multi-node expressions are not supported for now
        "
            function Listitem({ depth }) {
                return <Listitem depth={depth + 1} />
            }
        ",
        // conditional recursion
        "
            function Listitem({ depth }) {
                if (depth < 10) {
                    return <Listitem depth={depth + 1} />;
                }
                return null;
            }
        ",
        // conditional recursion (shorthand)
        "
            function Listitem({ depth }) {
                return depth > 5 ? <Listitem depth={depth + 1} /> : null;
            }
        ",
        // reference inside jsx expression but not attribute
        "
            function List({item}) {
                return (
                    <List>
                        {item}
                    </List>
                )
            }
        ",
        // create_diagnostic_jsx - JSX element without property match
        r"
            function Listitem({ depth }) {
                return <Listitem level={depth} />;
            }
        ",
        // JSX attribute not referencing function name
        r"
            function TestComponent({ body }) {
                return <AnotherComponent body={body} />;
            }
        ",
        // property not used in recursion
        r"
            function test({prop1, prop2}) {
                return (<></>)
            }
        ",
        // property swapped in recursion
        r"
            function Test({prop1, prop2}) {
                return (<Test prop1={prop2} prop2={prop1} />)
            }
        ",
        // arguments swapped in recursion (arrow)
        r"
           const Test = ({prop1, prop2}) => {
                return (<Test prop1={prop2} prop2={prop1} />)
            }
        ",
    ];

    let fail = vec![
        "
            function test(arg0) {
                return test(arg0);
            }
        ",
        r#"
            function test(arg0, arg1) {
                return test("", arg1);
            }
        "#,
        // Argument Not Altered in Recursion
        r"
            function test(arg0) {
                test(arg0);
            }
        ",
        // Wrong Number of Arguments in Recursion
        r"
            function test(arg0, arg1) {
                test(arg0);
            }
        ",
        r"
            module.exports = function test(a) {
                test(a)
            }
        ",
        r"
            export function test(a) {
                test(a)
            }
        ",
        // https://github.com/oxc-project/oxc/issues/4817
        // "
        //     const test = function test(arg0) {
        //         return test(arg0);
        //     }
        // ",
        "
            const a = (arg0) => {
                return a(arg0);
            }
        ",
        "//¿
function writeChunks(a,callac){writeChunks(m,callac)}writeChunks(i,{})",
        "
            function ListItem({ depth }) {
                return <ListItem depth={depth} />
            }
        ",
        "
            function ListItem({ depth: listDepth }) {
                return <ListItem depth={listDepth} />;
            }
        ",
        "
            function ListItem({depth = 0}) {
                return <ListItem depth={depth} />
            }
        ",
        "
            function ListItem({depth, ...otherProps}) {
                            return <ListItem depth={depth} />
            }
        ",
        r"
            function Test({a, b}) {
                return (
                    <Test a={a} b={b}/>
                )
            }
        ",
    ];

    let fix = vec![
        (
            r#"function test(a) {
             test(a)
            }

            test("")
            "#,
            r"function test() {
             test()
            }

            test()
            ",
        ),
        (
            r#"
            test(foo, bar);
            function test(arg0, arg1) {
                return test("", arg1);
            }
            "#,
            r#"
            test(foo);
            function test(arg0) {
                return test("");
            }
            "#,
        ),
        // Expecting no fix: function is exported
        (
            r"export function test(a) {
                  test(a)
              }
            ",
            r"export function test(a) {
                  test(a)
              }
            ",
        ),
        (
            r"function test(a) {
                  test(a)
              }
              export { test };
            ",
            r"function test(a) {
                  test(a)
              }
              export { test };
            ",
        ),
        (
            r"function test(a) {
                  test(a)
              }
              export default test;
            ",
            r"function test() {
                  test()
              }
              export default test;
            ",
        ),
        (
            r"function Test({a, b}) {
                  a++;
                  return (
                      <Test a={a} b={b}/>
                  )
              }
              export default test;
            ",
            r"function Test({a}) {
                  a++;
                  return (
                      <Test a={a} />
                  )
              }
              export default test;
            ",
        ),
        (
            r"function Test({a, b}) {
                console.log(b)
                return (<Test a={a} b={b} />)
            }
            ",
            r"function Test({b}) {
                console.log(b)
                return (<Test  b={b} />)
            }
            ",
        ),
        (
            r"function Test({a, b}) {
                b++;
                return (<Test a={a} b={b}/>)
            }
            ",
            r"function Test({b}) {
                b++;
                return (<Test  b={b}/>)
            }
            ",
        ),
        // Expecting no fix: function is exported
        (
            r"function ListItem({depth, ...otherProps}) {
                return <ListItem depth={depth} {...otherProps}/>
            }
            ",
            r"function ListItem({depth, ...otherProps}) {
                return <ListItem depth={depth} {...otherProps}/>
            }
            ",
        ),
        // Test that trailing commas are removed at external call sites
        (
            r"function recurse(used, unused) {
                return recurse(used + 1, unused);
            }
            recurse(0, 'delete_me');
            ",
            r"function recurse(used) {
                return recurse(used + 1);
            }
            recurse(0);
            ",
        ),
    ];

    Tester::new(OnlyUsedInRecursion::NAME, OnlyUsedInRecursion::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
