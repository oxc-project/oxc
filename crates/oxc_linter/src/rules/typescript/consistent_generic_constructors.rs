use oxc_ast::{
    AstKind,
    ast::{
        BindingPattern, Expression, NewExpression, TSType, TSTypeAnnotation, TSTypeName,
        TSTypeParameterInstantiation, TSTypeReference,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AstNode, context::LintContext, fixer::RuleFixer, rule::Rule};

fn consistent_generic_constructors_diagnostic_prefer_annotation(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "The generic type arguments should be specified as part of the type annotation.",
    )
    .with_help("Move the generic type to the type annotation")
    .with_label(span)
}

fn consistent_generic_constructors_diagnostic_prefer_constructor(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "The generic type arguments should be specified as part of the constructor type arguments.",
    )
    .with_help("Move the type annotation to the constructor")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ConsistentGenericConstructors(Box<ConsistentGenericConstructorsConfig>);

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ConsistentGenericConstructorsConfig {
    /// Specifies where the generic type should be specified.
    ///
    /// Possible values:
    /// - `"constructor"` (default): Type arguments that only appear on the type annotation are disallowed.
    /// - `"type-annotation"`: Type arguments that only appear on the constructor are disallowed.
    option: PreferGenericType,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum PreferGenericType {
    /// Type arguments that only appear on the type annotation are disallowed.
    #[default]
    Constructor,
    /// Type arguments that only appear on the constructor are disallowed.
    TypeAnnotation,
}

impl TryFrom<&str> for PreferGenericType {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "constructor" => Ok(Self::Constructor),
            "type-annotation" => Ok(Self::TypeAnnotation),
            _ => Err("Invalid value"),
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// When constructing a generic class, you can specify the type arguments on either the left-hand side (as a type annotation) or the right-hand side (as part of the constructor call).
    ///
    /// This rule enforces consistency in the way generic constructors are used.
    ///
    /// ### Why is this bad?
    ///
    /// Inconsistent usage of generic constructors can make the code harder to read and maintain.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// const a: Foo<string> = new Foo();
    /// const a = new Foo<string>(); // prefer type annotation
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// const a = new Foo<string>();
    /// const a: Foo<string> = new Foo(); // prefer type annotation
    /// ```
    ConsistentGenericConstructors,
    typescript,
    style,
    fix,
    config = ConsistentGenericConstructorsConfig
);

impl Rule for ConsistentGenericConstructors {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::VariableDeclarator(variable_declarator) => {
                let type_ann = variable_declarator.type_annotation.as_ref();
                let init = variable_declarator.init.as_ref();
                self.check(node, type_ann, init, ctx);
            }
            AstKind::FormalParameter(formal_parameter) => {
                let type_ann = formal_parameter.type_annotation.as_ref();
                let init = formal_parameter.initializer.as_deref();
                self.check(node, type_ann, init, ctx);
            }
            AstKind::PropertyDefinition(property_definition) => {
                let type_ann = property_definition.type_annotation.as_ref();
                let init = property_definition.value.as_ref();
                self.check(node, type_ann, init, ctx);
            }
            _ => {}
        }
    }

    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        Ok(Self(Box::new(ConsistentGenericConstructorsConfig {
            option: value
                .get(0)
                .and_then(|v| v.as_str())
                .and_then(|s| PreferGenericType::try_from(s).ok())
                .unwrap_or_default(),
        })))
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

impl ConsistentGenericConstructors {
    fn check<'a>(
        &self,
        node: &AstNode<'a>,
        type_annotation: Option<&oxc_allocator::Box<'a, TSTypeAnnotation<'a>>>,
        init: Option<&Expression<'a>>,
        ctx: &LintContext<'a>,
    ) {
        let Some(init) = init else { return };
        let Expression::NewExpression(new_expression) = init.get_inner_expression() else {
            return;
        };
        let Expression::Identifier(identifier) = &new_expression.callee else {
            return;
        };
        if let Some(type_annotation) = type_annotation {
            if let TSType::TSTypeReference(type_annotation) = &type_annotation.type_annotation {
                if let TSTypeName::IdentifierReference(ident) = &type_annotation.type_name {
                    if ident.name != identifier.name {
                        return;
                    }
                } else {
                    return;
                }
            } else {
                return;
            }
        }

        if matches!(self.0.option, PreferGenericType::TypeAnnotation) {
            if type_annotation.is_none()
                && let Some(type_arguments) = &new_expression.type_arguments
            {
                ctx.diagnostic_with_fix(
                    consistent_generic_constructors_diagnostic_prefer_annotation(
                        type_arguments.span,
                    ),
                    |fixer| {
                        Self::fix_prefer_type_annotation(
                            fixer,
                            node,
                            new_expression,
                            type_arguments,
                            ctx,
                        )
                    },
                );
            }
            return;
        }

        if let Some(type_ann) = &type_annotation
            && let TSType::TSTypeReference(type_ref) = &type_ann.type_annotation
            && let Some(type_params) = &type_ref.type_arguments
            && new_expression.type_arguments.is_none()
        {
            ctx.diagnostic_with_fix(
                consistent_generic_constructors_diagnostic_prefer_constructor(type_ann.span),
                |fixer| {
                    Self::fix_prefer_constructor(
                        fixer,
                        node,
                        type_ann,
                        type_ref,
                        type_params,
                        new_expression,
                        ctx,
                    )
                },
            );
        }
    }

    /// Fix for "prefer constructor" mode:
    /// Move type arguments from annotation to constructor
    /// e.g., `const a: Foo<string> = new Foo()` -> `const a = new Foo<string>()`
    fn fix_prefer_constructor<'a>(
        fixer: RuleFixer<'_, 'a>,
        node: &AstNode<'a>,
        type_ann: &TSTypeAnnotation<'a>,
        type_ref: &TSTypeReference<'a>,
        type_params: &TSTypeParameterInstantiation<'a>,
        new_expression: &NewExpression<'a>,
        ctx: &LintContext<'a>,
    ) -> crate::fixer::RuleFix {
        let fixer = fixer.for_multifix();
        let source_text = ctx.source_text();

        // Get the type arguments text
        let type_params_text =
            &source_text[type_params.span.start as usize..type_params.span.end as usize];

        // Find the position where the binding pattern ends (before the colon)
        let binding_end = Self::find_binding_end_position(node);

        // Find the colon position by searching only between the binding end and type annotation.
        // Use token-aware search to avoid picking up ':' characters inside comments.
        let colon_pos = if let Some(binding_end) = binding_end {
            ctx.find_next_token_within(binding_end, type_ann.span.start, ":")
                .map_or(type_ann.span.start, |offset| binding_end + offset)
        } else {
            type_ann.span.start
        };

        let type_name_start = type_ref.type_name.span().start;
        let type_name_end = type_ref.type_name.span().end;

        // Comments before type name (between colon and type name)
        let comments_before: String = ctx
            .comments_range((colon_pos + 1)..type_name_start)
            .map(|c| c.span.source_text(source_text))
            .collect();

        // Comments between type name and type arguments
        let comments_between: String = ctx
            .comments_range(type_name_end..type_params.span.start)
            .map(|c| c.span.source_text(source_text))
            .collect();

        // Build the new type arguments string to insert after constructor callee
        let new_type_args = format!("{comments_before}{comments_between}{type_params_text}");

        // Delete from before any whitespace preceding the colon to the end of the type annotation
        // This ensures we don't leave extra whitespace when removing ` : Type`
        let delete_start = {
            let before_colon = &source_text[..colon_pos as usize];
            let whitespace_len =
                before_colon.chars().rev().take_while(char::is_ascii_whitespace).count();
            #[expect(clippy::cast_possible_truncation)]
            {
                colon_pos - whitespace_len as u32
            }
        };
        let delete_span = Span::new(delete_start, type_ann.span.end);

        // Find where to insert type arguments in the new expression
        let callee_end = new_expression.callee.span().end;

        // Check if `new Foo;` (no parentheses) - need to handle this case
        // If the expression span ends at the callee span end (or type args if present), no parens
        let expr_end = new_expression.span.end;
        let callee_or_type_end =
            new_expression.type_arguments.as_ref().map_or(callee_end, |ta| ta.span.end);
        // Look for opening paren after the callee/type args
        let after_callee = &source_text[callee_or_type_end as usize..expr_end as usize];
        let needs_parens = !after_callee.contains('(');

        // Build the fix
        let mut fix = fixer.new_fix_with_capacity(if needs_parens { 3 } else { 2 });

        // Delete the type annotation (including leading colon and whitespace)
        fix.push(fixer.delete_range(delete_span));

        // Insert type arguments after callee
        fix.push(fixer.insert_text_after_range(Span::new(callee_end, callee_end), new_type_args));

        // If `new Foo;`, add empty parentheses at the end
        if needs_parens {
            let expr_end = new_expression.span.end;
            fix.push(fixer.insert_text_after_range(Span::new(expr_end, expr_end), "()"));
        }

        fix.with_message("Move the generic type to the constructor")
    }

    /// Fix for "prefer type annotation" mode:
    /// Move type arguments from constructor to annotation
    /// e.g., `const a = new Foo<string>()` -> `const a: Foo<string> = new Foo()`
    fn fix_prefer_type_annotation<'a>(
        fixer: RuleFixer<'_, 'a>,
        node: &AstNode<'a>,
        new_expression: &NewExpression<'a>,
        type_args: &TSTypeParameterInstantiation<'a>,
        ctx: &LintContext<'a>,
    ) -> crate::fixer::RuleFix {
        let fixer = fixer.for_multifix();
        let source_text = ctx.source_text();

        // Get the callee name (constructor name)
        let Expression::Identifier(callee_ident) = &new_expression.callee else {
            return fixer.noop();
        };
        let callee_name = callee_ident.name.as_str();

        // Get type arguments text (including any internal comments)
        let type_args_text =
            &source_text[type_args.span.start as usize..type_args.span.end as usize];

        // Build the type annotation to insert (no comments between name and type args)
        let type_annotation = format!(": {callee_name}{type_args_text}");

        // Find the position to insert the type annotation (after the binding pattern/identifier)
        let Some(insert_pos) = Self::find_type_annotation_insert_position(node, ctx) else {
            return fixer.noop();
        };

        // For the constructor, we need to delete just the type arguments (the `<...>` part)
        // but keep any comments that were between callee and type args
        // For `new Foo/* comment */ <string> /* another */()`, we delete ` <string>` (with leading space)
        // and keep `/* comment */` and `/* another */`
        let callee_end = new_expression.callee.span().end;

        // Check if there are only whitespace/comments between callee and type args
        // If there's whitespace before `<`, we delete from callee end to type args end
        // and replace with just the comments (if any)
        let comments_between: String = ctx
            .comments_range(callee_end..type_args.span.start)
            .map(|c| c.span.source_text(source_text))
            .collect();

        // The delete span is from callee end to type args end
        let delete_span = Span::new(callee_end, type_args.span.end);

        // Replacement: keep comments but remove type args
        let replacement =
            if comments_between.is_empty() { String::new() } else { comments_between };

        // Build the fix
        let mut fix = fixer.new_fix_with_capacity(2);

        // Insert type annotation after binding
        fix.push(fixer.insert_text_after_range(Span::new(insert_pos, insert_pos), type_annotation));

        // Replace the type arguments area (callee_end to type_args end) with just comments
        fix.push(fixer.replace(delete_span, replacement));

        fix.with_message("Move the generic type to the type annotation")
    }

    /// Find the position where the binding pattern ends (before the colon in type annotation)
    fn find_binding_end_position(node: &AstNode<'_>) -> Option<u32> {
        match node.kind() {
            AstKind::VariableDeclarator(var_decl) => Some(var_decl.id.span().end),
            AstKind::FormalParameter(param) => Some(param.pattern.span().end),
            AstKind::PropertyDefinition(prop_def) => Some(prop_def.key.span().end),
            _ => {
                debug_assert!(false, "Unexpected node kind in find_binding_end_position");
                None
            }
        }
    }

    /// Find the position to insert a type annotation for the current node
    fn find_type_annotation_insert_position(
        node: &AstNode<'_>,
        ctx: &LintContext<'_>,
    ) -> Option<u32> {
        match node.kind() {
            AstKind::VariableDeclarator(var_decl) => {
                // Insert after the binding identifier/pattern
                Some(var_decl.id.span().end)
            }
            AstKind::FormalParameter(param) => {
                // Insert after the binding pattern
                match &param.pattern {
                    BindingPattern::BindingIdentifier(ident) => Some(ident.span.end),
                    BindingPattern::ObjectPattern(obj) => Some(obj.span.end),
                    BindingPattern::ArrayPattern(arr) => Some(arr.span.end),
                    BindingPattern::AssignmentPattern(assign) => {
                        // For assignment pattern like `a = new Foo<string>()`,
                        // we need to insert after the left side
                        Some(assign.left.span().end)
                    }
                }
            }
            AstKind::PropertyDefinition(prop_def) => {
                // For computed properties like `[a]` or `[a + b]`, we need to insert
                // after the closing bracket `]`, not after the key expression
                if prop_def.computed {
                    // Find the closing bracket after the key
                    let key_end = prop_def.key.span().end;
                    // find_next_token_from returns offset from key_end, add 1 for position after ']'
                    ctx.find_next_token_from(key_end, "]").map(|offset| key_end + offset + 1)
                } else {
                    // Insert after the property key
                    Some(prop_def.key.span().end)
                }
            }
            _ => None,
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("const a = new Foo();", None),
        ("const a = new Foo<string>();", None),
        ("const a: Foo<string> = new Foo<string>();", None),
        ("const a: Foo = new Foo();", None),
        ("const a: Bar<string> = new Foo();", None),
        ("const a: Foo = new Foo<string>();", None),
        ("const a: Bar = new Foo<string>();", None),
        ("const a: Bar<string> = new Foo<string>();", None),
        ("const a: Foo<string> = Foo<string>();", None),
        ("const a: Foo<string> = Foo();", None),
        ("const a: Foo = Foo<string>();", None),
        (
            "
			class Foo {
			  a = new Foo<string>();
			}
			    ",
            None,
        ),
        (
            "
			function foo(a: Foo = new Foo<string>()) {}
			    ",
            None,
        ),
        (
            "
			function foo({ a }: Foo = new Foo<string>()) {}
			    ",
            None,
        ),
        (
            "
			function foo([a]: Foo = new Foo<string>()) {}
			    ",
            None,
        ),
        (
            "
			class A {
			  constructor(a: Foo = new Foo<string>()) {}
			}
			    ",
            None,
        ),
        (
            "
			const a = function (a: Foo = new Foo<string>()) {};
			    ",
            None,
        ),
        ("const a = new Foo();", Some(serde_json::json!(["type-annotation"]))),
        ("const a: Foo<string> = new Foo();", Some(serde_json::json!(["type-annotation"]))),
        ("const a: Foo<string> = new Foo<string>();", Some(serde_json::json!(["type-annotation"]))),
        ("const a: Foo = new Foo();", Some(serde_json::json!(["type-annotation"]))),
        ("const a: Bar = new Foo<string>();", Some(serde_json::json!(["type-annotation"]))),
        ("const a: Bar<string> = new Foo<string>();", Some(serde_json::json!(["type-annotation"]))),
        ("const a: Foo<string> = Foo<string>();", Some(serde_json::json!(["type-annotation"]))),
        ("const a: Foo<string> = Foo();", Some(serde_json::json!(["type-annotation"]))),
        ("const a: Foo = Foo<string>();", Some(serde_json::json!(["type-annotation"]))),
        ("const a = new (class C<T> {})<string>();", Some(serde_json::json!(["type-annotation"]))),
        (
            "
			class Foo {
			  a: Foo<string> = new Foo();
			}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			function foo(a: Foo<string> = new Foo()) {}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			function foo({ a }: Foo<string> = new Foo()) {}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			function foo([a]: Foo<string> = new Foo()) {}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			class A {
			  constructor(a: Foo<string> = new Foo()) {}
			}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			const a = function (a: Foo<string> = new Foo()) {};
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			const [a = new Foo<string>()] = [];
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			function a([a = new Foo<string>()]) {}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
    ];

    let fail = vec![
        ("const a: Foo<string> = new Foo();", None),
        ("const a: Map<string, number> = new Map();", None),
        ("const a: Map <string, number> = new Map();", None),
        ("const a: Map< string, number > = new Map();", None),
        ("const a: Map<string, number> = new Map ();", None),
        ("const a: Foo<number> = new Foo;", None),
        ("const a: /* comment */ Foo/* another */ <string> = new Foo();", None),
        ("const a: Foo/* comment */ <string> = new Foo /* another */();", None),
        (
            "const a: Foo<string> = new
			 Foo
			 ();",
            None,
        ),
        (
            "
			class Foo {
			  a: Foo<string> = new Foo();
			}
			      ",
            None,
        ),
        (
            "
			class Foo {
			  [a]: Foo<string> = new Foo();
			}
			      ",
            None,
        ),
        (
            "
			function foo(a: Foo<string> = new Foo()) {}
			      ",
            None,
        ),
        (
            "
			function foo({ a }: Foo<string> = new Foo()) {}
			      ",
            None,
        ),
        (
            "
			function foo([a]: Foo<string> = new Foo()) {}
			      ",
            None,
        ),
        (
            "
			class A {
			  constructor(a: Foo<string> = new Foo()) {}
			}
			      ",
            None,
        ),
        (
            "
			const a = function (a: Foo<string> = new Foo()) {};
			      ",
            None,
        ),
        ("const a = new Foo<string>();", Some(serde_json::json!(["type-annotation"]))),
        ("const a = new Map<string, number>();", Some(serde_json::json!(["type-annotation"]))),
        ("const a = new Map <string, number> ();", Some(serde_json::json!(["type-annotation"]))),
        ("const a = new Map< string, number >();", Some(serde_json::json!(["type-annotation"]))),
        (
            "const a = new
			 Foo<string>
			 ();",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "const a = new Foo/* comment */ <string> /* another */();",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "const a = new Foo</* comment */ string, /* another */ number>();",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			class Foo {
			  a = new Foo<string>();
			}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			class Foo {
			  [a] = new Foo<string>();
			}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			class Foo {
			  [a + b] = new Foo<string>();
			}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			function foo(a = new Foo<string>()) {}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			function foo({ a } = new Foo<string>()) {}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			function foo([a] = new Foo<string>()) {}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			class A {
			  constructor(a = new Foo<string>()) {}
			}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			const a = function (a = new Foo<string>()) {};
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
    ];

    let fix = vec![
        ("const a: Foo<string> = new Foo();", "const a = new Foo<string>();", None),
        ("const a: Map<string, number> = new Map();", "const a = new Map<string, number>();", None),
        (
            "const a: Map <string, number> = new Map();",
            "const a = new Map<string, number>();",
            None,
        ),
        (
            "const a: Map< string, number > = new Map();",
            "const a = new Map< string, number >();",
            None,
        ),
        (
            "const a: Map<string, number> = new Map ();",
            "const a = new Map<string, number> ();",
            None,
        ),
        ("const a: Foo<number> = new Foo;", "const a = new Foo<number>();", None),
        (
            "const a: /* comment */ Foo/* another */ <string> = new Foo();",
            "const a = new Foo/* comment *//* another */<string>();",
            None,
        ),
        (
            "const a: Foo/* comment */ <string> = new Foo /* another */();",
            "const a = new Foo/* comment */<string> /* another */();",
            None,
        ),
        (
            "const a: Foo<string> = new
			 Foo
			 ();",
            "const a = new
			 Foo<string>
			 ();",
            None,
        ),
        (
            "
			class Foo {
			  a: Foo<string> = new Foo();
			}
			      ",
            "
			class Foo {
			  a = new Foo<string>();
			}
			      ",
            None,
        ),
        (
            "
			class Foo {
			  [a]: Foo<string> = new Foo();
			}
			      ",
            "
			class Foo {
			  [a] = new Foo<string>();
			}
			      ",
            None,
        ),
        (
            "
			function foo(a: Foo<string> = new Foo()) {}
			      ",
            "
			function foo(a = new Foo<string>()) {}
			      ",
            None,
        ),
        (
            "
			function foo({ a }: Foo<string> = new Foo()) {}
			      ",
            "
			function foo({ a } = new Foo<string>()) {}
			      ",
            None,
        ),
        (
            "
			function foo([a]: Foo<string> = new Foo()) {}
			      ",
            "
			function foo([a] = new Foo<string>()) {}
			      ",
            None,
        ),
        (
            "
			class A {
			  constructor(a: Foo<string> = new Foo()) {}
			}
			      ",
            "
			class A {
			  constructor(a = new Foo<string>()) {}
			}
			      ",
            None,
        ),
        (
            "
			const a = function (a: Foo<string> = new Foo()) {};
			      ",
            "
			const a = function (a = new Foo<string>()) {};
			      ",
            None,
        ),
        (
            "const a = new Foo<string>();",
            "const a: Foo<string> = new Foo();",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "const a = new Map<string, number>();",
            "const a: Map<string, number> = new Map();",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "const a = new Map <string, number> ();",
            "const a: Map<string, number> = new Map ();",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "const a = new Map< string, number >();",
            "const a: Map< string, number > = new Map();",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "const a = new
			 Foo<string>
			 ();",
            "const a: Foo<string> = new
			 Foo
			 ();",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "const a = new Foo/* comment */ <string> /* another */();",
            "const a: Foo<string> = new Foo/* comment */ /* another */();",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "const a = new Foo</* comment */ string, /* another */ number>();",
            "const a: Foo</* comment */ string, /* another */ number> = new Foo();",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			class Foo {
			  a = new Foo<string>();
			}
			      ",
            "
			class Foo {
			  a: Foo<string> = new Foo();
			}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			class Foo {
			  [a] = new Foo<string>();
			}
			      ",
            "
			class Foo {
			  [a]: Foo<string> = new Foo();
			}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			class Foo {
			  [a + b] = new Foo<string>();
			}
			      ",
            "
			class Foo {
			  [a + b]: Foo<string> = new Foo();
			}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			function foo(a = new Foo<string>()) {}
			      ",
            "
			function foo(a: Foo<string> = new Foo()) {}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			function foo({ a } = new Foo<string>()) {}
			      ",
            "
			function foo({ a }: Foo<string> = new Foo()) {}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			function foo([a] = new Foo<string>()) {}
			      ",
            "
			function foo([a]: Foo<string> = new Foo()) {}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			class A {
			  constructor(a = new Foo<string>()) {}
			}
			      ",
            "
			class A {
			  constructor(a: Foo<string> = new Foo()) {}
			}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			const a = function (a = new Foo<string>()) {};
			      ",
            "
			const a = function (a: Foo<string> = new Foo()) {};
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "foo({ bar: 'all' });
             const baz: Map<number, number> = new Map();",
            "foo({ bar: 'all' });
             const baz = new Map<number, number>();",
            None,
        ),
        (
            "const baz /* note: map */ : Map<number, number> = new Map();",
            "const baz /* note: map */ = new Map<number, number>();",
            None,
        ),
    ];

    Tester::new(
        ConsistentGenericConstructors::NAME,
        ConsistentGenericConstructors::PLUGIN,
        pass,
        fail,
    )
    .expect_fix(fix)
    .test_and_snapshot();
}
