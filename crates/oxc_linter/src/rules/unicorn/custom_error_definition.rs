use oxc_ast::{
    AstKind,
    ast::{
        AssignmentTarget, Class, ClassElement, Expression, MethodDefinitionKind,
        PropertyDefinition, Statement,
    },
    match_member_expression,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, ast_util, context::LintContext, rule::Rule};

fn invalid_class_name_diagnostic(span: Span, expected: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Invalid class name, use `{expected}`.")).with_label(span)
}

fn missing_super_call_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing call to `super()` in constructor.").with_label(span)
}

fn invalid_name_property_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("The `name` property should be set to `{name}`.")).with_label(span)
}

fn pass_message_to_super_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Pass the error message to `super()` instead of setting `this.message`.")
        .with_label(span)
}

fn invalid_export_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Exported error name should match error class").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct CustomErrorDefinition;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the only valid way of Error subclassing. It works with any super class that ends in Error.
    ///
    /// ### Why is this bad?
    ///
    /// Incorrectly defined custom errors can lead to unexpected behavior when
    /// catching and identifying errors. Missing `super()` calls, wrong `name`
    /// property values, or non-standard class names make error handling unreliable.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// class CustomError extends Error {
    /// 	constructor(message) {
    /// 		super(message);
    /// 		// The `this.message` assignment is useless as it's already set via the `super()` call.
    /// 		this.message = message;
    /// 		this.name = 'CustomError';
    /// 	}
    /// }
    ///
    /// class CustomError extends Error {
    /// 	constructor(message) {
    /// 		super();
    /// 		// Pass the error message to `super()` instead of setting `this.message`.
    /// 		this.message = message;
    /// 		this.name = 'CustomError';
    /// 	}
    /// }
    ///
    /// class CustomError extends Error {
    /// 	constructor(message) {
    /// 		super(message);
    /// 		// No `name` property set. The name property is needed so the
    /// 		// error shows up as `[CustomError: foo]` and not `[Error: foo]`.
    /// 	}
    /// }
    ///
    /// class CustomError extends Error {
    /// 	constructor(message) {
    /// 		super(message);
    /// 		// Use a string literal to set the `name` property as it will not change after minifying.
    /// 		this.name = this.constructor.name;
    /// 	}
    /// }
    ///
    /// class CustomError extends Error {
    /// 	constructor(message) {
    /// 		super(message);
    /// 		// The `name` property should be set to the class name.
    /// 		this.name = 'MyError';
    /// 	}
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// class CustomError extends Error {
    /// 	constructor(message) {
    /// 		super(message);
    /// 		this.name = 'CustomError';
    /// 	}
    /// }
    ///
    /// class CustomError extends Error {
    ///	    constructor() {
    ///		    super('My custom error');
    ///		    this.name = 'CustomError';
    ///	    }
    /// }
    ///
    /// class CustomError extends TypeError {
    ///	    constructor() {
    ///		    super();
    ///		    this.name = 'CustomError';
    ///     }
    /// }
    ///
    /// class CustomError extends Error {
    /// 	name = 'CustomError';
    /// }
    /// ```
    CustomErrorDefinition,
    unicorn,
    style,
    pending,
    version = "1.57.0",
);

impl Rule for CustomErrorDefinition {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::Class(class) = node.kind() {
            check_class(class, node, ctx);
        }
    }
}

fn check_export_assignment(
    class: &Class,
    exports_property_span: Span,
    exports_property_name: &str,
    ctx: &LintContext,
) {
    if !has_valid_super_class(class) {
        return;
    }

    let Some(id) = &class.id else {
        return;
    };

    let error_name = id.name.as_str();
    if exports_property_name != error_name {
        ctx.diagnostic(invalid_export_diagnostic(exports_property_span));
    }
}

fn check_class<'a>(class: &Class<'a>, node: &AstNode<'a>, ctx: &LintContext<'a>) {
    if !has_valid_super_class(class) {
        return;
    }

    if let Some((exports_property_span, exports_property_name)) =
        get_export_assignment_info(node, ctx)
    {
        check_export_assignment(class, exports_property_span, exports_property_name, ctx);
    }

    let Some(id) = &class.id else {
        return;
    };

    let name = id.name.as_str();
    let expected_class_name = get_class_name(name);

    if name != expected_class_name {
        ctx.diagnostic(invalid_class_name_diagnostic(id.span, &expected_class_name));
    }

    let constructor = class.body.body.iter().find_map(|el| match el {
        ClassElement::MethodDefinition(method)
            if method.kind == MethodDefinitionKind::Constructor && method.value.body.is_some() =>
        {
            Some(method)
        }
        _ => None,
    });
    let has_constructor_signature = class.body.body.iter().any(|el| {
        matches!(el, ClassElement::MethodDefinition(method) if method.kind == MethodDefinitionKind::Constructor)
    });

    let name_property = class.body.body.iter().find_map(|el| {
        if let ClassElement::PropertyDefinition(prop) = el
            && is_name_property_definition(prop)
        {
            Some(prop.as_ref())
        } else {
            None
        }
    });

    let Some(constructor_method) = constructor else {
        if has_constructor_signature {
            return;
        }

        if is_valid_name_property(name_property, name) {
            return;
        }

        if let Some(prop) = name_property {
            let span = prop.value.as_ref().map_or(prop.span, oxc_span::GetSpan::span);
            ctx.diagnostic(invalid_name_property_diagnostic(span, name));
        } else {
            ctx.diagnostic(invalid_name_property_diagnostic(class.span, name));
        }
        return;
    };

    let Some(body) = &constructor_method.value.body else {
        return;
    };

    let statements = &body.statements;

    let has_super = statements.iter().any(|s| is_super_call(s));

    let message_assignment_idx = statements.iter().position(|s| is_this_assignment(s, "message"));

    if !has_super {
        ctx.diagnostic(missing_super_call_diagnostic(body.span));
    } else if message_assignment_idx.is_some() {
        let Some(super_stmt) = statements.iter().find(|s| is_super_call(s)) else {
            return;
        };

        let Statement::ExpressionStatement(super_expr_stmt) = super_stmt else {
            return;
        };

        ctx.diagnostic(pass_message_to_super_diagnostic(super_expr_stmt.span));
    }

    let name_assignment = statements.iter().find(|s| is_this_assignment(s, "name"));

    let invalid_name_span = if let Some(name_stmt) = name_assignment {
        let Statement::ExpressionStatement(expr_stmt) = name_stmt else {
            return;
        };
        let Expression::AssignmentExpression(assign) = &expr_stmt.expression else {
            return;
        };

        (!is_expected_string_literal(&assign.right, name)).then_some(assign.right.span())
    } else if let Some(prop) = name_property {
        let span = prop.value.as_ref().map_or(prop.span, oxc_span::GetSpan::span);
        match &prop.value {
            Some(value) => (!is_expected_string_literal(value, name)).then_some(span),
            None => Some(span),
        }
    } else {
        Some(body.span)
    };

    if let Some(span) = invalid_name_span {
        ctx.diagnostic(invalid_name_property_diagnostic(span, name));
    }
}

fn get_export_assignment_info<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> Option<(Span, &'a str)> {
    let AstKind::AssignmentExpression(assign) =
        ast_util::iter_outer_expressions(ctx.nodes(), node.id()).next()?
    else {
        return None;
    };

    let AssignmentTarget::StaticMemberExpression(member) = &assign.left else {
        return None;
    };

    let Expression::Identifier(obj_ident) = &member.object else {
        return None;
    };

    (obj_ident.name.as_str() == "exports")
        .then_some((member.property.span, member.property.name.as_str()))
}

fn get_class_name(name: &str) -> String {
    let uppered = upper_first(name);

    if let Some(stripped) = strip_suffix_case_insensitive(&uppered, "error") {
        format!("{stripped}Error")
    } else {
        format!("{uppered}Error")
    }
}

fn strip_suffix_case_insensitive<'a>(value: &'a str, suffix: &str) -> Option<&'a str> {
    let start = value.len().checked_sub(suffix.len())?;
    value[start..].eq_ignore_ascii_case(suffix).then_some(&value[..start])
}

fn is_name_property_definition(prop: &PropertyDefinition) -> bool {
    !prop.r#static && !prop.computed && prop.key.is_specific_static_name("name")
}

fn is_valid_super_class_name(name: &str) -> bool {
    let Some(prefix) = name.strip_suffix("Error") else {
        return false;
    };

    let bytes = prefix.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if !bytes[index].is_ascii_uppercase() {
            return false;
        }

        index += 1;

        while index < bytes.len()
            && (bytes[index].is_ascii_lowercase() || bytes[index].is_ascii_digit())
        {
            index += 1;
        }
    }

    true
}

fn has_valid_super_class(class: &Class) -> bool {
    let Some(super_class) = &class.super_class else {
        return false;
    };
    let name = match super_class.get_inner_expression() {
        Expression::Identifier(ident) => Some(ident.name.as_str()),
        e @ match_member_expression!(Expression) => e.to_member_expression().static_property_name(),
        _ => None,
    };
    name.is_some_and(is_valid_super_class_name)
}

fn upper_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

fn is_super_call(stmt: &Statement) -> bool {
    let Statement::ExpressionStatement(expr_stmt) = stmt else { return false };
    let Expression::CallExpression(call) = &expr_stmt.expression else { return false };
    matches!(&call.callee, Expression::Super(_))
}

fn is_this_assignment(stmt: &Statement, prop_name: &str) -> bool {
    let Statement::ExpressionStatement(expr_stmt) = stmt else { return false };
    let Expression::AssignmentExpression(assign) = &expr_stmt.expression else { return false };

    if let AssignmentTarget::StaticMemberExpression(member) = &assign.left {
        matches!(&member.object, Expression::ThisExpression(_))
            && member.property.name.as_str() == prop_name
    } else {
        false
    }
}

fn is_valid_name_property(name_property: Option<&PropertyDefinition>, class_name: &str) -> bool {
    if let Some(prop) = name_property
        && let Some(Expression::StringLiteral(lit)) = &prop.value
    {
        return lit.value.as_str() == class_name;
    }

    false
}

fn is_expected_string_literal(expr: &Expression, expected: &str) -> bool {
    matches!(expr, Expression::StringLiteral(lit) if lit.value.as_str() == expected)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "class Foo { }",
        "class Foo extends Bar { }",
        "class Foo extends Bar() { }",
        "const Foo = class { }",
        r"const FooError = class extends Error {
            constructor(message) {
                super(message);
                this.name = 'FooError';
            }
        }",
        r"class FooError extends Http.ProtocolError {
            constructor(message) {
                super(message);
                this.name = 'FooError';
            }
        }",
        r"class FooError extends Error {
            constructor(message) {
                super(message);
                this.name = 'FooError';
            }
        }",
        r"class FooError extends Error {
            constructor() {
                super('My super awesome Foo Error');
                this.name = 'FooError';
            }
        }",
        r"class FooError extends TypeError {
            constructor() {
                super();
                this.name = 'FooError';
            }
        }",
        r"class FooError extends Error {
            name = 'FooError';
        }",
        r"export class FooError extends TypeError {
            constructor() {
                super();
                this.name = 'FooError';
            }
        };",
        r"export default class FooError extends TypeError {
            constructor() {
                super();
                this.name = 'FooError';
            }
        };",
        r"module.exports = class FooError extends TypeError {
            constructor() {
                super();
                this.name = 'FooError';
            }
        };",
        r"exports.FooError = class FooError extends TypeError {
            constructor() {
                super();
                this.name = 'FooError';
            }
        };",
        r"exports.FooError = (class FooError extends TypeError {
            constructor() {
                super();
                this.name = 'FooError';
            }
        });",
        r"exports.FooError = class extends Error {
            constructor(error) {
                super(error);
            }
        };",
        r"exports.fooError = class extends Error {
            constructor(error) {
                super(error);
                this.name = 'fooError';
            }
        };",
        "exports.whatever = class Whatever {};",
        r"class FooError extends Error {
            constructor(error) {
                super(error);
                this.name = 'FooError';
            }
        };
        exports.fooError = FooError;",
        r"class FooError extends Error {
            constructor() {
                super();
                this.name = 'FooError';
                someThingNotThis.message = 'My custom message';
            }
        }",
        r"export class ValidationError extends Error {
            name = 'ValidationError';
            constructor(message) {
                super(message);
            }
        }",
        r"class CustomError extends Error {
            constructor(type: string, text: string, reply?: any);
        }",
    ];

    let fail = vec![
        r"class FooError extends Error {}",
        r"class FooError extends Error {
            name = 'BadError';
        }",
        r"class FooError extends Error {
            static name = 'FooError';
        }",
        r"class fooError extends Error {
            constructor(message) {
                super(message);
                this.name = 'FooError';
            }
        }",
        r"class fooError extends Error {
            constructor(message) {
                super(message);
                this.name = 'fooError';
            }
        }",
        r"class Foo extends Error {
            constructor(message) {
                super(message);
                this.name = 'Foo';
            }
        }",
        r"class FooERROR extends Error {
            constructor(message) {
                super(message);
                this.name = 'FooERROR';
            }
        }",
        r"class fooerror extends Error {
            constructor(message) {
                super(message);
                this.name = 'fooerror';
            }
        }",
        r"class FooError extends Error {
            constructor() { }
        }",
        r"class FooError extends Error {
            constructor() {
                super();
                this.message = 'My custom message';
            }
        }",
        r"class FooError extends Error {
            constructor() {
                super();
            }
        }",
        r"class FooError extends Error {
            constructor(message: string);
            constructor(message: string) {
                this.name = 'FooError';
            }
        }",
        r"class FooError extends Error {
            constructor() {
                super('My awesome Foo Error');
                this.name = this.constructor.name;
            }
        }",
        r"class FooError extends Error {
            constructor(message) {
                super(message);
                this.message = message;
                this.name = 'FooError';
            }
        }",
        r"class FooError extends Error {
            constructor(message) {
                super();
                this.message = message;
                this.name = 'FooError';
            }
        }",
        r"class FooError extends Error {
            constructor(message) {
                super();
                this.message = message;
            }
        }",
        r"class FooError extends Error {
            constructor(message) {
                super();
                this.name = 'FooError';
                this.message = message;
            }
        }",
        r"class FooError extends Http.ProtocolError {
            constructor() {
                super();
                this.name = 'foo';
            }
        }",
        r"module.exports = class FooError extends TypeError {
            constructor() {
                super();
                this.name = 'foo';
            }
        };",
        r"exports.fooError = class FooError extends Error {
            constructor(error) {
                super(error);
                this.name = 'FooError';
            }
        };",
        r"exports.fooError = (class FooError extends Error {
            constructor(error) {
                super(error);
                this.name = 'FooError';
            }
        });",
        r"exports.FooError = class FooError extends TypeError {
            constructor() {
                super();
                this.name = 'foo';
            }
        };",
        r"export class FooError extends TypeError {
            constructor() {
                super();
                this.name = 'foo';
            }
        };",
        r"export default class FooError extends TypeError {
            constructor() {
                super();
                this.name = 'foo';
            }
        };",
        r"class AbortError extends Error {
            constructor(message) {
                if (message instanceof Error) {
                    this.originalError = message;
                    message = message.message;
                }

                super();
                this.name = 'AbortError';
                this.message = message;
            }
        }",
        r"class FooError extends Error {
            constructor() {
                super();
                this.message = foo.error;
                this.name = 'FooError';
            }
        }",
        r"export class ValidationError extends Error {
            name = 'FOO';
            constructor(message) {
                super(message);
            }
        }",
        r"const name = 'computed-name';
        class FooError extends Error {
            [name] = 'FooError';
            constructor(message) {
                super(message);
            }
        }",
        r"export class ValidationError extends Error {
            'name': SomeType;
            constructor(message) {
                super(message);
            }
        }",
        r"class FooError extends Error {
            name: string;
        }",
    ];

    Tester::new(CustomErrorDefinition::NAME, CustomErrorDefinition::PLUGIN, pass, fail)
        .test_and_snapshot();
}
