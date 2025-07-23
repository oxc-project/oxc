use itertools::Itertools;
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNode, AstNodes, NodeId};
use oxc_span::GetSpan;
use oxc_span::Span;
use oxc_syntax::class::ElementKind;

use crate::{context::LintContext, rule::Rule};

fn no_unused_private_class_members_diagnostic(name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{name}' is defined but never used.")).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnusedPrivateClassMembers;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow unused private class members
    ///
    /// ### Why is this bad?
    ///
    /// Private class members that are declared and not used anywhere in the code are most likely an error due to incomplete refactoring. Such class members take up space in the code and can lead to confusion by readers.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// class A {
    ///		#unusedMember = 5;
    ///	}
    ///
    ///	class B {
    ///			#usedOnlyInWrite = 5;
    ///			method() {
    ///					this.#usedOnlyInWrite = 42;
    ///			}
    ///	}
    ///
    ///	class C {
    ///			#usedOnlyToUpdateItself = 5;
    ///			method() {
    ///					this.#usedOnlyToUpdateItself++;
    ///			}
    ///	}
    ///
    ///	class D {
    ///			#unusedMethod() {}
    ///	}
    ///
    ///	class E {
    ///			get #unusedAccessor() {}
    ///			set #unusedAccessor(value) {}
    ///	}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// class A {
    ///		#usedMember = 42;
    ///		method() {
    ///				return this.#usedMember;
    ///		}
    ///	}
    ///
    ///	class B {
    ///			#usedMethod() {
    ///					return 42;
    ///			}
    ///			anotherMethod() {
    ///					return this.#usedMethod();
    ///			}
    ///	}
    ///
    ///	class C {
    ///			get #usedAccessor() {}
    ///			set #usedAccessor(value) {}
    ///
    ///			method() {
    ///					this.#usedAccessor = 42;
    ///			}
    ///	}
    /// ```
    NoUnusedPrivateClassMembers,
    eslint,
    correctness
);

impl Rule for NoUnusedPrivateClassMembers {
    fn run_once(&self, ctx: &LintContext) {
        ctx.classes().iter_enumerated().for_each(|(class_id, _)| {
            for (element_id, element) in ctx.classes().elements[class_id].iter_enumerated() {
                if !element.kind.intersects(ElementKind::Property | ElementKind::Method) {
                    continue;
                }

                if element.is_private {
                    let mut found = false;
                    for ident in ctx.classes().iter_private_identifiers(class_id) {
                        let is_property = element.kind.is_property();
                        let read = !is_property || is_read(ident.id, ctx.nodes());
                        if read && ident.element_ids.contains(&element_id) {
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        ctx.diagnostic(no_unused_private_class_members_diagnostic(
                            &element.name,
                            element.span,
                        ));
                    }
                }
            }
        });
    }
}

fn is_read(current_node_id: NodeId, nodes: &AstNodes) -> bool {
    for (curr, parent) in
        nodes.ancestors(current_node_id).tuple_windows::<(&AstNode<'_>, &AstNode<'_>)>()
    {
        match (curr.kind(), parent.kind()) {
            // Skip member expressions in identifier context
            (member_expr, AstKind::IdentifierReference(_))
                if member_expr.is_member_expression_kind() => {}
            // Skip identifier references in assignment targets
            (
                AstKind::IdentifierReference(_),
                AstKind::ArrayAssignmentTarget(_)
                | AstKind::ObjectAssignmentTarget(_)
                | AstKind::IdentifierReference(_),
            ) => {}
            // Write-only contexts
            (
                AstKind::ArrayAssignmentTarget(_)
                | AstKind::ObjectAssignmentTarget(_)
                | AstKind::IdentifierReference(_),
                AstKind::ForInStatement(_)
                | AstKind::ForOfStatement(_)
                | AstKind::AssignmentTargetWithDefault(_)
                | AstKind::AssignmentTargetPropertyIdentifier(_)
                | AstKind::ArrayAssignmentTarget(_)
                | AstKind::ObjectAssignmentTarget(_)
                | AstKind::AssignmentTargetRest(_)
                | AstKind::AssignmentTargetPropertyProperty(_),
            ) => {
                return false;
            }
            // Assignment and update expressions are writes
            (AstKind::IdentifierReference(_), AstKind::AssignmentExpression(_))
            | (_, AstKind::UpdateExpression(_)) => {
                return false;
            }
            // All these are read contexts for private fields
            (
                AstKind::PrivateFieldExpression(_),
                AstKind::ReturnStatement(_)
                | AstKind::CallExpression(_)
                | AstKind::BinaryExpression(_)
                | AstKind::VariableDeclarator(_)
                | AstKind::ExpressionStatement(_)
                | AstKind::PropertyDefinition(_)
                | AstKind::Argument(_),
            ) => {
                return true;
            }
            // AssignmentExpression: right-hand side is a read, compound assignment result in value context is a read
            (AstKind::PrivateFieldExpression(_), AstKind::AssignmentExpression(assign_expr)) => {
                if assign_expr.right.span() == curr.span() {
                    return true;
                }
                use oxc_ast::ast::AssignmentOperator;
                if assign_expr.operator != AssignmentOperator::Assign {
                    if let Some((_, grandparent)) = nodes
                        .ancestors(parent.id())
                        .tuple_windows::<(&AstNode<'_>, &AstNode<'_>)>()
                        .next()
                    {
                        match grandparent.kind() {
                            AstKind::CallExpression(_)
                            | AstKind::ReturnStatement(_)
                            | AstKind::VariableDeclarator(_)
                            | AstKind::ExpressionStatement(_)
                            | AstKind::BinaryExpression(_)
                            | AstKind::PropertyDefinition(_)
                            | AstKind::ArrayExpression(_)
                            | AstKind::ObjectProperty(_)
                            | AstKind::JSXExpressionContainer(_)
                            | AstKind::Argument(_) => {
                                return true;
                            }
                            _ => {}
                        }
                    }
                }
            }
            // ForIn/ForOf: only right-hand side is a read
            (AstKind::PrivateFieldExpression(_), AstKind::ForInStatement(for_in)) => {
                if for_in.right.span() == curr.span() {
                    return true;
                }
            }
            (AstKind::PrivateFieldExpression(_), AstKind::ForOfStatement(for_of)) => {
                if for_of.right.span() == curr.span() {
                    return true;
                }
            }
            // AssignmentTargetPropertyProperty: only computed property name is a read
            (
                AstKind::PrivateFieldExpression(_),
                AstKind::AssignmentTargetPropertyProperty(prop),
            ) => {
                if prop.computed && prop.name.span() == curr.span() {
                    return true;
                }
            }
            _ => {
                return false;
            }
        }
    }
    true
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"class Foo {}",
        r"class Foo {
        	    publicMember = 42;
        	}",
        r"class Foo {
        	    #usedMember = 42;
        	    method() {
        	        return this.#usedMember;
        	    }
        	}",
        r"class Foo {
        	    #usedMember = 42;
        	    anotherMember = this.#usedMember;
        	}",
        r"class Foo {
        	    #usedMember = 42;
        	    foo() {
        	        anotherMember = this.#usedMember;
        	    }
        	}",
        r"class C {
			    #usedMember;

			    foo() {
			        bar(this.#usedMember += 1);
			    }
			}",
        r"class Foo {
			    #usedMember = 42;
			    method() {
			        return someGlobalMethod(this.#usedMember);
			    }
			}",
        r"class C {
			    #usedInOuterClass;

			    foo() {
			        return class {};
			    }

			    bar() {
			        return this.#usedInOuterClass;
			    }
			}",
        r"class Foo {
			    #usedInForInLoop;
			    method() {
			        for (const bar in this.#usedInForInLoop) {

			        }
			    }
			}",
        r"class Foo {
			    #usedInForOfLoop;
			    method() {
			        for (const bar of this.#usedInForOfLoop) {

			        }
			    }
			}",
        r"class Foo {
			    #usedInAssignmentPattern;
			    method() {
			        [bar = 1] = this.#usedInAssignmentPattern;
			    }
			}",
        r"class Foo {
			    #usedInArrayPattern;
			    method() {
			        [bar] = this.#usedInArrayPattern;
			    }
			}",
        r"class Foo {
			    #usedInAssignmentPattern;
			    method() {
			        [bar] = this.#usedInAssignmentPattern;
			    }
			}",
        r"class C {
			    #usedInObjectAssignment;

			    method() {
			        ({ [this.#usedInObjectAssignment]: a } = foo);
			    }
			}",
        r"class C {
            set #accessorWithSetterFirst(value) {
                doSomething(value);
            }
            get #accessorWithSetterFirst() {
                return something();
            }
            method() {
                this.#accessorWithSetterFirst += 1;
            }
        }",
        r"class Foo {
            set #accessorUsedInMemberAccess(value) {}

            method(a) {
                [this.#accessorUsedInMemberAccess] = a;
            }
        }",
        r"class C {
            get #accessorWithGetterFirst() {
                return something();
            }
            set #accessorWithGetterFirst(value) {
                doSomething(value);
            }
            method() {
                this.#accessorWithGetterFirst += 1;
            }
        }",
        // This is complicated case. Support this case maybe effect performance, so we don't support it now.
        // r"class C {
        //     #usedInInnerClass;

        //     method(a) {
        //         return class {
        //             foo = a.#usedInInnerClass;
        //         }
        //     }
        // }",
        r"class Foo {
			    #usedMethod() {
			        return 42;
			    }
			    anotherMethod() {
			        return this.#usedMethod();
			    }
			}",
        r"class C {
            set #x(value) {
                doSomething(value);
            }

            foo() {
                this.#x = 1;
            }
        }",
        r"type Callback<T> = () => Promise<T> | T;

         export class Issue_11039<T> {
            load: () => Promise<T>;

            constructor(callback: Callback<T>) {
                this.load = () => this.#load(callback);
            }

            async #load(callback: Callback<T>) {
                callback;
            }
         }",
        r"class ChildProcess extends EventEmitter { #stdioObject; #createStdioObject() {} get stdio() { return (this.#stdioObject ??= this.#createStdioObject()); } }",
        "export class Foo { readonly #select = 123; override render() { return html`foo=${this.#select}`; } }",
        "export class Foo { #listened = false; bar() { if (!this.#listened) return; this.#listened = false; } } ",
        "export class RichText { #verticalScrollContainer; init() { const verticalScrollContainer = this.#verticalScrollContainer || (this.#verticalScrollContainer = this.verticalScrollContainerGetter?.() || null); } }",
        "class Foo  { #a = false; on() { return this.#a ? [data] : data; } set setA(value) { this.#a = value; } }",
        "class WeakReference { #i = 0; inc() { return ++this.#i; }; dec() { return --this.#i; } }",
        "class Foo { #d; constructor(d) { this.#d = d || kDefaultD; } get getD(): string { return this.#d!; } }",
        "class F { #o; initialize(output) { this.#o = output; } text(e) { return this.#o!.text(e); } }",
        "class Foo { #a; constructor(a) { this.#a = a; }; b(b?: string): this { this.#a!.setB(b); return this; } resetA() { this.#a = undefined; } }",
    ];

    let fail = vec![
        r"class Foo {
			    #unusedMember = 5;
			}",
        r"class First {}
			class Second {
			    #unusedMemberInSecondClass = 5;
			}",
        r"class First {
			    #unusedMemberInFirstClass = 5;
			}
			class Second {}",
        r"class First {
			    #firstUnusedMemberInSameClass = 5;
			    #secondUnusedMemberInSameClass = 5;
			}",
        r"class Foo {
			    #usedOnlyInWrite = 5;
			    method() {
			        this.#usedOnlyInWrite = 42;
			    }
			}",
        r"class Foo {
			    #usedOnlyInWriteStatement = 5;
			    method() {
			        this.#usedOnlyInWriteStatement += 42;
			    }
			}",
        r"class C {
			    #usedOnlyInIncrement;

			    foo() {
			        this.#usedOnlyInIncrement++;
			    }
			}",
        r"class C {
			    #unusedInOuterClass;

			    foo() {
			        return class {
			            #unusedInOuterClass;

			            bar() {
			                return this.#unusedInOuterClass;
			            }
			        };
			    }
			}",
        r"class C {
			    #unusedOnlyInSecondNestedClass;

			    foo() {
			        return class {
			            #unusedOnlyInSecondNestedClass;

			            bar() {
			                return this.#unusedOnlyInSecondNestedClass;
			            }
			        };
			    }

			    baz() {
			        return this.#unusedOnlyInSecondNestedClass;
			    }

			    bar() {
			        return class {
			            #unusedOnlyInSecondNestedClass;
			        }
			    }
			}",
        r"class Foo {
			    #unusedMethod() {}
			}",
        r"class Foo {
			    #unusedMethod() {}
			    #usedMethod() {
			        return 42;
			    }
			    publicMethod() {
			        return this.#usedMethod();
			    }
			}",
        r"class Foo {
			    set #unusedSetter(value) {}
			}",
        r"class Foo {
			    #unusedForInLoop;
			    method() {
			        for (this.#unusedForInLoop in bar) {

			        }
			    }
			}",
        r"class Foo {
			    #unusedForOfLoop;
			    method() {
			        for (this.#unusedForOfLoop of bar) {

			        }
			    }
			}",
        r"class Foo {
			    #unusedInDestructuring;
			    method() {
			        ({ x: this.#unusedInDestructuring } = bar);
			    }
			}",
        r"class Foo {
			    #unusedInRestPattern;
			    method() {
			        [...this.#unusedInRestPattern] = bar;
			    }
			}",
        r"class Foo {
			    #unusedInAssignmentPattern;
			    method() {
			        [this.#unusedInAssignmentPattern = 1] = bar;
			    }
			}",
        r"class Foo {
			    #unusedInAssignmentPattern;
			    method() {
			        [this.#unusedInAssignmentPattern] = bar;
			    }
			}",
        r"class C {
			    #usedOnlyInTheSecondInnerClass;

			    method(a) {
			        return class {
			            #usedOnlyInTheSecondInnerClass;

			            method2(b) {
			                foo = b.#usedOnlyInTheSecondInnerClass;
			            }

			            method3(b) {
			                foo = b.#usedOnlyInTheSecondInnerClass;
			            }
			        }
			    }
			}",
    ];

    Tester::new(NoUnusedPrivateClassMembers::NAME, NoUnusedPrivateClassMembers::PLUGIN, pass, fail)
        .test_and_snapshot();
}
