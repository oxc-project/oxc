use phf::{Map, phf_map};

use oxc_allocator::Allocator;
use oxc_ast::{
    AstBuilder, AstKind,
    ast::{
        Argument, BindingPatternKind, BindingProperty, CallExpression, Expression, Function,
        MemberExpression, PropertyKey,
    },
};
use oxc_codegen::CodegenOptions;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{SPAN, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_keyboard_event_key_diagnostic(span: Span, deprecated_prop: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Use `.key` instead of `.{deprecated_prop}`"))
        .with_help(format!("The `{deprecated_prop}` property is deprecated."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferKeyboardEventKey;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the use of [`KeyboardEvent#key`](https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/key) over [`KeyboardEvent#keyCode`](https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/keyCode) which is deprecated.
    /// The `.key` property is also more semantic and readable.
    ///
    /// ### Why is this bad?
    ///
    /// The `keyCode`, `which`, and `charCode` properties are deprecated and should be avoided in favor of the `key` property.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// window.addEventListener('keydown', event => {
    /// 	if (event.keyCode === 8) {
    /// 		console.log('Backspace was pressed');
    /// 	}
    /// });
    ///
    /// window.addEventListener('keydown', event => {
    /// 	console.log(event.keyCode);
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// window.addEventListener('keydown', event => {
    ///     if (event.key === 'Backspace') {
    ///     	console.log('Backspace was pressed');
    ///     }
    /// });
    ///
    /// window.addEventListener('click', event => {
    /// 	console.log(event.key);
    /// });
    /// ```
    PreferKeyboardEventKey,
    unicorn,
    style,
    fix
);

impl Rule for PreferKeyboardEventKey {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::StaticMemberExpression(member_expr) => {
                Self::check_static_member_expression(member_expr, node, ctx);
            }
            AstKind::BindingProperty(prop) => {
                Self::check_binding_property(prop, node, ctx);
            }
            _ => {}
        }
    }
}

impl PreferKeyboardEventKey {
    /// Check direct property access like `event.keyCode`
    fn check_static_member_expression<'a>(
        member_expr: &oxc_ast::ast::StaticMemberExpression<'a>,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) {
        // Check if this is a deprecated property name
        let property_name = member_expr.property.name.as_str();
        if !is_deprecated_property(property_name) {
            return;
        }

        // Get the object being accessed
        let object = &member_expr.object;

        // Check if the object is an event parameter from addEventListener
        let Some(event_param_symbol_id) = Self::get_event_param_symbol_id(node, ctx) else {
            return;
        };

        // Check if the object references the event parameter
        if !Self::is_reference_to_symbol(object, event_param_symbol_id, ctx) {
            return;
        }

        // Report the diagnostic
        Self::report_diagnostic_with_fix(property_name, member_expr.property.span, node, ctx);
    }

    /// Check destructuring patterns like `const { keyCode } = event`
    fn check_binding_property<'a>(
        prop: &BindingProperty<'a>,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) {
        // Get the property name from the key
        let PropertyKey::StaticIdentifier(key_ident) = &prop.key else {
            return;
        };
        let key_name = key_ident.name.as_str();

        // Get the value name (the identifier being introduced)
        let BindingPatternKind::BindingIdentifier(value_ident) = &prop.value.kind else {
            return;
        };
        let value_name = value_ident.name.as_str();

        // Determine which property name we're checking and if we should report
        // There are several cases:
        // 1. Shorthand: { keyCode } = event -> report keyCode
        // 2. Non-shorthand: { keyCode: abc } = event -> OK (renaming to non-deprecated name)
        // 3. Non-shorthand: { a: keyCode } = event -> report keyCode (introducing deprecated variable)
        let (should_report, deprecated_name) = if prop.shorthand {
            // Shorthand case: key and value are the same
            (is_deprecated_property(key_name), key_name)
        } else if is_deprecated_property(key_name) && !is_deprecated_property(value_name) {
            // { keyCode: abc } = event -> OK, renaming away from deprecated
            (false, key_name)
        } else if is_deprecated_property(value_name) {
            // { a: keyCode } = event -> bad, introducing deprecated variable name
            (true, value_name)
        } else {
            (false, key_name)
        };

        if !should_report {
            return;
        }

        // Find the VariableDeclarator parent to check what we're destructuring
        let Some((declarator_node, init)) = Self::find_variable_declarator_init(node, ctx) else {
            // Also check if the property is directly in the function parameter destructuring
            if Self::is_in_event_parameter_destructuring(node, ctx) {
                // Get the span for the value (the identifier being introduced)
                let value_span = match &prop.value.kind {
                    BindingPatternKind::BindingIdentifier(ident) => ident.span,
                    _ => return,
                };
                ctx.diagnostic(prefer_keyboard_event_key_diagnostic(value_span, deprecated_name));
            }
            return;
        };

        // Check if the init is an event parameter from addEventListener
        let Some(event_param_symbol_id) = Self::get_event_param_symbol_id(declarator_node, ctx)
        else {
            return;
        };

        // Check if init references the event parameter
        if !Self::is_reference_to_symbol(init, event_param_symbol_id, ctx) {
            return;
        }

        let BindingPatternKind::BindingIdentifier(ident) = &prop.value.kind else {
            return;
        };

        ctx.diagnostic(prefer_keyboard_event_key_diagnostic(ident.span, deprecated_name));
    }

    /// Check if this property is directly in the event parameter destructuring
    /// e.g., addEventListener('click', ({ keyCode }) => {})
    fn is_in_event_parameter_destructuring<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
        // Walk up to find if we're in a function parameter
        let mut current = node;
        loop {
            let parent = ctx.nodes().parent_node(current.id());
            match parent.kind() {
                AstKind::FormalParameters(_) => {
                    // Check if the function containing these params is an addEventListener callback
                    let func_node = ctx.nodes().parent_node(parent.id());
                    return Self::is_add_event_listener_callback(func_node, ctx);
                }
                AstKind::Function(_)
                | AstKind::ArrowFunctionExpression(_)
                | AstKind::FunctionBody(_)
                | AstKind::Program(_) => {
                    return false;
                }
                _ => {
                    current = parent;
                }
            }
        }
    }

    /// Find the init expression of a VariableDeclarator
    fn find_variable_declarator_init<'a, 'b>(
        node: &'b AstNode<'a>,
        ctx: &'b LintContext<'a>,
    ) -> Option<(&'b AstNode<'a>, &'a Expression<'a>)> {
        let mut current = node;
        loop {
            let parent = ctx.nodes().parent_node(current.id());
            match parent.kind() {
                AstKind::VariableDeclarator(declarator) => {
                    return declarator.init.as_ref().map(|init| (parent, init));
                }
                AstKind::Function(_)
                | AstKind::ArrowFunctionExpression(_)
                | AstKind::FunctionBody(_)
                | AstKind::Program(_) => {
                    return None;
                }
                _ => {
                    current = parent;
                }
            }
        }
    }

    /// Get the symbol ID of the event parameter from addEventListener
    fn get_event_param_symbol_id<'a>(
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) -> Option<oxc_semantic::SymbolId> {
        // Walk up to find the addEventListener call and its callback
        let callback = Self::find_add_event_listener_callback(node, ctx)?;

        // Get the first parameter of the callback (the event)
        let params = match callback {
            CallbackFunction::Arrow(arrow) => &arrow.params,
            CallbackFunction::Regular(func) => &func.params,
        };

        let first_param = params.items.first()?;

        match &first_param.pattern.kind {
            BindingPatternKind::BindingIdentifier(ident) => Some(ident.symbol_id()),
            BindingPatternKind::ObjectPattern(_) => {
                // If the event itself is destructured, we handle this separately
                None
            }
            _ => None,
        }
    }

    /// Check if an expression references a specific symbol
    fn is_reference_to_symbol(
        expr: &Expression,
        symbol_id: oxc_semantic::SymbolId,
        ctx: &LintContext,
    ) -> bool {
        match expr {
            Expression::Identifier(ident) => {
                ctx.scoping().get_reference(ident.reference_id()).symbol_id() == Some(symbol_id)
            }
            _ => false,
        }
    }

    /// Check if a node is an addEventListener callback
    fn is_add_event_listener_callback<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
        Self::find_add_event_listener_callback(node, ctx).is_some()
    }

    /// Find the callback function of an addEventListener call that contains this node
    fn find_add_event_listener_callback<'a>(
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) -> Option<CallbackFunction<'a>> {
        let mut current_node = node;
        let mut callback: Option<CallbackFunction<'a>> = None;

        // Check if the node itself is a function
        match node.kind() {
            AstKind::ArrowFunctionExpression(arrow) => {
                callback = Some(CallbackFunction::Arrow(arrow));
            }
            AstKind::Function(func) => {
                callback = Some(CallbackFunction::Regular(func));
            }
            _ => {}
        }

        loop {
            let parent = ctx.nodes().parent_node(current_node.id());

            match parent.kind() {
                AstKind::ArrowFunctionExpression(arrow) => {
                    callback = Some(CallbackFunction::Arrow(arrow));
                }
                AstKind::Function(func) => {
                    callback = Some(CallbackFunction::Regular(func));
                }
                AstKind::CallExpression(call) => {
                    // Check if this is addEventListener
                    if Self::is_add_event_listener_call(call) && callback.is_some() {
                        // Verify the callback is the second argument
                        if Self::is_callback_argument(call, callback.as_ref()) {
                            return callback;
                        }
                    }
                    // Not our addEventListener, keep looking up
                }
                AstKind::Program(_) => {
                    return None;
                }
                _ => {}
            }

            current_node = parent;
        }
    }

    /// Check if a call expression is addEventListener
    fn is_add_event_listener_call(call: &CallExpression) -> bool {
        let Some(member_expr) = call.callee.get_member_expr() else {
            return false;
        };

        let MemberExpression::StaticMemberExpression(static_member) = member_expr else {
            return false;
        };

        static_member.property.name == "addEventListener"
    }

    /// Check if the callback is the second argument of the call
    fn is_callback_argument(call: &CallExpression, callback: Option<&CallbackFunction>) -> bool {
        let Some(callback) = callback else {
            return false;
        };

        let Some(second_arg) = call.arguments.get(1) else {
            return false;
        };

        match second_arg {
            Argument::ArrowFunctionExpression(arrow) => {
                matches!(callback, CallbackFunction::Arrow(cb) if std::ptr::eq(*cb, arrow.as_ref()))
            }
            Argument::FunctionExpression(func) => {
                matches!(callback, CallbackFunction::Regular(cb) if std::ptr::eq(*cb, func.as_ref()))
            }
            _ => false,
        }
    }

    /// Report diagnostic with auto-fix for direct property access
    fn report_diagnostic_with_fix<'a>(
        property_name: &str,
        property_span: Span,
        member_node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) {
        // Try to find if this is part of a binary comparison like `event.keyCode === 27`
        let parent = ctx.nodes().parent_node(member_node.id());

        let AstKind::BinaryExpression(binary) = parent.kind() else {
            ctx.diagnostic(prefer_keyboard_event_key_diagnostic(property_span, property_name));
            return;
        };

        // Check if it's an equality comparison
        if !matches!(
            binary.operator,
            oxc_ast::ast::BinaryOperator::Equality | oxc_ast::ast::BinaryOperator::StrictEquality
        ) {
            ctx.diagnostic(prefer_keyboard_event_key_diagnostic(property_span, property_name));
            return;
        }

        // Get the numeric literal from the comparison
        let number_value = match (&binary.left, &binary.right) {
            (_, Expression::NumericLiteral(num)) | (Expression::NumericLiteral(num), _) => {
                Some((num.value, num.span))
            }
            _ => None,
        };

        let Some((code_value, number_span)) = number_value else {
            ctx.diagnostic(prefer_keyboard_event_key_diagnostic(property_span, property_name));
            return;
        };

        // Get the key name for this code
        let Some(key_name) = get_key_from_code(code_value) else {
            ctx.diagnostic(prefer_keyboard_event_key_diagnostic(property_span, property_name));
            return;
        };

        // Apply fix
        ctx.diagnostic_with_fix(
            prefer_keyboard_event_key_diagnostic(property_span, property_name),
            |fixer| {
                let mut codegen = fixer
                    .codegen()
                    .with_options(CodegenOptions { single_quote: true, ..Default::default() });
                let alloc = Allocator::default();
                let ast = AstBuilder::new(&alloc);
                codegen.print_expression(&ast.expression_string_literal(
                    SPAN,
                    ast.atom(&key_name),
                    None,
                ));
                let key_str = codegen.into_source_text();

                let mut fix = fixer.new_fix_with_capacity(2);
                fix.push(fixer.replace(property_span, "key"));
                fix.push(fixer.replace(number_span, key_str));
                fix.with_message(format!("Replace `.{property_name}` with `.key`"))
            },
        );
    }
}

/// Enum to represent either an arrow function or regular function callback
enum CallbackFunction<'a> {
    Arrow(&'a oxc_ast::ast::ArrowFunctionExpression<'a>),
    Regular(&'a Function<'a>),
}

/// Check if a property name is a deprecated keyboard event property
fn is_deprecated_property(name: &str) -> bool {
    matches!(name, "keyCode" | "charCode" | "which")
}

/// Map from key code to key name for auto-fix
/// Based on <https://github.com/sindresorhus/eslint-plugin-unicorn/blob/main/rules/shared/event-keys.js>
const KEY_CODE_TO_KEY: Map<u32, &'static str> = phf_map! {
    8u32 => "Backspace",
    9u32 => "Tab",
    12u32 => "Clear",
    13u32 => "Enter",
    16u32 => "Shift",
    17u32 => "Control",
    18u32 => "Alt",
    19u32 => "Pause",
    20u32 => "CapsLock",
    27u32 => "Escape",
    32u32 => " ",
    33u32 => "PageUp",
    34u32 => "PageDown",
    35u32 => "End",
    36u32 => "Home",
    37u32 => "ArrowLeft",
    38u32 => "ArrowUp",
    39u32 => "ArrowRight",
    40u32 => "ArrowDown",
    45u32 => "Insert",
    46u32 => "Delete",
    112u32 => "F1",
    113u32 => "F2",
    114u32 => "F3",
    115u32 => "F4",
    116u32 => "F5",
    117u32 => "F6",
    118u32 => "F7",
    119u32 => "F8",
    120u32 => "F9",
    121u32 => "F10",
    122u32 => "F11",
    123u32 => "F12",
    144u32 => "NumLock",
    145u32 => "ScrollLock",
    186u32 => ";",
    187u32 => "=",
    188u32 => ",",
    189u32 => "-",
    190u32 => ".",
    191u32 => "/",
    219u32 => "[",
    220u32 => "\\",
    221u32 => "]",
    222u32 => "'",
    224u32 => "Meta",
};

/// Get the key string from a key code, either from the mapping or from char code
fn get_key_from_code(code: f64) -> Option<String> {
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let code_u32 = code as u32;

    // First try the known key mapping
    if let Some(key) = KEY_CODE_TO_KEY.get(&code_u32) {
        return Some((*key).to_string());
    }

    // For printable characters (A-Z, 0-9, etc.), convert from char code
    char::from_u32(code_u32).map(|c| c.to_string())
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "window.addEventListener('click', e => {
				console.log(e.key);
			})",
        "window.addEventListener('click', () => {
				console.log(keyCode, which, charCode);
				console.log(window.keyCode);
			})",
        "foo.addEventListener('click', (e, r, fg) => {
				function a() {
					if (true) {
						{
							{
								const e = {};
								const { charCode } = e;
								console.log(e.keyCode, charCode);
							}
						}
					}
				}
			});",
        "const e = {}
			foo.addEventListener('click', function (event) {
				function a() {
					if (true) {
						{
							{
								console.log(e.keyCode);
							}
						}
					}
				}
			});",
        "const { keyCode } = e",
        "const { charCode } = e",
        "const {a, b, c} = event",
        "const keyCode = () => 4",
        "const which = keyCode => 5",
        "function which(abc) { const {keyCode} = abc; return keyCode}",
        "const { which } = e",
        "const { keyCode: key } = e",
        "const { keyCode: abc } = e",
        "foo.addEventListener('keydown', e => {
				(function (abc) {
					if (e.key === 'ArrowLeft') return true;
					const { charCode } = abc;
				}())
			})",
        "foo.addEventListener('keydown', e => {
					if (e.key === 'ArrowLeft') return true;
				})",
        "a.addEventListener('keyup', function (event) {
					const key = event.key;
				})",
        "a.addEventListener('keyup', function (event) {
					const { key } = event;
				})",
        "foo.addEventListener('click', e => {
					const good = {};
					good.keyCode = '34';
				});",
        "foo.addEventListener('click', e => {
					const good = {};
					good.charCode = '34';
				});",
        "foo.addEventListener('click', e => {
					const good = {};
					good.which = '34';
				});",
        "foo.addEventListener('click', e => {
					const {keyCode: a, charCode: b, charCode: c} = e;
				});",
        "add.addEventListener('keyup', event => {
					f.addEventList('some', e => {
						const {charCode} = e;
						console.log(event.key)
					})
				})",
        "foo.addEventListener('click', e => {
					{
						const e = {};
						console.log(e.keyCode);
					}
				});",
    ];

    let fail = vec![
        "window.addEventListener('click', e => {
				console.log(e.keyCode);
			})",
        "window.addEventListener('click', ({keyCode}) => {
				console.log(keyCode);
			})",
        "window.addEventListener('click', ({which}) => {
				if (which === 23) {
					console.log('Wrong!')
				}
			})",
        "window.addEventListener('click', ({which, another}) => {
				if (which === 23) {
					console.log('Wrong!')
				}
			})",
        "foo123.addEventListener('click', event => {
				if (event.keyCode === 27) {
				}
			});",
        "foo.addEventListener('click', event => {
				if (event.keyCode === 65) {}
			});",
        "foo.addEventListener('click', event => {
				if (event.keyCode === 10) {}
			});",
        "foo.addEventListener('click', event => {
				if (!event.keyCode) {}
			});",
        "foo.addEventListener('click', a => {
				if (a.keyCode === 27) {
				}
			});",
        "foo.addEventListener('click', (a, b, c) => {
				if (a.keyCode === 27) {
				}
			});",
        "foo.addEventListener('click', function(a, b, c) {
				if (a.keyCode === 27) {
				}
			});",
        "foo.addEventListener('click', function(b) {
				if (b.keyCode === 27) {
				}
			});",
        "foo.addEventListener('click', e => {
				const {keyCode, a, b} = e;
			});",
        "foo.addEventListener('click', e => {
				const {a: keyCode, a, b} = e;
			});",
        "add.addEventListener('keyup', event => {
				f.addEventList('some', e => {
					const {keyCode} = event;
					console.log(event.key)
				})
			})",
        "window.addEventListener('click', e => {
				console.log(e.charCode);
			})",
        "foo11111111.addEventListener('click', event => {
				if (event.charCode === 27) {
				}
			});",
        "foo.addEventListener('click', a => {
				if (a.charCode === 27) {
				}
			});",
        "foo.addEventListener('click', (a, b, c) => {
				if (a.charCode === 27) {
				}
			});",
        "foo.addEventListener('click', function(a, b, c) {
				if (a.charCode === 27) {
				}
			});",
        "foo.addEventListener('click', function(b) {
				if (b.charCode === 27) {
				}
			});",
        "foo.addEventListener('click', e => {
				const {charCode, a, b} = e;
			});",
        "foo.addEventListener('click', e => {
				const {a: charCode, a, b} = e;
			});",
        "window.addEventListener('click', e => {
				console.log(e.which);
			})",
        "foo.addEventListener('click', event => {
				if (event.which === 27) {
				}
			});",
        "foo.addEventListener('click', a => {
				if (a.which === 27) {
				}
			});",
        "foo.addEventListener('click', (a, b, c) => {
				if (a.which === 27) {
				}
			});",
        "foo.addEventListener('click', function(a, b, c) {
				if (a.which === 27) {
				}
			});",
        "foo.addEventListener('click', function(b) {
				if (b.which === 27) {
				}
			});",
        "foo.addEventListener('click', e => {
				const {which, a, b} = e;
			});",
        "foo.addEventListener('click', e => {
				const {a: which, a, b} = e;
			});",
        "foo.addEventListener('click', function(b) {
				if (b.which === 27) {
				}
				const {keyCode} = b;
				if (keyCode === 32) return 4;
			});",
        "foo.addEventListener('click', function(b) {
				if (b.which > 27) {
				}
				const {keyCode} = b;
				if (keyCode === 32) return 4;
			});",
        "const e = {}
			foo.addEventListener('click', (e, r, fg) => {
				function a() {
					if (true) {
						{
							{
								const { charCode } = e;
								console.log(e.keyCode, charCode);
							}
						}
					}
				}
			});",
        "foo123.addEventListener('click', event => {
				if (event.keyCode === 13) {
				}
			});",
        "foo123.addEventListener('click', event => {
				if (event.keyCode === 38) {
				}
			});",
        "foo123.addEventListener('click', event => {
				if (event.keyCode === 40) {
				}
			});",
        "foo123.addEventListener('click', event => {
				if (event.keyCode === 37) {
				}
			});",
        "foo123.addEventListener('click', event => {
				if (event.keyCode === 39) {
				}
			});",
        "foo123.addEventListener('click', event => {
				if (event.keyCode === 221) {
				}
			});",
        "foo123.addEventListener('click', event => {
				if (event.keyCode === 186) {
				}
			});",
        "foo123.addEventListener('click', event => {
				if (event.keyCode === 187) {
				}
			});",
        "foo123.addEventListener('click', event => {
				if (event.keyCode === 188) {
				}
			});",
        "foo123.addEventListener('click', event => {
				if (event.keyCode === 189) {
				}
			});",
        "foo123.addEventListener('click', event => {
				if (event.keyCode === 190) {
				}
			});",
        "foo123.addEventListener('click', event => {
				if (event.keyCode === 191) {
				}
			});",
        "foo123.addEventListener('click', event => {
				if (event.keyCode === 219) {
				}
			});",
        "foo123.addEventListener('click', event => {
				if (event.keyCode === 222) {
				}
			});",
        "window.addEventListener('click', ({which, another}) => {
				if (which === 23) {
					console.log('Wrong!')
				}
			})",
        "foo123.addEventListener('click', event => {
				if (event.keyCode === 27) {
				}
			});",
    ];

    let fix = vec![
        (
            "foo123.addEventListener('click', event => {
				if (event.keyCode === 27) {
				}
			});",
            "foo123.addEventListener('click', event => {
				if (event.key === 'Escape') {
				}
			});",
            None,
        ),
        (
            "foo.addEventListener('click', event => {
				if (event.keyCode === 65) {}
			});",
            "foo.addEventListener('click', event => {
				if (event.key === 'A') {}
			});",
            None,
        ),
        (
            "foo.addEventListener('click', event => {
				if (event.keyCode === 10) {}
			});",
            r"foo.addEventListener('click', event => {
				if (event.key === '\n') {}
			});",
            None,
        ),
        (
            "foo.addEventListener('click', a => {
				if (a.keyCode === 27) {
				}
			});",
            "foo.addEventListener('click', a => {
				if (a.key === 'Escape') {
				}
			});",
            None,
        ),
        (
            "foo.addEventListener('click', (a, b, c) => {
				if (a.keyCode === 27) {
				}
			});",
            "foo.addEventListener('click', (a, b, c) => {
				if (a.key === 'Escape') {
				}
			});",
            None,
        ),
        (
            "foo.addEventListener('click', function(a, b, c) {
				if (a.keyCode === 27) {
				}
			});",
            "foo.addEventListener('click', function(a, b, c) {
				if (a.key === 'Escape') {
				}
			});",
            None,
        ),
        (
            "foo.addEventListener('click', function(b) {
				if (b.keyCode === 27) {
				}
			});",
            "foo.addEventListener('click', function(b) {
				if (b.key === 'Escape') {
				}
			});",
            None,
        ),
        (
            "foo11111111.addEventListener('click', event => {
				if (event.charCode === 27) {
				}
			});",
            "foo11111111.addEventListener('click', event => {
				if (event.key === 'Escape') {
				}
			});",
            None,
        ),
        (
            "foo.addEventListener('click', a => {
				if (a.charCode === 27) {
				}
			});",
            "foo.addEventListener('click', a => {
				if (a.key === 'Escape') {
				}
			});",
            None,
        ),
        (
            "foo.addEventListener('click', (a, b, c) => {
				if (a.charCode === 27) {
				}
			});",
            "foo.addEventListener('click', (a, b, c) => {
				if (a.key === 'Escape') {
				}
			});",
            None,
        ),
        (
            "foo.addEventListener('click', function(a, b, c) {
				if (a.charCode === 27) {
				}
			});",
            "foo.addEventListener('click', function(a, b, c) {
				if (a.key === 'Escape') {
				}
			});",
            None,
        ),
        (
            "foo.addEventListener('click', function(b) {
				if (b.charCode === 27) {
				}
			});",
            "foo.addEventListener('click', function(b) {
				if (b.key === 'Escape') {
				}
			});",
            None,
        ),
        (
            "foo.addEventListener('click', event => {
				if (event.which === 27) {
				}
			});",
            "foo.addEventListener('click', event => {
				if (event.key === 'Escape') {
				}
			});",
            None,
        ),
        (
            "foo.addEventListener('click', a => {
				if (a.which === 27) {
				}
			});",
            "foo.addEventListener('click', a => {
				if (a.key === 'Escape') {
				}
			});",
            None,
        ),
        (
            "foo.addEventListener('click', (a, b, c) => {
				if (a.which === 27) {
				}
			});",
            "foo.addEventListener('click', (a, b, c) => {
				if (a.key === 'Escape') {
				}
			});",
            None,
        ),
        (
            "foo.addEventListener('click', function(a, b, c) {
				if (a.which === 27) {
				}
			});",
            "foo.addEventListener('click', function(a, b, c) {
				if (a.key === 'Escape') {
				}
			});",
            None,
        ),
        (
            "foo.addEventListener('click', function(b) {
				if (b.which === 27) {
				}
			});",
            "foo.addEventListener('click', function(b) {
				if (b.key === 'Escape') {
				}
			});",
            None,
        ),
        (
            "foo.addEventListener('click', function(b) {
				if (b.which === 27) {
				}
				const {keyCode} = b;
				if (keyCode === 32) return 4;
			});",
            "foo.addEventListener('click', function(b) {
				if (b.key === 'Escape') {
				}
				const {keyCode} = b;
				if (keyCode === 32) return 4;
			});",
            None,
        ),
        (
            "foo123.addEventListener('click', event => {
				if (event.keyCode === 13) {
				}
			});",
            "foo123.addEventListener('click', event => {
				if (event.key === 'Enter') {
				}
			});",
            None,
        ),
        (
            "foo123.addEventListener('click', event => {
				if (event.keyCode === 38) {
				}
			});",
            "foo123.addEventListener('click', event => {
				if (event.key === 'ArrowUp') {
				}
			});",
            None,
        ),
        (
            "foo123.addEventListener('click', event => {
				if (event.keyCode === 40) {
				}
			});",
            "foo123.addEventListener('click', event => {
				if (event.key === 'ArrowDown') {
				}
			});",
            None,
        ),
        (
            "foo123.addEventListener('click', event => {
				if (event.keyCode === 37) {
				}
			});",
            "foo123.addEventListener('click', event => {
				if (event.key === 'ArrowLeft') {
				}
			});",
            None,
        ),
        (
            "foo123.addEventListener('click', event => {
				if (event.keyCode === 39) {
				}
			});",
            "foo123.addEventListener('click', event => {
				if (event.key === 'ArrowRight') {
				}
			});",
            None,
        ),
        (
            "foo123.addEventListener('click', event => {
				if (event.keyCode === 221) {
				}
			});",
            "foo123.addEventListener('click', event => {
				if (event.key === ']') {
				}
			});",
            None,
        ),
        (
            "foo123.addEventListener('click', event => {
				if (event.keyCode === 186) {
				}
			});",
            "foo123.addEventListener('click', event => {
				if (event.key === ';') {
				}
			});",
            None,
        ),
        (
            "foo123.addEventListener('click', event => {
				if (event.keyCode === 187) {
				}
			});",
            "foo123.addEventListener('click', event => {
				if (event.key === '=') {
				}
			});",
            None,
        ),
        (
            "foo123.addEventListener('click', event => {
				if (event.keyCode === 188) {
				}
			});",
            "foo123.addEventListener('click', event => {
				if (event.key === ',') {
				}
			});",
            None,
        ),
        (
            "foo123.addEventListener('click', event => {
				if (event.keyCode === 189) {
				}
			});",
            "foo123.addEventListener('click', event => {
				if (event.key === '-') {
				}
			});",
            None,
        ),
        (
            "foo123.addEventListener('click', event => {
				if (event.keyCode === 190) {
				}
			});",
            "foo123.addEventListener('click', event => {
				if (event.key === '.') {
				}
			});",
            None,
        ),
        (
            "foo123.addEventListener('click', event => {
				if (event.keyCode === 191) {
				}
			});",
            "foo123.addEventListener('click', event => {
				if (event.key === '/') {
				}
			});",
            None,
        ),
        (
            "foo123.addEventListener('click', event => {
				if (event.keyCode === 219) {
				}
			});",
            "foo123.addEventListener('click', event => {
				if (event.key === '[') {
				}
			});",
            None,
        ),
        (
            "foo123.addEventListener('click', event => {
				if (event.keyCode === 222) {
				}
			});",
            "foo123.addEventListener('click', event => {
				if (event.key === '\\'') {
				}
			});",
            None,
        ),
    ];
    Tester::new(PreferKeyboardEventKey::NAME, PreferKeyboardEventKey::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
