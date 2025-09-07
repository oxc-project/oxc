use itertools::Itertools;
use oxc_ast::{AstKind, ast::AssignmentOperator};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNode, NodeId, Semantic};
use oxc_span::{GetSpan, Span};
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

                if element.is_private
                    && !ctx.classes().iter_private_identifiers(class_id).any(|ident| {
                        // If the element is a property, it must be read.
                        (!element.kind.is_property() || is_read(ident.id, ctx.semantic()))
                            && ident.element_ids.contains(&element_id)
                    })
                {
                    ctx.diagnostic(no_unused_private_class_members_diagnostic(
                        &element.name,
                        element.span,
                    ));
                }
            }
        });
    }
}

fn is_read(current_node_id: NodeId, semantic: &Semantic) -> bool {
    for (curr, parent) in semantic
        .nodes()
        .ancestors(current_node_id)
        .filter(|parent| {
            !matches!(
                parent.kind(),
                AstKind::ParenthesizedExpression(_)
                    | AstKind::TSAsExpression(_)
                    | AstKind::TSSatisfiesExpression(_)
                    | AstKind::TSInstantiationExpression(_)
                    | AstKind::TSNonNullExpression(_)
                    | AstKind::TSTypeAssertion(_)
            )
        })
        .tuple_windows::<(&AstNode<'_>, &AstNode<'_>)>()
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
            // All these are read contexts for private fields
            (AstKind::PrivateFieldExpression(_), _) if is_value_context(parent, semantic) => {
                return true;
            }
            // AssignmentExpression: right-hand side is a read, compound assignment result in value context is a read
            (AstKind::PrivateFieldExpression(_), AstKind::AssignmentExpression(assign_expr)) => {
                // Right-hand side of assignment is a read
                if assign_expr.right.span() == curr.span() {
                    return true;
                }
                // Compound assignment result used in a value context is a read
                if assign_expr.operator != AssignmentOperator::Assign
                    && is_compound_assignment_read(parent.id(), semantic)
                {
                    return true;
                }
                // Not a read otherwise
                return false;
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
            (
                AstKind::PrivateFieldExpression(_),
                AstKind::ConditionalExpression(conditional_expr),
            ) => {
                if conditional_expr.test.span() == curr.span() {
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

/// Check if the given AST kind represents a context where a value is being read/used
fn is_value_context(kind: &AstNode, semantic: &Semantic<'_>) -> bool {
    match kind.kind() {
        AstKind::ReturnStatement(_)
        | AstKind::CallExpression(_)
        | AstKind::BinaryExpression(_)
        | AstKind::VariableDeclarator(_)
        | AstKind::PropertyDefinition(_)
        | AstKind::ArrayExpression(_)
        | AstKind::ObjectProperty(_)
        | AstKind::JSXExpressionContainer(_)
        | AstKind::ChainExpression(_)
        | AstKind::StaticMemberExpression(_)
        | AstKind::ComputedMemberExpression(_)
        | AstKind::TemplateLiteral(_)
        | AstKind::UnaryExpression(_)
        | AstKind::IfStatement(_)
        | AstKind::SpreadElement(_)
        | AstKind::LogicalExpression(_) => true,
        AstKind::ExpressionStatement(_) => {
            let parent_node = semantic.nodes().parent_node(kind.id());
            if let AstKind::FunctionBody(_) = parent_node.kind()
                && let AstKind::ArrowFunctionExpression(arrow) =
                    semantic.nodes().parent_kind(parent_node.id())
                && arrow.expression
            {
                return true;
            }
            false
        }
        AstKind::ParenthesizedExpression(_)
        | AstKind::TSAsExpression(_)
        | AstKind::TSSatisfiesExpression(_)
        | AstKind::TSInstantiationExpression(_)
        | AstKind::TSNonNullExpression(_)
        | AstKind::TSTypeAssertion(_)
        | AstKind::UpdateExpression(_)
        | AstKind::AwaitExpression(_) => {
            is_value_context(semantic.nodes().parent_node(kind.id()), semantic)
        }

        _ => false,
    }
}

/// Check if a compound assignment result is being used in a value context
fn is_compound_assignment_read(parent_id: NodeId, semantic: &Semantic) -> bool {
    semantic
        .nodes()
        .ancestors(parent_id)
        .next()
        .is_some_and(|grandparent| is_value_context(grandparent, semantic))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"
            class Foo { #privateMember = {}; a() { return { ...this.#privateMember }; } }
        ",
        r"
            class Test {
                #prop = undefined

                getProp() {
                    return this.#prop ??= 0
                }
            }
        ",
        r"
            class Test {
                #prop = undefined

                getProp() {
                    return this.#prop ||= 0
                }
            }
        ",
        r"
            class Test {
                #prop = undefined

                getProp() {
                    return this.#prop += 0
                }
            }
        ",
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
        "class Foo { #a = false; on(data) { return this.#a ? [data] : data; } set setA(value) { this.#a = value; } }",
        "class Foo { #a = false; on(data) { return this.#a ? [data] : data; } }",
        "class WeakReference { #i = 0; inc() { return ++this.#i; }; dec() { return --this.#i; } }",
        "class Foo { #d; constructor(d) { this.#d = d || kDefaultD; } get getD(): string { return this.#d!; } }",
        "class F { #o; initialize(output) { this.#o = output; } text(e) { return this.#o!.text(e); } }",
        "class Foo { #a; constructor(a) { this.#a = a; }; b(b?: string): this { this.#a!.setB(b); return this; } resetA() { this.#a = undefined; } }",
        // Test for static block - issue #13179
        r"let getPrivate; class C { #private; constructor(v) { this.#private = v; } static { getPrivate = klass => klass.#private; } }",
        r"let getPrivate; class C { #private; constructor(v) { this.#private = v; } static { getPrivate = klass => { return klass.#private; } } }",
        r"class C { #field = 1; static { const obj = new C(); console.log(obj.#field); } }",
        r"class C { #method() { return 42; } static { const obj = new C(); obj.#method(); } }",
        r"class C { #field = 1; static { const getField = obj => { return obj.#field; }; } }",
        r"export class Database<const S extends idb.DBSchema> { readonly #db: Promise<idb.IDBPDatabase<S>>; constructor(name: string, version: number, hooks: idb.OpenDBCallbacks<S>) { this.#db = idb.openDB<S>(name, version, hooks); }  async read() { let db = await this.#db; } }",
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
        r"class Foo { #awaitedMember; async method() { await this.#awaitedMember; } }",
    ];

    Tester::new(NoUnusedPrivateClassMembers::NAME, NoUnusedPrivateClassMembers::PLUGIN, pass, fail)
        .test_and_snapshot();
}
