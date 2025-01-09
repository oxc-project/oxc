use oxc_ast::{
    ast::{
        BindingIdentifier, BindingPatternKind, BindingProperty, CallExpression, Expression,
        FormalParameters, JSXAttributeItem, JSXElementName,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{NodeId, SymbolId};
use oxc_span::{GetSpan, Span};

use crate::{
    ast_util::get_function_like_declaration, context::LintContext, fixer::Fix, rule::Rule, AstNode,
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
    /// Inspired by https://rust-lang.github.io/rust-clippy/master/#/only_used_in_recursion
    ///
    /// ### Why is this bad?
    ///
    /// Supplying an argument that is only used in recursive calls is likely a mistake.
    ///
    /// It increase cognitive complexity and may impact performance.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// function test(only_used_in_recursion) {
    ///     return test(only_used_in_recursion);
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
    dangerous_fix
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

        if is_function_maybe_reassigned(function_id, ctx) {
            return;
        }

        for (arg_index, formal_parameter) in function_parameters.items.iter().enumerate() {
            match &formal_parameter.pattern.kind {
                BindingPatternKind::BindingIdentifier(arg) => {
                    if is_argument_only_used_in_recursion(function_id, arg, arg_index, ctx) {
                        create_diagnostic(
                            ctx,
                            function_id,
                            function_parameters,
                            arg,
                            arg_index,
                            function_span,
                        );
                    }
                }
                BindingPatternKind::ObjectPattern(pattern) => {
                    for property in &pattern.properties {
                        let Some(ident) = property.value.get_binding_identifier() else {
                            continue;
                        };

                        let Some(name) = property.key.name() else {
                            continue;
                        };
                        if is_property_only_used_in_recursion_jsx(ident, &name, function_id, ctx) {
                            create_diagnostic_jsx(ctx, function_id, property);
                        }
                    }
                }
                _ => continue,
            }
        }
    }
}

fn create_diagnostic(
    ctx: &LintContext,
    function_id: &BindingIdentifier,
    function_parameters: &FormalParameters,
    arg: &BindingIdentifier,
    arg_index: usize,
    function_span: Span,
) {
    let is_last_arg = arg_index == function_parameters.items.len() - 1;

    let is_diagnostic_only = !is_last_arg || is_exported(function_id, ctx);

    if is_diagnostic_only {
        return ctx.diagnostic(only_used_in_recursion_diagnostic(arg.span, arg.name.as_str()));
    }

    ctx.diagnostic_with_dangerous_fix(
        only_used_in_recursion_diagnostic(arg.span, arg.name.as_str()),
        |fixer| {
            let mut fix = fixer.new_fix_with_capacity(
                ctx.semantic().symbol_references(arg.symbol_id()).count() + 1,
            );
            fix.push(Fix::delete(arg.span()));

            for reference in ctx.semantic().symbol_references(arg.symbol_id()) {
                let node = ctx.nodes().get_node(reference.node_id());
                fix.push(Fix::delete(node.span()));
            }

            // search for references to the function and remove the argument
            for reference in ctx.semantic().symbol_references(function_id.symbol_id()) {
                let node = ctx.nodes().get_node(reference.node_id());

                if let Some(AstKind::CallExpression(call_expr)) = ctx.nodes().parent_kind(node.id())
                {
                    if call_expr.arguments.len() != function_parameters.items.len()
                        || function_span.contains_inclusive(call_expr.span)
                    {
                        continue;
                    }

                    let arg_to_delete = call_expr.arguments[arg_index].span();
                    fix.push(Fix::delete(Span::new(
                        arg_to_delete.start,
                        skip_to_next_char(
                            ctx.source_text(),
                            arg_to_delete.end,
                            &Direction::Forward,
                        )
                        .unwrap_or(arg_to_delete.end),
                    )));
                }
            }

            fix
        },
    );
}

fn create_diagnostic_jsx(
    ctx: &LintContext,
    function_id: &BindingIdentifier,
    property: &BindingProperty,
) {
    let Some(property_name) = &property.key.static_name() else { return };
    if is_exported(function_id, ctx) {
        return ctx.diagnostic(only_used_in_recursion_diagnostic(property.span(), property_name));
    }

    let Some(property_ident) = property.value.get_binding_identifier() else { return };
    let property_symbol_id = property_ident.symbol_id();
    let mut references = ctx.semantic().symbol_references(property_symbol_id);

    let has_spread_attribute = references.any(|x| used_with_spread_attribute(x.node_id(), ctx));

    if has_spread_attribute {
        // If the JSXElement has a spread attribute, we cannot apply a fix safely,
        // as the same property name could be exist within the spread attribute.
        return ctx.diagnostic(only_used_in_recursion_diagnostic(property.span(), property_name));
    }

    let Some(property_name) = property.key.static_name() else {
        return;
    };

    ctx.diagnostic_with_dangerous_fix(
        only_used_in_recursion_diagnostic(property.span, &property_name),
        |fixer| {
            let mut fix = fixer.new_fix_with_capacity(references.count() + 1);

            let source = ctx.source_text();
            let span_start = skip_to_next_char(source, property.span.start, &Direction::Backward)
                .unwrap_or(property.span.start);
            let span_end =
                skip_to_next_char(ctx.source_text(), property.span.end, &Direction::Forward)
                    .unwrap_or(property.span.end);

            fix.push(Fix::delete(Span::new(span_start, span_end)));

            // search for references to the function and remove the property
            for reference in ctx.semantic().symbol_references(property_symbol_id) {
                let mut ancestor_ids = ctx.nodes().ancestor_ids(reference.node_id());

                let Some(attr) =
                    ancestor_ids.find_map(|node| match ctx.nodes().get_node(node).kind() {
                        AstKind::JSXAttributeItem(attr) => Some(attr),
                        _ => None,
                    })
                else {
                    continue;
                };

                fix.push(Fix::delete(attr.span()));
            }

            fix
        },
    );
}

fn used_with_spread_attribute(node_id: NodeId, ctx: &LintContext) -> bool {
    ctx.nodes().ancestor_kinds(node_id).any(|kind| match kind {
        AstKind::JSXOpeningElement(opening_element) => opening_element
            .attributes
            .iter()
            .any(|attr| matches!(attr, JSXAttributeItem::SpreadAttribute(_))),
        _ => false,
    })
}

fn is_argument_only_used_in_recursion<'a>(
    function_id: &'a BindingIdentifier,
    arg: &'a BindingIdentifier,
    arg_index: usize,
    ctx: &'a LintContext<'_>,
) -> bool {
    let mut references = ctx.semantic().symbol_references(arg.symbol_id()).peekable();

    // Avoid returning true for an empty iterator
    if references.peek().is_none() {
        return false;
    }

    let function_symbol_id = function_id.symbol_id();

    for reference in references {
        let Some(AstKind::Argument(argument)) = ctx.nodes().parent_kind(reference.node_id()) else {
            return false;
        };
        let Some(AstKind::CallExpression(call_expr)) =
            ctx.nodes().parent_kind(ctx.nodes().parent_node(reference.node_id()).unwrap().id())
        else {
            return false;
        };

        let Some(call_arg) = call_expr.arguments.get(arg_index) else {
            return false;
        };

        if argument.span() != call_arg.span() {
            return false;
        }

        if !is_recursive_call(call_expr, function_symbol_id, ctx) {
            return false;
        }
    }

    true
}

fn is_property_only_used_in_recursion_jsx(
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
        let Some(may_jsx_expr_container) = ctx.nodes().parent_node(reference.node_id()) else {
            return false;
        };
        let AstKind::JSXExpressionContainer(_) = may_jsx_expr_container.kind() else {
            // In this case, we simply ignore the references inside JSXExpressionContainer that are not single-node expression.
            //   e.g. <Increment count={count+1} />
            //
            // To support this case, we need to check whether expression contains side-effect like ++val
            return false;
        };

        let Some(attr) = ctx.nodes().ancestors(may_jsx_expr_container.id()).find_map(|node| {
            if let AstKind::JSXAttributeItem(attr) = node.kind() {
                Some(attr)
            } else {
                None
            }
        }) else {
            return false;
        };

        let JSXAttributeItem::Attribute(jsx_attr_name) = attr else {
            return false;
        };
        let Some(attr_name) = jsx_attr_name.name.as_identifier() else {
            return false;
        };
        if attr_name.name != property_name {
            return false;
        }

        let Some(opening_element) =
            ctx.nodes().ancestor_ids(reference.node_id()).find_map(|node| {
                if let AstKind::JSXOpeningElement(elem) = ctx.nodes().get_node(node).kind() {
                    Some(elem)
                } else {
                    None
                }
            })
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
    if let Expression::Identifier(identifier) = &call_expr.callee {
        if let Some(symbol_id) = ctx.symbols().get_reference(identifier.reference_id()).symbol_id()
        {
            return symbol_id == function_symbol_id;
        }
    }
    false
}

fn is_function_maybe_reassigned<'a>(
    function_id: &'a BindingIdentifier,
    ctx: &'a LintContext<'_>,
) -> bool {
    ctx.semantic().symbol_references(function_id.symbol_id()).any(|reference| {
        matches!(
            ctx.nodes().parent_kind(reference.node_id()),
            Some(AstKind::SimpleAssignmentTarget(_))
        )
    })
}

fn get_jsx_element_symbol_id<'a>(
    node: &'a JSXElementName<'a>,
    ctx: &'a LintContext<'_>,
) -> Option<SymbolId> {
    let node = match node {
        JSXElementName::IdentifierReference(ident) => Some(ident.as_ref()),
        JSXElementName::MemberExpression(expr) => expr.get_identifier(),
        JSXElementName::Identifier(_)
        | JSXElementName::NamespacedName(_)
        | JSXElementName::ThisExpression(_) => None,
    }?;

    ctx.symbols().get_reference(node.reference_id()).symbol_id()
}

enum Direction {
    Forward,
    Backward,
}

// Skips whitespace and commas in a given direction and
// returns the next character if found.
#[allow(clippy::cast_possible_truncation)]
fn skip_to_next_char(s: &str, start: u32, direction: &Direction) -> Option<u32> {
    // span is a half-open interval: [start, end)
    // so we should return in that way.
    let start = start as usize;
    match direction {
        Direction::Forward => s
            .char_indices()
            .skip(start)
            .find(|&(_, c)| !c.is_whitespace() && c != ',')
            .map(|(i, _)| i as u32),

        Direction::Backward => s
            .char_indices()
            .rev()
            .skip(s.len() - start)
            .take_while(|&(_, c)| c.is_whitespace() || c == ',')
            .map(|(i, _)| i as u32)
            .last(),
    }
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
        "function foo() {}
        declare function foo() {}",
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
        // Unused Argument in Recursion
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
            test(foo, );
            function test(arg0, ) {
                return test("", );
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
    ];

    Tester::new(OnlyUsedInRecursion::NAME, OnlyUsedInRecursion::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
