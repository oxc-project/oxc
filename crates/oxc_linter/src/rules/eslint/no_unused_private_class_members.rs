use oxc_ast::{
    ast::{Expression, MethodDefinitionKind},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNodeId;
use oxc_span::{Atom, Span};
use rustc_hash::FxHashMap;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-unused-private-class-members): '{0}' is defined but never used.")]
#[diagnostic(severity(warning))]
struct NoUnusedPrivateClassMembersDiagnostic(Atom, #[label] pub Span);

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
    /// ### Example
    /// ```javascript
		/// 
		/// /// bad
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
		/// 
		/// /// Good
		/// class A {
		///		#usedMember = 42;
		///		method() {
		///				return this.#usedMember;
		///		}
		///	}

		///	class B {
		///			#usedMethod() {
		///					return 42;
		///			}
		///			anotherMethod() {
		///					return this.#usedMethod();
		///			}
		///	}

		///	class C {
		///			get #usedAccessor() {}
		///			set #usedAccessor(value) {}
		///			
		///			method() {
		///					this.#usedAccessor = 42;
		///			}
		///	}
		/// 
    /// ```
    NoUnusedPrivateClassMembers,
    correctness
);


impl Rule for NoUnusedPrivateClassMembers {
	fn run_once(&self, ctx: &LintContext) {
		// TODO: If we can tell that the ClassBody exits, we can improve its performance. refer https://github.com/eslint/eslint/blob/main/lib/rules/no-unused-private-class-members.js
			type Identifiers<'a> = FxHashMap<&'a Atom, (Span, usize, usize, bool)>;
        let mut identifiers_map: FxHashMap<AstNodeId,Identifiers> =
            FxHashMap::default();

        ctx.nodes().iter().for_each(|node| {
            let AstKind::PrivateIdentifier(identifier) = node.kind() else {
                return;
            };

            let Some(parent_node) = ctx.nodes().parent_node(node.id()) else {
                return;
            };

            let mut node_id = node.id();

            if let AstKind::PrivateFieldExpression(expr) = parent_node.kind() {
                // method(a) { return class { foo = a.#usedInInnerClass; }}
                // We need to find a node_id
                if let Expression::Identifier(ident) = &expr.object {
                    if let Some(reference) = ident
                        .reference_id
                        .get()
                        .map(|id| ctx.symbols().get_reference(id))
                    {
                        if let Some(symbol_id) = reference.symbol_id() {
                            node_id = ctx.symbols().get_declaration(symbol_id);
                        }
                    };
                }
            }

            let class_body_id = ctx
                .nodes()
                .ancestors(node_id)
                .find(|node_id| matches!(ctx.nodes().get_node(*node_id).kind(), AstKind::Class(_)));

            let Some(class_body_id) = class_body_id else {
                return;
            };

            let mut is_set_method = false;

            let is_write = ctx
                .nodes()
                // MemberExpression
                .parent_node(parent_node.id())
                .and_then(|node| {
                    // MethodDefinition
                    // set #xxx(x) get #xxx()
                    if let AstKind::MethodDefinition(definition) = node.kind() {
                        is_set_method = matches!(definition.kind, MethodDefinitionKind::Set);
                    }

                    ctx.nodes().parent_node(node.id())
                })
                .and_then(|node| {
                    if matches!(node.kind(), AstKind::PropertyKey(_)) {
                        return None;
                    }

                    ctx.nodes().parent_id(node.id())
                })
                .and_then(|id| ctx.nodes().parent_node(id))
                .is_some_and(|node| {
                    match node.kind() {
												// this.#accessorWithSetterFirst += 1;
                        AstKind::AssignmentExpression(expr) => {
                            if !matches!(expr.right, Expression::MemberExpression(_)) && matches!(
															ctx.nodes().parent_kind(node.id()),
															Some(AstKind::ExpressionStatement(_))
													) {
															 return true;
                            }
                        }
												//  for (this.#unusedForInLoop in bar)
                        AstKind::ForInStatement(stmt) => {
                            if !matches!(stmt.right, Expression::PrivateInExpression(_)) {
                                return true;
                            }
                        }
												// for (this.#unusedForOfLoop of bar) 
                        AstKind::ForOfStatement(stmt) => {
                            if !matches!(stmt.right, Expression::PrivateInExpression(_)) {
                                return true;
                            }
                        }
												// ({ x: this.#unusedInDestructuring } = bar);
												AstKind::AssignmentTargetPropertyProperty(_)
												// this.#usedOnlyInIncrement++;
												| AstKind::ExpressionStatement(_) 
                        // [...this.#unusedInRestPattern] = bar;
												
                        | AstKind::ArrayAssignmentTarget(_)
                        // [this.#unusedInAssignmentPattern = 1]
                        | AstKind::AssignmentTargetWithDefault(_) => {
                            return true;
                        }
                        _ => {}
                    }
                    false
                });

            let identifiers = identifiers_map.entry(class_body_id).or_default();

            identifiers
                .entry(&identifier.name)
                .and_modify(|(_, get, set, flag)| {
                    if is_write {
                        *set += 1;
                    } else {
                        *get += 1;
                    }
                    if is_set_method {
                        *flag = is_set_method;
                    }
                })
                .or_insert((identifier.span, 0, 0, is_set_method));
        });

        for identifiers in identifiers_map.into_values() {
					for (name, (span, get, set, flag)) in identifiers {
                if (flag && set == 0) || (!flag && get == 0) {
                    ctx.diagnostic(NoUnusedPrivateClassMembersDiagnostic(name.clone(), span));
                }
					};
        }
    }
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
        r"class C {
			    #usedInInnerClass;
			
			    method(a) {
			        return class {
			            foo = a.#usedInInnerClass;
			        }
			    }
			}",
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

    Tester::new_without_config(NoUnusedPrivateClassMembers::NAME, pass, fail).test_and_snapshot();
}
