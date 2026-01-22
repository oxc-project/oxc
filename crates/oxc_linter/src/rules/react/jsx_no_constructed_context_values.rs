use oxc_ast::{
    AstKind,
    ast::{
        Expression, IdentifierReference, JSXAttributeItem, JSXAttributeName, JSXAttributeValue,
        JSXOpeningElement,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn jsx_no_constructed_context_values(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "The Context value prop should not be constructed. Move it out of the render pass, use a constant value, or wrap with useMemo.",
    )
    .with_help("Wrap the value prop in useMemo() or useCallback() to prevent unnecessary rerenders. Alternatively, move the value outside the render function if it doesn't depend on props or state.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct JsxNoConstructedContextValues;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows JSX context provider values from taking values that will cause needless rerenders.
    ///
    /// ### Why is this bad?
    ///
    /// React Context and all its child nodes and Consumers are rerendered whenever the value prop
    /// changes. Because each JavaScript object carries its own identity, things like object
    /// expressions (`{foo: 'bar'}`) or function expressions get a new identity on every render.
    /// This makes the context think it has gotten a new object and can cause needless rerenders
    /// and unintended consequences.
    ///
    /// This can be a large performance hit because not only will it cause the context providers
    /// and consumers to rerender with all the elements in its subtree, the processing for the
    /// tree scan React does to render the provider and find consumers is also wasted.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// return (
    ///   <SomeContext.Provider value={{foo: 'bar'}}>
    ///     ...
    ///   </SomeContext.Provider>
    /// )
    /// ```
    ///
    /// ```jsx
    /// function Component() {
    ///   function foo() {}
    ///   return <MyContext.Provider value={foo} />
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// const foo = useMemo(() => ({foo: 'bar'}), []);
    /// return (
    ///   <SomeContext.Provider value={foo}>
    ///     ...
    ///   </SomeContext.Provider>
    /// )
    /// ```
    ///
    /// ```jsx
    /// const MyContext = createContext();
    /// const Component = () => <MyContext.Provider value="Some string" />
    /// ```
    JsxNoConstructedContextValues,
    react,
    correctness,
);

impl Rule for JsxNoConstructedContextValues {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_opening_elem) = node.kind() else {
            return;
        };

        // Check if this is a Context.Provider component or a createContext() context
        if !is_context_provider(jsx_opening_elem, ctx) {
            return;
        }

        // Check if we're inside a function component (not a top-level render call)
        if !is_inside_component(node, ctx) {
            return;
        }

        // Find the "value" prop
        for attr in &jsx_opening_elem.attributes {
            let JSXAttributeItem::Attribute(attr) = attr else {
                continue;
            };

            let JSXAttributeName::Identifier(attr_name) = &attr.name else {
                continue;
            };

            if attr_name.name != "value" {
                continue;
            }

            if let Some(attr_value) = &attr.value
                && is_constructed_value(attr_value, ctx)
            {
                ctx.diagnostic(jsx_no_constructed_context_values(attr.span()));
            }
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

fn is_context_provider(jsx_opening_elem: &JSXOpeningElement, ctx: &LintContext<'_>) -> bool {
    match &jsx_opening_elem.name {
        oxc_ast::ast::JSXElementName::MemberExpression(member_expr) => {
            &member_expr.property.name == "Provider"
        }
        name @ (oxc_ast::ast::JSXElementName::Identifier(_)
        | oxc_ast::ast::JSXElementName::IdentifierReference(_)) => {
            // Check if this identifier refers to a context created with createContext()
            name.get_identifier().is_some_and(|ident| is_react_context(ident, ctx))
        }
        _ => false,
    }
}

/// Checks if an identifier refers to a React context created with createContext()
fn is_react_context(ident: &IdentifierReference, ctx: &LintContext) -> bool {
    let reference = ctx.scoping().get_reference(ident.reference_id());
    let Some(symbol_id) = reference.symbol_id() else {
        return false;
    };

    // Get the symbol's declaration
    let declaration_node_id = ctx.scoping().symbol_declaration(symbol_id);
    let declaration_node = ctx.nodes().get_node(declaration_node_id);

    // Check if it's a variable declarator with createContext() initializer
    let AstKind::VariableDeclarator(decl) = declaration_node.kind() else {
        return false;
    };

    let Some(init) = &decl.init else {
        return false;
    };

    is_create_context_call(init)
}

/// Checks if an expression is a call to createContext() or React.createContext()
fn is_create_context_call(expr: &Expression) -> bool {
    let Expression::CallExpression(call_expr) = expr else {
        return false;
    };

    match &call_expr.callee {
        // Direct call: createContext()
        Expression::Identifier(ident) => ident.name == "createContext",
        // Member call: React.createContext()
        Expression::StaticMemberExpression(member) => {
            if let Expression::Identifier(obj) = &member.object {
                obj.name == "React" && member.property.name == "createContext"
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Checks if the JSX element is inside a React component (function or class).
/// Top-level render calls like `ReactDOM.createRoot(...).render()` should not trigger the rule.
fn is_inside_component(node: &AstNode, ctx: &LintContext) -> bool {
    for parent_id in ctx.nodes().ancestor_ids(node.id()).skip(1) {
        let parent = ctx.nodes().get_node(parent_id);
        match parent.kind() {
            // Arrow function, regular function, or class method - likely a component
            AstKind::ArrowFunctionExpression(_)
            | AstKind::Function(_)
            | AstKind::MethodDefinition(_) => {
                return true;
            }
            _ => {}
        }
    }
    false
}

fn is_constructed_value(attr_value: &JSXAttributeValue, ctx: &LintContext) -> bool {
    match attr_value {
        JSXAttributeValue::ExpressionContainer(container) => {
            if let Some(expr) = container.expression.as_expression() {
                is_constructed_expression(expr, ctx)
            } else {
                false
            }
        }
        _ => false,
    }
}

fn is_constructed_expression(expr: &Expression, ctx: &LintContext) -> bool {
    match expr {
        Expression::ObjectExpression(_)
        | Expression::ArrayExpression(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::JSXElement(_)
        | Expression::JSXFragment(_)
        | Expression::ClassExpression(_)
        | Expression::NewExpression(_)
        | Expression::UpdateExpression(_)
        | Expression::AssignmentExpression(_)
        | Expression::TaggedTemplateExpression(_)
        | Expression::AwaitExpression(_)
        | Expression::YieldExpression(_)
        | Expression::ImportExpression(_) => true,

        Expression::CallExpression(call_expr) => {
            if let Expression::Identifier(ident) = &call_expr.callee {
                let name = ident.name.as_str();
                if name == "useMemo" || name == "useCallback" {
                    return false;
                }
            }
            true
        }

        Expression::TemplateLiteral(template) => !template.expressions.is_empty(),

        Expression::BinaryExpression(bin_expr) => {
            is_constructed_expression(&bin_expr.left, ctx)
                || is_constructed_expression(&bin_expr.right, ctx)
        }

        Expression::LogicalExpression(log_expr) => {
            is_constructed_expression(&log_expr.left, ctx)
                || is_constructed_expression(&log_expr.right, ctx)
        }

        Expression::ConditionalExpression(cond_expr) => {
            is_constructed_expression(&cond_expr.consequent, ctx)
                || is_constructed_expression(&cond_expr.alternate, ctx)
        }

        Expression::UnaryExpression(unary_expr) => {
            is_constructed_expression(&unary_expr.argument, ctx)
        }

        Expression::SequenceExpression(seq_expr) => {
            seq_expr.expressions.iter().any(|e| is_constructed_expression(e, ctx))
        }

        Expression::ParenthesizedExpression(paren_expr) => {
            is_constructed_expression(&paren_expr.expression, ctx)
        }

        Expression::Identifier(ident) => is_identifier_a_constructed_value(ident, ctx),

        Expression::ChainExpression(chain_expr) => match &chain_expr.expression {
            oxc_ast::ast::ChainElement::CallExpression(call) => {
                if let Expression::Identifier(ident) = &call.callee {
                    let name = ident.name.as_str();
                    if name == "useMemo" || name == "useCallback" {
                        return false;
                    }
                }
                true
            }
            _ => false,
        },

        Expression::TSAsExpression(ts_as) => is_constructed_expression(&ts_as.expression, ctx),
        Expression::TSSatisfiesExpression(ts_sat) => {
            is_constructed_expression(&ts_sat.expression, ctx)
        }
        Expression::TSTypeAssertion(ts_type) => is_constructed_expression(&ts_type.expression, ctx),
        Expression::TSNonNullExpression(ts_non_null) => {
            is_constructed_expression(&ts_non_null.expression, ctx)
        }
        Expression::TSInstantiationExpression(ts_inst) => {
            is_constructed_expression(&ts_inst.expression, ctx)
        }

        _ => false,
    }
}

/// Checks if an identifier refers to a constructed value defined within the current component.
/// Constructed values are recreated on every render, causing unnecessary re-renders.
fn is_identifier_a_constructed_value(ident: &IdentifierReference, ctx: &LintContext) -> bool {
    let reference = ctx.scoping().get_reference(ident.reference_id());
    let Some(symbol_id) = reference.symbol_id() else {
        return false;
    };

    // Check if the symbol is declared at module/top level (not inside a function)
    // Top-level declarations are stable and don't get recreated on each render
    let symbol_scope_id = ctx.scoping().symbol_scope_id(symbol_id);
    if ctx.scoping().scope_flags(symbol_scope_id).is_top() {
        return false;
    }

    let declaration_node_id = ctx.scoping().symbol_declaration(symbol_id);
    let declaration_node = ctx.nodes().get_node(declaration_node_id);

    match declaration_node.kind() {
        // Function declarations and class declarations
        AstKind::Function(_) | AstKind::Class(_) => true,
        // Variable declarations - check the initializer for constructed values
        AstKind::VariableDeclarator(decl) => {
            if let Some(init) = &decl.init {
                is_construction_expression(init.without_parentheses(), ctx)
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Checks if an expression is a construction that creates a new object identity.
/// This is used for checking variable initializers.
fn is_construction_expression(expr: &Expression, ctx: &LintContext) -> bool {
    match expr {
        // Direct constructions and regex literals create new objects
        Expression::ObjectExpression(_)
        | Expression::ArrayExpression(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::ClassExpression(_)
        | Expression::NewExpression(_)
        | Expression::JSXElement(_)
        | Expression::JSXFragment(_)
        | Expression::RegExpLiteral(_) => true,

        // Conditional/logical expressions - check if any branch is a construction
        Expression::ConditionalExpression(cond_expr) => {
            is_construction_expression(&cond_expr.consequent, ctx)
                || is_construction_expression(&cond_expr.alternate, ctx)
        }
        Expression::LogicalExpression(log_expr) => {
            is_construction_expression(&log_expr.left, ctx)
                || is_construction_expression(&log_expr.right, ctx)
        }

        // Assignment expressions - check the right side
        Expression::AssignmentExpression(assign_expr) => {
            is_construction_expression(&assign_expr.right, ctx)
        }

        // Follow through parentheses and type assertions
        Expression::ParenthesizedExpression(paren) => {
            is_construction_expression(&paren.expression, ctx)
        }
        Expression::TSAsExpression(ts_as) => is_construction_expression(&ts_as.expression, ctx),
        Expression::TSSatisfiesExpression(ts_sat) => {
            is_construction_expression(&ts_sat.expression, ctx)
        }
        Expression::TSTypeAssertion(ts_type) => {
            is_construction_expression(&ts_type.expression, ctx)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            is_construction_expression(&ts_non_null.expression, ctx)
        }

        // For identifiers, recursively check if they refer to constructed values
        Expression::Identifier(ident) => is_identifier_a_constructed_value(ident, ctx),

        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Original eslint-plugin-react tests
        "const Component = () => <Context.Provider value={props}></Context.Provider>",
        "const Component = () => <Context.Provider value={100}></Context.Provider>",
        r#"const Component = () => <Context.Provider value="Some string"></Context.Provider>"#,
        "function Component() { const foo = useMemo(() => { return {} }, []); return (<Context.Provider value={foo}></Context.Provider>)}",
        "
            function Component({oneProp, twoProp, redProp, blueProp,}) {
              return (
                <NewContext.Provider value={twoProp}></NewContext.Provider>
              );
            }
        ",
        "
            function Foo(section) {
              const foo = section.section_components?.edges;
              return (
                <Context.Provider value={foo}></Context.Provider>
              )
            }
        ",
        "
            import foo from 'foo';
            function innerContext() {
              return (
                <Context.Provider value={foo.something}></Context.Provider>
              )
            }
        ",
        "
            // Passes because the lint rule doesn't handle JSX spread attributes
            function innerContext() {
              const foo = {value: 'something'}
              return (
                <Context.Provider {...foo}></Context.Provider>
              )
            }
        ",
        "
            // Passes because the lint rule doesn't handle JSX spread attributes
            function innerContext() {
              const foo = useMemo(() => {
                return bar;
              })
              return (
                <Context.Provider value={foo}></Context.Provider>
              )
            }
        ",
        "
            // Passes because we can't statically check if it's using the default value
            function Component({ a = {} }) {
              return (<Context.Provider value={a}></Context.Provider>);
            }
        ",
        "
            import React from 'react';
            import MyContext from './MyContext';

            const value = '';

            function ContextProvider(props) {
                return (
                    <MyContext.Provider value={value as any}>
                        {props.children}
                    </MyContext.Provider>
                )
            }
        ",
        "
            import React from 'react';
            import BooleanContext from './BooleanContext';

            function ContextProvider(props) {
                return (
                    <BooleanContext.Provider value>
                        {props.children}
                    </BooleanContext.Provider>
                )
            }
        ",
        "
            const root = ReactDOM.createRoot(document.getElementById('root'));
            root.render(
              <AppContext.Provider value={{}}>
                <AppView />
              </AppContext.Provider>
            );
        ",
        "
            // Passes because the context is not a provider
            function Component() {
              return <MyContext.Consumer value={{ foo: 'bar' }} />;
            }
        ",
        // Tests for createContext() - contexts used directly without .Provider
        // These should pass because they're not inside components or don't use constructed values
        "
            import React from 'react';

            const MyContext = React.createContext();
            const Component = () => <MyContext value={props}></MyContext>;
        ",
        "
            import React from 'react';

            const MyContext = React.createContext();
            const Component = () => <MyContext value={100}></MyContext>;
        ",
        "
            const SomeContext = createContext();
            const Component = () => <SomeContext value=\"Some string\"></SomeContext>;
        ",
        "
            // Passes because MyContext is not a variable declarator
            function Component({ MyContext }) {
              return <MyContext value={{ foo: \"bar\" }} />;
            }
        ",
    ];

    let fail = vec![
        // Invalid because object construction creates a new identity
        "function Component() { const foo = {}; return (<Context.Provider value={foo}></Context.Provider>) }",
        // Invalid because array construction creates a new identity
        "function Component() { const foo = []; return (<Context.Provider value={foo}></Context.Provider>) }",
        // Invalid because arrow Function creates a new identity
        "function Component() { const foo = () => {}; return (<Context.Provider value={foo}></Context.Provider>)}",
        // Invalid because function expression creates a new identity
        "function Component() { const foo = function bar(){}; return (<Context.Provider value={foo}></Context.Provider>)}",
        // Invalid because class expression creates a new identity
        "function Component() { const foo = class SomeClass{}; return (<Context.Provider value={foo}></Context.Provider>)}",
        // Invalid because new expression creates a new identity
        "function Component() { const foo = new SomeClass(); return (<Context.Provider value={foo}></Context.Provider>)}",
        // Invalid because function declaration creates a new identity
        "function Component() { function foo() {}; return (<Context.Provider value={foo}></Context.Provider>)}",
        // Invalid because the object value of the ternary will create a new identity
        r#"function Component() { const foo = true ? {} : "fine"; return (<Context.Provider value={foo}></Context.Provider>)}"#,
        // Invalid because the object value of the logical OR will create a new identity
        "function Component() { const foo = bar || {}; return (<Context.Provider value={foo}></Context.Provider>)}",
        // Invalid because the object value of the logical AND will create a new identity
        "function Component() { const foo = bar && {}; return (<Context.Provider value={foo}></Context.Provider>)}",
        // Invalid because the object value of the nested ternary will create a new identity
        "function Component() { const foo = bar ? baz ? {} : null : null; return (<Context.Provider value={foo}></Context.Provider>)}",
        // Invalid because the object value will create a new identity (let)
        "function Component() { let foo = {}; return (<Context.Provider value={foo}></Context.Provider>) }",
        // Invalid because the object value will create a new identity (var)
        "function Component() { var foo = {}; return (<Context.Provider value={foo}></Context.Provider>)}",
        // Variable reassignment - currently reports the object identity
        "
            function Component() {
              let a = {};
              a = 10;
              return (<Context.Provider value={a}></Context.Provider>);
            }
        ",
        // Invalid variable reassignment from parameter because bar is an object identity
        "
            function Component() {
              const foo = {};
              const bar = foo;
              return (<Context.Provider value={bar}></Context.Provider>);
            }
        ",
        // Invalid because the object expression possibly returned from the ternary will create a new identity
        "
            function Component(foo) {
              let bar = true ? foo : {};
              return (<Context.Provider value={bar}></Context.Provider>);
            }
        ",
        // Invalid because inline object construction will create a new identity
        r#"function Component() { return (<Context.Provider value={{foo: "bar"}}></Context.Provider>);}"#,
        // Invalid because Wrapper returns JSX which has a new identity
        "function Component() { const Wrapper = (<SomeComp />); return (<Context.Provider value={Wrapper}></Context.Provider>);}",
        // Invalid because RegEx returns a new object which has a new identity
        "function Component() { const someRegex = /HelloWorld/; return (<Context.Provider value={someRegex}></Context.Provider>);}",
        // Invalid because the right hand side of the assignment expression contains a function
        "
            function Component() {
              let foo = null;
              let bar = x = () => {};
              return (<Context.Provider value={bar}></Context.Provider>);
            }
        ",
        // Invalid because function declaration creates a new identity - with createContext
        "
            import React from 'react';

            const Context = React.createContext();
            function Component() {
              function foo() {};
              return (<Context value={foo}></Context>)
            }
        ",
        // Invalid because the object value will create a new identity - with createContext
        "
            const MyContext = createContext();
            function Component() { const foo = {}; return (<MyContext value={foo}></MyContext>) }
        ",
        // Invalid because inline object construction will create a new identity - with createContext
        "
            const MyContext = createContext();
            function Component() { return (<MyContext value={{foo: \"bar\"}}></MyContext>); }
        ",
    ];

    Tester::new(
        JsxNoConstructedContextValues::NAME,
        JsxNoConstructedContextValues::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
