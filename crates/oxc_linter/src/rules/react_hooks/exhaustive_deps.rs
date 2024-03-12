use oxc_ast::{ast::CallExpression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{ast_util::is_method_call, context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("react-hooks(exhaustive-deps):")]
#[diagnostic(severity(warning), help(""))]
struct ExhaustiveDepsDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct ExhaustiveDeps;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    ExhaustiveDeps,
    correctness
);

impl Rule for ExhaustiveDeps {
    fn run_once(&self, ctx: &LintContext) {
        ctx.semantic().nodes().iter().for_each(|node| {
            if is_hook_call(node) {
                dbg!(ctx.semantic().scopes());
            }
        });
    }
}

// TODO: register vars in component scope?
// TODO: how to detect whether a func is a component?
// for each var access in hook, check if access is either:
// - valid (ref, const, etc)
// - listed as a dependency in dependencies array

// Check the declared dependencies for this reactive hook. If there is no
// second argument then the reactive callback will re-run on every render.
// So no need to check for dependency inclusion.

// struct ReactHookCall {
//     name: string,
// }

fn is_hook_call(node: &AstNode) -> bool {
    let AstKind::CallExpression(call_expr) = node.kind() else { return false };
    let Some(ident) = call_expr.callee.get_identifier_reference() else { return false };
    let func_arg = &call_expr.arguments[0];

    dbg!(func_arg);
    println!("function name {:?}", ident.name);
    true
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // r"function MyComponent() {
        //   const local = {};
        //   useEffect(() => {
        //     console.log(local);
        //   });
        // }",
        r"function MyComponent(props) {
            useCallback(() => {
              console.log(props.foo?.toString());
            }, [props.foo]);
          }",
    ];

    let fail = vec![
        r"function MyComponent(props) {
            useCallback(() => {
              console.log(props.foo?.toString());
            }, []);
          }",
    ];

    // let pass = vec![
    //     r"function MyComponent() {
    //       const local = {};
    //       useEffect(() => {
    //         console.log(local);
    //       });
    //     }",
    //     r"function MyComponent() {
    //       useEffect(() => {
    //         const local = {};
    //         console.log(local);
    //       }, []);
    //     }",
    //     r"function MyComponent() {
    //       const local = someFunc();
    //       useEffect(() => {
    //         console.log(local);
    //       }, [local]);
    //     }",
    //     r"function MyComponent() {
    //       useEffect(() => {
    //         console.log(props.foo);
    //       }, []);
    //     }",
    //     r"function MyComponent() {
    //       const local1 = {};
    //       {
    //         const local2 = {};
    //         useEffect(() => {
    //           console.log(local1);
    //           console.log(local2);
    //         });
    //       }
    //     }",
    //     r"function MyComponent() {
    //       const local1 = someFunc();
    //       {
    //         const local2 = someFunc();
    //         useCallback(() => {
    //           console.log(local1);
    //           console.log(local2);
    //         }, [local1, local2]);
    //       }
    //     }",
    //     r"function MyComponent() {
    //       const local1 = someFunc();
    //       function MyNestedComponent() {
    //         const local2 = someFunc();
    //         useCallback(() => {
    //           console.log(local1);
    //           console.log(local2);
    //         }, [local2]);
    //       }
    //     }",
    //     r"function MyComponent() {
    //       const local = someFunc();
    //       useEffect(() => {
    //         console.log(local);
    //         console.log(local);
    //       }, [local]);
    //     }",
    //     r"function MyComponent() {
    //       useEffect(() => {
    //         console.log(unresolved);
    //       }, []);
    //     }",
    //     r"function MyComponent() {
    //       const local = someFunc();
    //       useEffect(() => {
    //         console.log(local);
    //       }, [,,,local,,,]);
    //     }",
    //     r"function MyComponent({ foo }) {
    //       useEffect(() => {
    //         console.log(foo.length);
    //       }, [foo]);
    //     }",
    //     r"function MyComponent({ foo }) {
    //       useEffect(() => {
    //         console.log(foo.length);
    //         console.log(foo.slice(0));
    //       }, [foo]);
    //     }",
    //     r"function MyComponent({ history }) {
    //       useEffect(() => {
    //         return history.listen();
    //       }, [history]);
    //     }",
    //     r"function MyComponent(props) {
    //       useEffect(() => {});
    //       useLayoutEffect(() => {});
    //       useImperativeHandle(props.innerRef, () => {});
    //     }",
    //     r"function MyComponent(props) {
    //       useEffect(() => {
    //         console.log(props.foo);
    //       }, [props.foo]);
    //     }",
    //     r"function MyComponent(props) {
    //       useEffect(() => {
    //         console.log(props.foo);
    //         console.log(props.bar);
    //       }, [props.bar, props.foo]);
    //     }",
    //     r"function MyComponent(props) {
    //       useEffect(() => {
    //         console.log(props.foo);
    //         console.log(props.bar);
    //       }, [props.foo, props.bar]);
    //     }",
    //     r"function MyComponent(props) {
    //       const local = someFunc();
    //       useEffect(() => {
    //         console.log(props.foo);
    //         console.log(props.bar);
    //         console.log(local);
    //       }, [props.foo, props.bar, local]);
    //     }",
    //     r"function MyComponent(props) {
    //       const local = {};
    //       useEffect(() => {
    //         console.log(props.foo);
    //         console.log(props.bar);
    //       }, [props, props.foo]);

    //       let color = someFunc();
    //       useEffect(() => {
    //         console.log(props.foo.bar.baz);
    //         console.log(color);
    //       }, [props.foo, props.foo.bar.baz, color]);
    //     }",
    //     r"function MyComponent(props) {
    //       useEffect(() => {
    //         console.log(props.foo?.bar?.baz ?? null);
    //       }, [props.foo]);
    //     }",
    //     r"function MyComponent(props) {
    //       useEffect(() => {
    //         console.log(props.foo?.bar);
    //       }, [props.foo?.bar]);
    //     }",
    //     r"function MyComponent(props) {
    //       useEffect(() => {
    //         console.log(props.foo?.bar);
    //       }, [props.foo.bar]);
    //     }",
    //     r"function MyComponent(props) {
    //       useEffect(() => {
    //         console.log(props.foo.bar);
    //       }, [props.foo?.bar]);
    //     }",
    //     r"function MyComponent(props) {
    //       useEffect(() => {
    //         console.log(props.foo.bar);
    //         console.log(props.foo?.bar);
    //       }, [props.foo?.bar]);
    //     }",
    //     r"function MyComponent(props) {
    //       useEffect(() => {
    //         console.log(props.foo.bar);
    //         console.log(props.foo?.bar);
    //       }, [props.foo.bar]);
    //     }",
    //     r"function MyComponent(props) {
    //       useEffect(() => {
    //         console.log(props.foo);
    //         console.log(props.foo?.bar);
    //       }, [props.foo]);
    //     }",
    //     r"function MyComponent(props) {
    //       useEffect(() => {
    //         console.log(props.foo?.toString());
    //       }, [props.foo]);
    //     }",
    //     r"function MyComponent(props) {
    //       useMemo(() => {
    //         console.log(props.foo?.toString());
    //       }, [props.foo]);
    //     }",
    //     r"function MyComponent(props) {
    //       useCallback(() => {
    //         console.log(props.foo?.toString());
    //       }, [props.foo]);
    //     }",
    //     r"function MyComponent(props) {
    //       useCallback(() => {
    //         console.log(props.foo.bar?.toString());
    //       }, [props.foo.bar]);
    //     }",
    //     r"function MyComponent(props) {
    //       useCallback(() => {
    //         console.log(props.foo?.bar?.toString());
    //       }, [props.foo.bar]);
    //     }",
    //     r"function MyComponent(props) {
    //       useCallback(() => {
    //         console.log(props.foo.bar.toString());
    //       }, [props?.foo?.bar]);
    //     }",
    //     r"function MyComponent(props) {
    //       useCallback(() => {
    //         console.log(props.foo?.bar?.baz);
    //       }, [props?.foo.bar?.baz]);
    //     }",
    //     r"function MyComponent() {
    //       const myEffect = () => {
    //         // Doesn't use anything
    //       };
    //       useEffect(myEffect, []);
    //     }",
    //     r"const local = {};
    //     function MyComponent() {
    //       const myEffect = () => {
    //         console.log(local);
    //       };
    //       useEffect(myEffect, []);
    //     }",
    //     r"const local = {};
    //     function MyComponent() {
    //       function myEffect() {
    //         console.log(local);
    //       }
    //       useEffect(myEffect, []);
    //     }",
    //     r"function MyComponent() {
    //       const local = someFunc();
    //       function myEffect() {
    //         console.log(local);
    //       }
    //       useEffect(myEffect, [local]);
    //     }",
    //     r"function MyComponent() {
    //       function myEffect() {
    //         console.log(global);
    //       }
    //       useEffect(myEffect, []);
    //     }",
    //     r"const local = {};
    //     function MyComponent() {
    //       const myEffect = () => {
    //         otherThing()
    //       }
    //       const otherThing = () => {
    //         console.log(local);
    //       }
    //       useEffect(myEffect, []);
    //     }",
    //     r"function MyComponent({delay}) {
    //       const local = {};
    //       const myEffect = debounce(() => {
    //         console.log(local);
    //       }, delay);
    //       useEffect(myEffect, [myEffect]);
    //     }",
    //     r"function MyComponent({myEffect}) {
    //       useEffect(myEffect, [,myEffect]);
    //     }",
    //     r"function MyComponent({myEffect}) {
    //       useEffect(myEffect, [,myEffect,,]);
    //     }",
    //     r"let local = {};
    //     function myEffect() {
    //       console.log(local);
    //     }
    //     function MyComponent() {
    //       useEffect(myEffect, []);
    //     }",
    //     r"function MyComponent({myEffect}) {
    //       useEffect(myEffect, [myEffect]);
    //     }",
    //     r"function MyComponent({myEffect}) {
    //       useEffect(myEffect);
    //     }",
    //     r"function MyComponent(props) {
    //       useCustomEffect(() => {
    //         console.log(props.foo);
    //       });
    //     }",
    //     r"function MyComponent(props) {
    //       useCustomEffect(() => {
    //         console.log(props.foo);
    //       }, [props.foo]);
    //     }",
    //     r"function MyComponent(props) {
    //       useCustomEffect(() => {
    //         console.log(props.foo);
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       useWithoutEffectSuffix(() => {
    //         console.log(props.foo);
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       return renderHelperConfusedWithEffect(() => {
    //         console.log(props.foo);
    //       }, []);
    //     }",
    //     r"const local = {};
    //     useEffect(() => {
    //       console.log(local);
    //     }, []);",
    //     r"const local1 = {};
    //     {
    //       const local2 = {};
    //       useEffect(() => {
    //         console.log(local1);
    //         console.log(local2);
    //       }, []);
    //     }",
    //     r"function MyComponent() {
    //       const ref = useRef();
    //       useEffect(() => {
    //         console.log(ref.current);
    //       }, [ref]);
    //     }",
    //     r"function MyComponent() {
    //       const ref = useRef();
    //       useEffect(() => {
    //         console.log(ref.current);
    //       }, []);
    //     }",
    //     r"function MyComponent({ maybeRef2, foo }) {
    //       const definitelyRef1 = useRef();
    //       const definitelyRef2 = useRef();
    //       const maybeRef1 = useSomeOtherRefyThing();
    //       const [state1, setState1] = useState();
    //       const [state2, setState2] = React.useState();
    //       const [state3, dispatch1] = useReducer();
    //       const [state4, dispatch2] = React.useReducer();
    //       const [state5, maybeSetState] = useFunnyState();
    //       const [state6, maybeDispatch] = useFunnyReducer();
    //       const [isPending1] = useTransition();
    //       const [isPending2, startTransition2] = useTransition();
    //       const [isPending3] = React.useTransition();
    //       const [isPending4, startTransition4] = React.useTransition();
    //       const mySetState = useCallback(() => {}, []);
    //       let myDispatch = useCallback(() => {}, []);

    //       useEffect(() => {
    //         // Known to be static
    //         console.log(definitelyRef1.current);
    //         console.log(definitelyRef2.current);
    //         console.log(maybeRef1.current);
    //         console.log(maybeRef2.current);
    //         setState1();
    //         setState2();
    //         dispatch1();
    //         dispatch2();
    //         startTransition1();
    //         startTransition2();
    //         startTransition3();
    //         startTransition4();

    //         // Dynamic
    //         console.log(state1);
    //         console.log(state2);
    //         console.log(state3);
    //         console.log(state4);
    //         console.log(state5);
    //         console.log(state6);
    //         console.log(isPending2);
    //         console.log(isPending4);
    //         mySetState();
    //         myDispatch();

    //         // Not sure; assume dynamic
    //         maybeSetState();
    //         maybeDispatch();
    //       }, [
    //         // Dynamic
    //         state1, state2, state3, state4, state5, state6,
    //         maybeRef1, maybeRef2,
    //         isPending2, isPending4,

    //         // Not sure; assume dynamic
    //         mySetState, myDispatch,
    //         maybeSetState, maybeDispatch

    //         // In this test, we don't specify static deps.
    //         // That should be okay.
    //       ]);
    //     }",
    //     r"function MyComponent({ maybeRef2 }) {
    //       const definitelyRef1 = useRef();
    //       const definitelyRef2 = useRef();
    //       const maybeRef1 = useSomeOtherRefyThing();

    //       const [state1, setState1] = useState();
    //       const [state2, setState2] = React.useState();
    //       const [state3, dispatch1] = useReducer();
    //       const [state4, dispatch2] = React.useReducer();

    //       const [state5, maybeSetState] = useFunnyState();
    //       const [state6, maybeDispatch] = useFunnyReducer();

    //       const mySetState = useCallback(() => {}, []);
    //       let myDispatch = useCallback(() => {}, []);

    //       useEffect(() => {
    //         // Known to be static
    //         console.log(definitelyRef1.current);
    //         console.log(definitelyRef2.current);
    //         console.log(maybeRef1.current);
    //         console.log(maybeRef2.current);
    //         setState1();
    //         setState2();
    //         dispatch1();
    //         dispatch2();

    //         // Dynamic
    //         console.log(state1);
    //         console.log(state2);
    //         console.log(state3);
    //         console.log(state4);
    //         console.log(state5);
    //         console.log(state6);
    //         mySetState();
    //         myDispatch();

    //         // Not sure; assume dynamic
    //         maybeSetState();
    //         maybeDispatch();
    //       }, [
    //         // Dynamic
    //         state1, state2, state3, state4, state5, state6,
    //         maybeRef1, maybeRef2,

    //         // Not sure; assume dynamic
    //         mySetState, myDispatch,
    //         maybeSetState, maybeDispatch,

    //         // In this test, we specify static deps.
    //         // That should be okay too!
    //         definitelyRef1, definitelyRef2, setState1, setState2, dispatch1, dispatch2
    //       ]);
    //     }",
    //     r"const MyComponent = forwardRef((props, ref) => {
    //       useImperativeHandle(ref, () => ({
    //         focus() {
    //           alert(props.hello);
    //         }
    //       }))
    //     });",
    //     r"const MyComponent = forwardRef((props, ref) => {
    //       useImperativeHandle(ref, () => ({
    //         focus() {
    //           alert(props.hello);
    //         }
    //       }), [props.hello])
    //     });",
    //     r"function MyComponent(props) {
    //       let obj = someFunc();
    //       useEffect(() => {
    //         obj.foo = true;
    //       }, [obj]);
    //     }",
    //     r"function MyComponent(props) {
    //       let foo = {}
    //       useEffect(() => {
    //         foo.bar.baz = 43;
    //       }, [foo.bar]);
    //     }",
    //     r"function MyComponent() {
    //       const myRef = useRef();
    //       useEffect(() => {
    //         const handleMove = () => {};
    //         myRef.current = {};
    //         return () => {
    //           console.log(myRef.current.toString())
    //         };
    //       }, []);
    //       return <div />;
    //     }",
    //     r"function MyComponent() {
    //       const myRef = useRef();
    //       useEffect(() => {
    //         const handleMove = () => {};
    //         myRef.current = {};
    //         return () => {
    //           console.log(myRef?.current?.toString())
    //         };
    //       }, []);
    //       return <div />;
    //     }",
    //     r"function useMyThing(myRef) {
    //       useEffect(() => {
    //         const handleMove = () => {};
    //         myRef.current = {};
    //         return () => {
    //           console.log(myRef.current.toString())
    //         };
    //       }, [myRef]);
    //     }",
    //     r"function MyComponent() {
    //       const myRef = useRef();
    //       useEffect(() => {
    //         const handleMove = () => {};
    //         const node = myRef.current;
    //         node.addEventListener('mousemove', handleMove);
    //         return () => node.removeEventListener('mousemove', handleMove);
    //       }, []);
    //       return <div ref={myRef} />;
    //     }",
    //     r"function useMyThing(myRef) {
    //       useEffect(() => {
    //         const handleMove = () => {};
    //         const node = myRef.current;
    //         node.addEventListener('mousemove', handleMove);
    //         return () => node.removeEventListener('mousemove', handleMove);
    //       }, [myRef]);
    //       return <div ref={myRef} />;
    //     }",
    //     r"function useMyThing(myRef) {
    //       useCallback(() => {
    //         const handleMouse = () => {};
    //         myRef.current.addEventListener('mousemove', handleMouse);
    //         myRef.current.addEventListener('mousein', handleMouse);
    //         return function() {
    //           setTimeout(() => {
    //             myRef.current.removeEventListener('mousemove', handleMouse);
    //             myRef.current.removeEventListener('mousein', handleMouse);
    //           });
    //         }
    //       }, [myRef]);
    //     }",
    //     r"function useMyThing() {
    //       const myRef = useRef();
    //       useEffect(() => {
    //         const handleMove = () => {
    //           console.log(myRef.current)
    //         };
    //         window.addEventListener('mousemove', handleMove);
    //         return () => window.removeEventListener('mousemove', handleMove);
    //       }, []);
    //       return <div ref={myRef} />;
    //     }",
    //     r"function useMyThing() {
    //       const myRef = useRef();
    //       useEffect(() => {
    //         const handleMove = () => {
    //           return () => window.removeEventListener('mousemove', handleMove);
    //         };
    //         window.addEventListener('mousemove', handleMove);
    //         return () => {};
    //       }, []);
    //       return <div ref={myRef} />;
    //     }",
    //     r"function MyComponent() {
    //       const local1 = 42;
    //       const local2 = '42';
    //       const local3 = null;
    //       useEffect(() => {
    //         console.log(local1);
    //         console.log(local2);
    //         console.log(local3);
    //       }, []);
    //     }",
    //     r"function MyComponent() {
    //       const local1 = 42;
    //       const local2 = '42';
    //       const local3 = null;
    //       useEffect(() => {
    //         console.log(local1);
    //         console.log(local2);
    //         console.log(local3);
    //       }, [local1, local2, local3]);
    //     }",
    //     r"function MyComponent(props) {
    //       const local = props.local;
    //       useEffect(() => {}, [local]);
    //     }",
    //     r"function Foo({ activeTab }) {
    //       useEffect(() => {
    //         window.scrollTo(0, 0);
    //       }, [activeTab]);
    //     }",
    //     r"function MyComponent(props) {
    //       useEffect(() => {
    //         console.log(props.foo.bar.baz);
    //       }, [props]);
    //       useEffect(() => {
    //         console.log(props.foo.bar.baz);
    //       }, [props.foo]);
    //       useEffect(() => {
    //         console.log(props.foo.bar.baz);
    //       }, [props.foo.bar]);
    //       useEffect(() => {
    //         console.log(props.foo.bar.baz);
    //       }, [props.foo.bar.baz]);
    //     }",
    //     r"function MyComponent(props) {
    //       const fn = useCallback(() => {
    //         console.log(props.foo.bar.baz);
    //       }, [props]);
    //       const fn2 = useCallback(() => {
    //         console.log(props.foo.bar.baz);
    //       }, [props.foo]);
    //       const fn3 = useMemo(() => {
    //         console.log(props.foo.bar.baz);
    //       }, [props.foo.bar]);
    //       const fn4 = useMemo(() => {
    //         console.log(props.foo.bar.baz);
    //       }, [props.foo.bar.baz]);
    //     }",
    //     r"function MyComponent(props) {
    //       function handleNext1() {
    //         console.log('hello');
    //       }
    //       const handleNext2 = () => {
    //         console.log('hello');
    //       };
    //       let handleNext3 = function() {
    //         console.log('hello');
    //       };
    //       useEffect(() => {
    //         return Store.subscribe(handleNext1);
    //       }, []);
    //       useLayoutEffect(() => {
    //         return Store.subscribe(handleNext2);
    //       }, []);
    //       useMemo(() => {
    //         return Store.subscribe(handleNext3);
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       function handleNext() {
    //         console.log('hello');
    //       }
    //       useEffect(() => {
    //         return Store.subscribe(handleNext);
    //       }, []);
    //       useLayoutEffect(() => {
    //         return Store.subscribe(handleNext);
    //       }, []);
    //       useMemo(() => {
    //         return Store.subscribe(handleNext);
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       let [, setState] = useState();
    //       let [, dispatch] = React.useReducer();

    //       function handleNext1(value) {
    //         let value2 = value * 100;
    //         setState(value2);
    //         console.log('hello');
    //       }
    //       const handleNext2 = (value) => {
    //         setState(foo(value));
    //         console.log('hello');
    //       };
    //       let handleNext3 = function(value) {
    //         console.log(value);
    //         dispatch({ type: 'x', value });
    //       };
    //       useEffect(() => {
    //         return Store.subscribe(handleNext1);
    //       }, []);
    //       useLayoutEffect(() => {
    //         return Store.subscribe(handleNext2);
    //       }, []);
    //       useMemo(() => {
    //         return Store.subscribe(handleNext3);
    //       }, []);
    //     }",
    //     r"function useInterval(callback, delay) {
    //       const savedCallback = useRef();
    //       useEffect(() => {
    //         savedCallback.current = callback;
    //       });
    //       useEffect(() => {
    //         function tick() {
    //           savedCallback.current();
    //         }
    //         if (delay !== null) {
    //           let id = setInterval(tick, delay);
    //           return () => clearInterval(id);
    //         }
    //       }, [delay]);
    //     }",
    //     r"function Counter() {
    //       const [count, setCount] = useState(0);

    //       useEffect(() => {
    //         let id = setInterval(() => {
    //           setCount(c => c + 1);
    //         }, 1000);
    //         return () => clearInterval(id);
    //       }, []);

    //       return <h1>{count}</h1>;
    //     }",
    //     r"function Counter(unstableProp) {
    //       let [count, setCount] = useState(0);
    //       setCount = unstableProp
    //       useEffect(() => {
    //         let id = setInterval(() => {
    //           setCount(c => c + 1);
    //         }, 1000);
    //         return () => clearInterval(id);
    //       }, [setCount]);

    //       return <h1>{count}</h1>;
    //     }",
    //     r"function Counter() {
    //       const [count, setCount] = useState(0);

    //       function tick() {
    //         setCount(c => c + 1);
    //       }

    //       useEffect(() => {
    //         let id = setInterval(() => {
    //           tick();
    //         }, 1000);
    //         return () => clearInterval(id);
    //       }, []);

    //       return <h1>{count}</h1>;
    //     }",
    //     r"function Counter() {
    //       const [count, dispatch] = useReducer((state, action) => {
    //         if (action === 'inc') {
    //           return state + 1;
    //         }
    //       }, 0);

    //       useEffect(() => {
    //         let id = setInterval(() => {
    //           dispatch('inc');
    //         }, 1000);
    //         return () => clearInterval(id);
    //       }, []);

    //       return <h1>{count}</h1>;
    //     }",
    //     r"function Counter() {
    //       const [count, dispatch] = useReducer((state, action) => {
    //         if (action === 'inc') {
    //           return state + 1;
    //         }
    //       }, 0);

    //       const tick = () => {
    //         dispatch('inc');
    //       };

    //       useEffect(() => {
    //         let id = setInterval(tick, 1000);
    //         return () => clearInterval(id);
    //       }, []);

    //       return <h1>{count}</h1>;
    //     }",
    //     r"function Podcasts() {
    //       useEffect(() => {
    //         setPodcasts([]);
    //       }, []);
    //       let [podcasts, setPodcasts] = useState(null);
    //     }",
    //     r"function withFetch(fetchPodcasts) {
    //       return function Podcasts({ id }) {
    //         let [podcasts, setPodcasts] = useState(null);
    //         useEffect(() => {
    //           fetchPodcasts(id).then(setPodcasts);
    //         }, [id]);
    //       }
    //     }",
    //     r"function Podcasts({ id }) {
    //       let [podcasts, setPodcasts] = useState(null);
    //       useEffect(() => {
    //         function doFetch({ fetchPodcasts }) {
    //           fetchPodcasts(id).then(setPodcasts);
    //         }
    //         doFetch({ fetchPodcasts: API.fetchPodcasts });
    //       }, [id]);
    //     }",
    //     r"function Counter() {
    //       let [count, setCount] = useState(0);

    //       function increment(x) {
    //         return x + 1;
    //       }

    //       useEffect(() => {
    //         let id = setInterval(() => {
    //           setCount(increment);
    //         }, 1000);
    //         return () => clearInterval(id);
    //       }, []);

    //       return <h1>{count}</h1>;
    //     }",
    //     r"function Counter() {
    //       let [count, setCount] = useState(0);

    //       function increment(x) {
    //         return x + 1;
    //       }

    //       useEffect(() => {
    //         let id = setInterval(() => {
    //           setCount(count => increment(count));
    //         }, 1000);
    //         return () => clearInterval(id);
    //       }, []);

    //       return <h1>{count}</h1>;
    //     }",
    //     r"import increment from './increment';
    //     function Counter() {
    //       let [count, setCount] = useState(0);

    //       useEffect(() => {
    //         let id = setInterval(() => {
    //           setCount(count => count + increment);
    //         }, 1000);
    //         return () => clearInterval(id);
    //       }, []);

    //       return <h1>{count}</h1>;
    //     }",
    //     r"function withStuff(increment) {
    //       return function Counter() {
    //         let [count, setCount] = useState(0);

    //         useEffect(() => {
    //           let id = setInterval(() => {
    //             setCount(count => count + increment);
    //           }, 1000);
    //           return () => clearInterval(id);
    //         }, []);

    //         return <h1>{count}</h1>;
    //       }
    //     }",
    //     r"function App() {
    //       const [query, setQuery] = useState('react');
    //       const [state, setState] = useState(null);
    //       useEffect(() => {
    //         let ignore = false;
    //         fetchSomething();
    //         async function fetchSomething() {
    //           const result = await (await fetch('http://hn.algolia.com/api/v1/search?query=' + query)).json();
    //           if (!ignore) setState(result);
    //         }
    //         return () => { ignore = true; };
    //       }, [query]);
    //       return (
    //         <>
    //           <input value={query} onChange={e => setQuery(e.target.value)} />
    //           {JSON.stringify(state)}
    //         </>
    //       );
    //     }",
    //     r"function Example() {
    //       const foo = useCallback(() => {
    //         foo();
    //       }, []);
    //     }",
    //     r"function Example({ prop }) {
    //       const foo = useCallback(() => {
    //         if (prop) {
    //           foo();
    //         }
    //       }, [prop]);
    //     }",
    //     r"function Hello() {
    //       const [state, setState] = useState(0);
    //       useEffect(() => {
    //         const handleResize = () => setState(window.innerWidth);
    //         window.addEventListener('resize', handleResize);
    //         return () => window.removeEventListener('resize', handleResize);
    //       });
    //     }",
    //     r"function Example() {
    //       useEffect(() => {
    //         arguments
    //       }, [])
    //     }",
    //     r"function Example() {
    //       useEffect(() => {
    //         const bar = () => {
    //           arguments;
    //         };
    //         bar();
    //       }, [])
    //     }",
    //     r"function Example(props) {
    //       useEffect(() => {
    //         let topHeight = 0;
    //         topHeight = props.upperViewHeight;
    //       }, [props.upperViewHeight]);
    //     }",
    //     r"function Example(props) {
    //       useEffect(() => {
    //         let topHeight = 0;
    //         topHeight = props?.upperViewHeight;
    //       }, [props?.upperViewHeight]);
    //     }",
    //     r"function Example(props) {
    //       useEffect(() => {
    //         let topHeight = 0;
    //         topHeight = props?.upperViewHeight;
    //       }, [props]);
    //     }",
    //     r"function useFoo(foo){
    //       return useMemo(() => foo, [foo]);
    //     }",
    //     r"function useFoo(){
    //       const foo = 'hi!';
    //       return useMemo(() => foo, [foo]);
    //     }",
    //     r"function useFoo(){
    //       let {foo} = {foo: 1};
    //       return useMemo(() => foo, [foo]);
    //     }",
    //     r"function useFoo(){
    //       let [foo] = [1];
    //       return useMemo(() => foo, [foo]);
    //     }",
    //     r"function useFoo() {
    //       const foo = 'fine';
    //       if (true) {
    //         // Shadowed variable with constant construction in a nested scope is fine.
    //         const foo = {};
    //       }
    //       return useMemo(() => foo, [foo]);
    //     }",
    //     r"function MyComponent({foo}) {
    //       return useMemo(() => foo, [foo])
    //     }",
    //     r"function MyComponent() {
    //       const foo = true ? 'fine' : 'also fine';
    //       return useMemo(() => foo, [foo]);
    //     }",
    //     r"function MyComponent() {
    //       useEffect(() => {
    //         console.log('banana banana banana');
    //       }, undefined);
    //     }",
    // ];

    // let fail = vec![
    //     r"function MyComponent(props) {
    //       useCallback(() => {
    //         console.log(props.foo?.toString());
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       useCallback(() => {
    //         console.log(props.foo?.bar.baz);
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       useCallback(() => {
    //         console.log(props.foo?.bar?.baz);
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       useCallback(() => {
    //         console.log(props.foo?.bar.toString());
    //       }, []);
    //     }",
    //     r"function MyComponent() {
    //       const local = someFunc();
    //       useEffect(() => {
    //         console.log(local);
    //       }, []);
    //     }",
    //     r"function Counter(unstableProp) {
    //       let [count, setCount] = useState(0);
    //       setCount = unstableProp
    //       useEffect(() => {
    //         let id = setInterval(() => {
    //           setCount(c => c + 1);
    //         }, 1000);
    //         return () => clearInterval(id);
    //       }, []);

    //       return <h1>{count}</h1>;
    //     }",
    //     r"function MyComponent() {
    //       let local = 42;
    //       useEffect(() => {
    //         console.log(local);
    //       }, []);
    //     }",
    //     r"function MyComponent() {
    //       const local = /foo/;
    //       useEffect(() => {
    //         console.log(local);
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       const value = useMemo(() => { return 2*2; });
    //       const fn = useCallback(() => { alert('foo'); });
    //     }",
    //     r"function MyComponent({ fn1, fn2 }) {
    //       const value = useMemo(fn1);
    //       const fn = useCallback(fn2);
    //     }",
    //     r"function MyComponent() {
    //       useEffect()
    //       useLayoutEffect()
    //       useCallback()
    //       useMemo()
    //     }",
    //     r"function MyComponent() {
    //       const local = someFunc();
    //       useEffect(() => {
    //         if (true) {
    //           console.log(local);
    //         }
    //       }, []);
    //     }",
    //     r"function MyComponent() {
    //       const local = {};
    //       useEffect(() => {
    //         try {
    //           console.log(local);
    //         } finally {}
    //       }, []);
    //     }",
    //     r"function MyComponent() {
    //       const local = {};
    //       useEffect(() => {
    //         function inner() {
    //           console.log(local);
    //         }
    //         inner();
    //       }, []);
    //     }",
    //     r"function MyComponent() {
    //       const local1 = someFunc();
    //       {
    //         const local2 = someFunc();
    //         useEffect(() => {
    //           console.log(local1);
    //           console.log(local2);
    //         }, []);
    //       }
    //     }",
    //     r"function MyComponent() {
    //       const local1 = {};
    //       const local2 = {};
    //       useEffect(() => {
    //         console.log(local1);
    //         console.log(local2);
    //       }, [local1]);
    //     }",
    //     r"function MyComponent() {
    //       const local1 = {};
    //       const local2 = {};
    //       useMemo(() => {
    //         console.log(local1);
    //       }, [local1, local2]);
    //     }",
    //     r"function MyComponent() {
    //       const local1 = someFunc();
    //       function MyNestedComponent() {
    //         const local2 = {};
    //         useCallback(() => {
    //           console.log(local1);
    //           console.log(local2);
    //         }, [local1]);
    //       }
    //     }",
    //     r"function MyComponent() {
    //       const local = {};
    //       useEffect(() => {
    //         console.log(local);
    //         console.log(local);
    //       }, []);
    //     }",
    //     r"function MyComponent() {
    //       const local = {};
    //       useEffect(() => {
    //         console.log(local);
    //         console.log(local);
    //       }, [local, local]);
    //     }",
    //     r"function MyComponent() {
    //       useCallback(() => {}, [window]);
    //     }",
    //     r"function MyComponent(props) {
    //       let local = props.foo;
    //       useCallback(() => {}, [local]);
    //     }",
    //     r"function MyComponent({ history }) {
    //       useEffect(() => {
    //         return history.listen();
    //       }, []);
    //     }",
    //     r"function MyComponent({ history }) {
    //       useEffect(() => {
    //         return [
    //           history.foo.bar[2].dobedo.listen(),
    //           history.foo.bar().dobedo.listen[2]
    //         ];
    //       }, []);
    //     }",
    //     r"function MyComponent({ history }) {
    //       useEffect(() => {
    //         return [
    //           history?.foo
    //         ];
    //       }, []);
    //     }",
    //     r"function MyComponent() {
    //       useEffect(() => {}, ['foo']);
    //     }",
    //     r"function MyComponent({ foo, bar, baz }) {
    //       useEffect(() => {
    //         console.log(foo, bar, baz);
    //       }, ['foo', 'bar']);
    //     }",
    //     r"function MyComponent({ foo, bar, baz }) {
    //       useEffect(() => {
    //         console.log(foo, bar, baz);
    //       }, [42, false, null]);
    //     }",
    //     r"function MyComponent() {
    //       const dependencies = [];
    //       useEffect(() => {}, dependencies);
    //     }",
    //     r"function MyComponent() {
    //       const local = {};
    //       const dependencies = [local];
    //       useEffect(() => {
    //         console.log(local);
    //       }, dependencies);
    //     }",
    //     r"function MyComponent() {
    //       const local = {};
    //       const dependencies = [local];
    //       useEffect(() => {
    //         console.log(local);
    //       }, [...dependencies]);
    //     }",
    //     r"function MyComponent() {
    //       const local = someFunc();
    //       useEffect(() => {
    //         console.log(local);
    //       }, [local, ...dependencies]);
    //     }",
    //     r"function MyComponent() {
    //       const local = {};
    //       useEffect(() => {
    //         console.log(local);
    //       }, [computeCacheKey(local)]);
    //     }",
    //     r"function MyComponent(props) {
    //       useEffect(() => {
    //         console.log(props.items[0]);
    //       }, [props.items[0]]);
    //     }",
    //     r"function MyComponent(props) {
    //       useEffect(() => {
    //         console.log(props.items[0]);
    //       }, [props.items, props.items[0]]);
    //     }",
    //     r"function MyComponent({ items }) {
    //       useEffect(() => {
    //         console.log(items[0]);
    //       }, [items[0]]);
    //     }",
    //     r"function MyComponent({ items }) {
    //       useEffect(() => {
    //         console.log(items[0]);
    //       }, [items, items[0]]);
    //     }",
    //     r"function MyComponent(props) {
    //       const local = {};
    //       useCallback(() => {
    //         console.log(props.foo);
    //         console.log(props.bar);
    //       }, [props, props.foo]);
    //     }",
    //     r"function MyComponent(props) {
    //       const local = {};
    //       useCallback(() => {
    //         console.log(props.foo);
    //         console.log(props.bar);
    //       }, []);
    //     }",
    //     r"function MyComponent() {
    //       const local = {id: 42};
    //       useEffect(() => {
    //         console.log(local);
    //       }, [local.id]);
    //     }",
    //     r"function MyComponent() {
    //       const local = {id: 42};
    //       const fn = useCallback(() => {
    //         console.log(local);
    //       }, [local.id]);
    //     }",
    //     r"function MyComponent() {
    //       const local = {id: 42};
    //       const fn = useCallback(() => {
    //         console.log(local);
    //       }, [local.id, local]);
    //     }",
    //     r"function MyComponent(props) {
    //       const fn = useCallback(() => {
    //         console.log(props.foo.bar.baz);
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       let color = {}
    //       const fn = useCallback(() => {
    //         console.log(props.foo.bar.baz);
    //         console.log(color);
    //       }, [props.foo, props.foo.bar.baz]);
    //     }",
    //     r"function MyComponent(props) {
    //       const fn = useCallback(() => {
    //         console.log(props.foo.bar.baz);
    //       }, [props.foo.bar.baz, props.foo]);
    //     }",
    //     r"function MyComponent(props) {
    //       const fn = useCallback(() => {
    //         console.log(props.foo.bar.baz);
    //         console.log(props.foo.fizz.bizz);
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       const fn = useCallback(() => {
    //         console.log(props.foo.bar);
    //       }, [props.foo.bar.baz]);
    //     }",
    //     r"function MyComponent(props) {
    //       const fn = useCallback(() => {
    //         console.log(props);
    //         console.log(props.hello);
    //       }, [props.foo.bar.baz]);
    //     }",
    //     r"function MyComponent() {
    //       const local = {};
    //       useEffect(() => {
    //         console.log(local);
    //       }, [local, local]);
    //     }",
    //     r"function MyComponent() {
    //       const local1 = {};
    //       useCallback(() => {
    //         const local1 = {};
    //         console.log(local1);
    //       }, [local1]);
    //     }",
    //     r"function MyComponent() {
    //       const local1 = {};
    //       useCallback(() => {}, [local1]);
    //     }",
    //     r"function MyComponent(props) {
    //       useEffect(() => {
    //         console.log(props.foo);
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       useEffect(() => {
    //         console.log(props.foo);
    //         console.log(props.bar);
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       let a, b, c, d, e, f, g;
    //       useEffect(() => {
    //         console.log(b, e, d, c, a, g, f);
    //       }, [c, a, g]);
    //     }",
    //     r"function MyComponent(props) {
    //       let a, b, c, d, e, f, g;
    //       useEffect(() => {
    //         console.log(b, e, d, c, a, g, f);
    //       }, [a, c, g]);
    //     }",
    //     r"function MyComponent(props) {
    //       let a, b, c, d, e, f, g;
    //       useEffect(() => {
    //         console.log(b, e, d, c, a, g, f);
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       const local = {};
    //       useEffect(() => {
    //         console.log(props.foo);
    //         console.log(props.bar);
    //         console.log(local);
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       const local = {};
    //       useEffect(() => {
    //         console.log(props.foo);
    //         console.log(props.bar);
    //         console.log(local);
    //       }, [props]);
    //     }",
    //     r"function MyComponent(props) {
    //       useEffect(() => {
    //         console.log(props.foo);
    //       }, []);
    //       useCallback(() => {
    //         console.log(props.foo);
    //       }, []);
    //       useMemo(() => {
    //         console.log(props.foo);
    //       }, []);
    //       React.useEffect(() => {
    //         console.log(props.foo);
    //       }, []);
    //       React.useCallback(() => {
    //         console.log(props.foo);
    //       }, []);
    //       React.useMemo(() => {
    //         console.log(props.foo);
    //       }, []);
    //       React.notReactiveHook(() => {
    //         console.log(props.foo);
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       useCustomEffect(() => {
    //         console.log(props.foo);
    //       }, []);
    //       useEffect(() => {
    //         console.log(props.foo);
    //       }, []);
    //       React.useEffect(() => {
    //         console.log(props.foo);
    //       }, []);
    //       React.useCustomEffect(() => {
    //         console.log(props.foo);
    //       }, []);
    //     }",
    //     r"function MyComponent() {
    //       const local = {};
    //       useEffect(() => {
    //         console.log(local);
    //       }, [a ? local : b]);
    //     }",
    //     r"function MyComponent() {
    //       const local = {};
    //       useEffect(() => {
    //         console.log(local);
    //       }, [a && local]);
    //     }",
    //     r"function MyComponent(props) {
    //       useEffect(() => {}, [props?.attribute.method()]);
    //     }",
    //     r"function MyComponent(props) {
    //       useEffect(() => {}, [props.method()]);
    //     }",
    //     r"function MyComponent() {
    //       const ref = useRef();
    //       const [state, setState] = useState();
    //       useEffect(() => {
    //         ref.current = {};
    //         setState(state + 1);
    //       }, []);
    //     }",
    //     r"function MyComponent() {
    //       const ref = useRef();
    //       const [state, setState] = useState();
    //       useEffect(() => {
    //         ref.current = {};
    //         setState(state + 1);
    //       }, [ref]);
    //     }",
    //     r"function MyComponent(props) {
    //       const ref1 = useRef();
    //       const ref2 = useRef();
    //       useEffect(() => {
    //         ref1.current.focus();
    //         console.log(ref2.current.textContent);
    //         alert(props.someOtherRefs.current.innerHTML);
    //         fetch(props.color);
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       const ref1 = useRef();
    //       const ref2 = useRef();
    //       useEffect(() => {
    //         ref1.current.focus();
    //         console.log(ref2.current.textContent);
    //         alert(props.someOtherRefs.current.innerHTML);
    //         fetch(props.color);
    //       }, [ref1.current, ref2.current, props.someOtherRefs, props.color]);
    //     }",
    //     r"function MyComponent(props) {
    //       const ref1 = useRef();
    //       const ref2 = useRef();
    //       useEffect(() => {
    //         ref1?.current?.focus();
    //         console.log(ref2?.current?.textContent);
    //         alert(props.someOtherRefs.current.innerHTML);
    //         fetch(props.color);
    //       }, [ref1?.current, ref2?.current, props.someOtherRefs, props.color]);
    //     }",
    //     r"function MyComponent() {
    //       const ref = useRef();
    //       useEffect(() => {
    //         console.log(ref.current);
    //       }, [ref.current]);
    //     }",
    //     r"function MyComponent({ activeTab }) {
    //       const ref1 = useRef();
    //       const ref2 = useRef();
    //       useEffect(() => {
    //         ref1.current.scrollTop = 0;
    //         ref2.current.scrollTop = 0;
    //       }, [ref1.current, ref2.current, activeTab]);
    //     }",
    //     r"function MyComponent({ activeTab, initY }) {
    //       const ref1 = useRef();
    //       const ref2 = useRef();
    //       const fn = useCallback(() => {
    //         ref1.current.scrollTop = initY;
    //         ref2.current.scrollTop = initY;
    //       }, [ref1.current, ref2.current, activeTab, initY]);
    //     }",
    //     r"function MyComponent() {
    //       const ref = useRef();
    //       useEffect(() => {
    //         console.log(ref.current);
    //       }, [ref.current, ref]);
    //     }",
    //     r"const MyComponent = forwardRef((props, ref) => {
    //       useImperativeHandle(ref, () => ({
    //         focus() {
    //           alert(props.hello);
    //         }
    //       }), [])
    //     });",
    //     r"function MyComponent(props) {
    //       useEffect(() => {
    //         if (props.onChange) {
    //           props.onChange();
    //         }
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       useEffect(() => {
    //         if (props?.onChange) {
    //           props?.onChange();
    //         }
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       useEffect(() => {
    //         function play() {
    //           props.onPlay();
    //         }
    //         function pause() {
    //           props.onPause();
    //         }
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       useEffect(() => {
    //         if (props.foo.onChange) {
    //           props.foo.onChange();
    //         }
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       useEffect(() => {
    //         props.onChange();
    //         if (props.foo.onChange) {
    //           props.foo.onChange();
    //         }
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       const [skillsCount] = useState();
    //       useEffect(() => {
    //         if (skillsCount === 0 && !props.isEditMode) {
    //           props.toggleEditMode();
    //         }
    //       }, [skillsCount, props.isEditMode, props.toggleEditMode]);
    //     }",
    //     r"function MyComponent(props) {
    //       const [skillsCount] = useState();
    //       useEffect(() => {
    //         if (skillsCount === 0 && !props.isEditMode) {
    //           props.toggleEditMode();
    //         }
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       useEffect(() => {
    //         externalCall(props);
    //         props.onChange();
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       useEffect(() => {
    //         props.onChange();
    //         externalCall(props);
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       let value;
    //       let value2;
    //       let value3;
    //       let value4;
    //       let asyncValue;
    //       useEffect(() => {
    //         if (value4) {
    //           value = {};
    //         }
    //         value2 = 100;
    //         value = 43;
    //         value4 = true;
    //         console.log(value2);
    //         console.log(value3);
    //         setTimeout(() => {
    //           asyncValue = 100;
    //         });
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       let value;
    //       let value2;
    //       let value3;
    //       let asyncValue;
    //       useEffect(() => {
    //         value = {};
    //         value2 = 100;
    //         value = 43;
    //         console.log(value2);
    //         console.log(value3);
    //         setTimeout(() => {
    //           asyncValue = 100;
    //         });
    //       }, [value, value2, value3]);
    //     }",
    //     r"function MyComponent() {
    //       const myRef = useRef();
    //       useEffect(() => {
    //         const handleMove = () => {};
    //         myRef.current.addEventListener('mousemove', handleMove);
    //         return () => myRef.current.removeEventListener('mousemove', handleMove);
    //       }, []);
    //       return <div ref={myRef} />;
    //     }",
    //     r"function MyComponent() {
    //       const myRef = useRef();
    //       useEffect(() => {
    //         const handleMove = () => {};
    //         myRef?.current?.addEventListener('mousemove', handleMove);
    //         return () => myRef?.current?.removeEventListener('mousemove', handleMove);
    //       }, []);
    //       return <div ref={myRef} />;
    //     }",
    //     r"function MyComponent() {
    //       const myRef = useRef();
    //       useEffect(() => {
    //         const handleMove = () => {};
    //         myRef.current.addEventListener('mousemove', handleMove);
    //         return () => myRef.current.removeEventListener('mousemove', handleMove);
    //       });
    //       return <div ref={myRef} />;
    //     }",
    //     r"function useMyThing(myRef) {
    //       useEffect(() => {
    //         const handleMove = () => {};
    //         myRef.current.addEventListener('mousemove', handleMove);
    //         return () => myRef.current.removeEventListener('mousemove', handleMove);
    //       }, [myRef]);
    //     }",
    //     r"function useMyThing(myRef) {
    //       useEffect(() => {
    //         const handleMouse = () => {};
    //         myRef.current.addEventListener('mousemove', handleMouse);
    //         myRef.current.addEventListener('mousein', handleMouse);
    //         return function() {
    //           setTimeout(() => {
    //             myRef.current.removeEventListener('mousemove', handleMouse);
    //             myRef.current.removeEventListener('mousein', handleMouse);
    //           });
    //         }
    //       }, [myRef]);
    //     }",
    //     r"function useMyThing(myRef, active) {
    //       useEffect(() => {
    //         const handleMove = () => {};
    //         if (active) {
    //           myRef.current.addEventListener('mousemove', handleMove);
    //           return function() {
    //             setTimeout(() => {
    //               myRef.current.removeEventListener('mousemove', handleMove);
    //             });
    //           }
    //         }
    //       }, [myRef, active]);
    //     }",
    //     r"function MyComponent() {
    //               const myRef = useRef();
    //               useLayoutEffect_SAFE_FOR_SSR(() => {
    //                 const handleMove = () => {};
    //                 myRef.current.addEventListener('mousemove', handleMove);
    //                 return () => myRef.current.removeEventListener('mousemove', handleMove);
    //               });
    //               return <div ref={myRef} />;
    //             }",
    //     r"function MyComponent() {
    //       const local1 = 42;
    //       const local2 = '42';
    //       const local3 = null;
    //       const local4 = {};
    //       useEffect(() => {
    //         console.log(local1);
    //         console.log(local2);
    //         console.log(local3);
    //         console.log(local4);
    //       }, [local1, local3]);
    //     }",
    //     r"function MyComponent() {
    //       useEffect(() => {
    //         window.scrollTo(0, 0);
    //       }, [window]);
    //     }",
    //     r"import MutableStore from 'store';

    //     function MyComponent() {
    //       useEffect(() => {
    //         console.log(MutableStore.hello);
    //       }, [MutableStore.hello]);
    //     }",
    //     r"import MutableStore from 'store';
    //     let z = {};

    //     function MyComponent(props) {
    //       let x = props.foo;
    //       {
    //         let y = props.bar;
    //         useEffect(() => {
    //           console.log(MutableStore.hello.world, props.foo, x, y, z, global.stuff);
    //         }, [MutableStore.hello.world, props.foo, x, y, z, global.stuff]);
    //       }
    //     }",
    //     r"import MutableStore from 'store';
    //     let z = {};

    //     function MyComponent(props) {
    //       let x = props.foo;
    //       {
    //         let y = props.bar;
    //         useEffect(() => {
    //           // nothing
    //         }, [MutableStore.hello.world, props.foo, x, y, z, global.stuff]);
    //       }
    //     }",
    //     r"import MutableStore from 'store';
    //     let z = {};

    //     function MyComponent(props) {
    //       let x = props.foo;
    //       {
    //         let y = props.bar;
    //         const fn = useCallback(() => {
    //           // nothing
    //         }, [MutableStore.hello.world, props.foo, x, y, z, global.stuff]);
    //       }
    //     }",
    //     r"import MutableStore from 'store';
    //     let z = {};

    //     function MyComponent(props) {
    //       let x = props.foo;
    //       {
    //         let y = props.bar;
    //         const fn = useCallback(() => {
    //           // nothing
    //         }, [MutableStore?.hello?.world, props.foo, x, y, z, global?.stuff]);
    //       }
    //     }",
    //     r"function MyComponent(props) {
    //       let [, setState] = useState();
    //       let [, dispatch] = React.useReducer();
    //       let taint = props.foo;

    //       function handleNext1(value) {
    //         let value2 = value * taint;
    //         setState(value2);
    //         console.log('hello');
    //       }
    //       const handleNext2 = (value) => {
    //         setState(taint(value));
    //         console.log('hello');
    //       };
    //       let handleNext3 = function(value) {
    //         setTimeout(() => console.log(taint));
    //         dispatch({ type: 'x', value });
    //       };
    //       useEffect(() => {
    //         return Store.subscribe(handleNext1);
    //       }, []);
    //       useLayoutEffect(() => {
    //         return Store.subscribe(handleNext2);
    //       }, []);
    //       useMemo(() => {
    //         return Store.subscribe(handleNext3);
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       let [, setState] = useState();
    //       let [, dispatch] = React.useReducer();
    //       let taint = props.foo;

    //       // Shouldn't affect anything
    //       function handleChange() {}

    //       function handleNext1(value) {
    //         let value2 = value * taint;
    //         setState(value2);
    //         console.log('hello');
    //       }
    //       const handleNext2 = (value) => {
    //         setState(taint(value));
    //         console.log('hello');
    //       };
    //       let handleNext3 = function(value) {
    //         console.log(taint);
    //         dispatch({ type: 'x', value });
    //       };
    //       useEffect(() => {
    //         return Store.subscribe(handleNext1);
    //       }, []);
    //       useLayoutEffect(() => {
    //         return Store.subscribe(handleNext2);
    //       }, []);
    //       useMemo(() => {
    //         return Store.subscribe(handleNext3);
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       let [, setState] = useState();
    //       let [, dispatch] = React.useReducer();
    //       let taint = props.foo;

    //       // Shouldn't affect anything
    //       const handleChange = () => {};

    //       function handleNext1(value) {
    //         let value2 = value * taint;
    //         setState(value2);
    //         console.log('hello');
    //       }
    //       const handleNext2 = (value) => {
    //         setState(taint(value));
    //         console.log('hello');
    //       };
    //       let handleNext3 = function(value) {
    //         console.log(taint);
    //         dispatch({ type: 'x', value });
    //       };
    //       useEffect(() => {
    //         return Store.subscribe(handleNext1);
    //       }, []);
    //       useLayoutEffect(() => {
    //         return Store.subscribe(handleNext2);
    //       }, []);
    //       useMemo(() => {
    //         return Store.subscribe(handleNext3);
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       let [, setState] = useState();

    //       function handleNext(value) {
    //         setState(value);
    //       }

    //       useEffect(() => {
    //         return Store.subscribe(handleNext);
    //       }, [handleNext]);
    //     }",
    //     r"function MyComponent(props) {
    //       let [, setState] = useState();

    //       const handleNext = (value) => {
    //         setState(value);
    //       };

    //       useEffect(() => {
    //         return Store.subscribe(handleNext);
    //       }, [handleNext]);
    //     }",
    //     r"function MyComponent(props) {
    //       let [, setState] = useState();

    //       const handleNext = (value) => {
    //         setState(value);
    //       };

    //       useEffect(() => {
    //         return Store.subscribe(handleNext);
    //       }, [handleNext]);

    //       return <div onClick={handleNext} />;
    //     }",
    //     r"function MyComponent(props) {
    //       function handleNext1() {
    //         console.log('hello');
    //       }
    //       const handleNext2 = () => {
    //         console.log('hello');
    //       };
    //       let handleNext3 = function() {
    //         console.log('hello');
    //       };
    //       useEffect(() => {
    //         return Store.subscribe(handleNext1);
    //       }, [handleNext1]);
    //       useLayoutEffect(() => {
    //         return Store.subscribe(handleNext2);
    //       }, [handleNext2]);
    //       useMemo(() => {
    //         return Store.subscribe(handleNext3);
    //       }, [handleNext3]);
    //     }",
    //     r"function MyComponent(props) {
    //       function handleNext1() {
    //         console.log('hello');
    //       }
    //       const handleNext2 = () => {
    //         console.log('hello');
    //       };
    //       let handleNext3 = function() {
    //         console.log('hello');
    //       };
    //       useEffect(() => {
    //         handleNext1();
    //         return Store.subscribe(() => handleNext1());
    //       }, [handleNext1]);
    //       useLayoutEffect(() => {
    //         handleNext2();
    //         return Store.subscribe(() => handleNext2());
    //       }, [handleNext2]);
    //       useMemo(() => {
    //         handleNext3();
    //         return Store.subscribe(() => handleNext3());
    //       }, [handleNext3]);
    //     }",
    //     r"function MyComponent(props) {
    //       function handleNext1() {
    //         console.log('hello');
    //       }
    //       const handleNext2 = () => {
    //         console.log('hello');
    //       };
    //       let handleNext3 = function() {
    //         console.log('hello');
    //       };
    //       useEffect(() => {
    //         handleNext1();
    //         return Store.subscribe(() => handleNext1());
    //       }, [handleNext1]);
    //       useLayoutEffect(() => {
    //         handleNext2();
    //         return Store.subscribe(() => handleNext2());
    //       }, [handleNext2]);
    //       useMemo(() => {
    //         handleNext3();
    //         return Store.subscribe(() => handleNext3());
    //       }, [handleNext3]);
    //       return (
    //         <div
    //           onClick={() => {
    //             handleNext1();
    //             setTimeout(handleNext2);
    //             setTimeout(() => {
    //               handleNext3();
    //             });
    //           }}
    //         />
    //       );
    //     }",
    //     r"function MyComponent(props) {
    //       const handleNext1 = () => {
    //         console.log('hello');
    //       };
    //       function handleNext2() {
    //         console.log('hello');
    //       }
    //       useEffect(() => {
    //         return Store.subscribe(handleNext1);
    //         return Store.subscribe(handleNext2);
    //       }, [handleNext1, handleNext2]);
    //       useEffect(() => {
    //         return Store.subscribe(handleNext1);
    //         return Store.subscribe(handleNext2);
    //       }, [handleNext1, handleNext2]);
    //     }",
    //     r"function MyComponent(props) {
    //       let handleNext = () => {
    //         console.log('hello');
    //       };
    //       if (props.foo) {
    //         handleNext = () => {
    //           console.log('hello');
    //         };
    //       }
    //       useEffect(() => {
    //         return Store.subscribe(handleNext);
    //       }, [handleNext]);
    //     }",
    //     r"function MyComponent(props) {
    //       let [, setState] = useState();
    //       let taint = props.foo;

    //       function handleNext(value) {
    //         let value2 = value * taint;
    //         setState(value2);
    //         console.log('hello');
    //       }

    //       useEffect(() => {
    //         return Store.subscribe(handleNext);
    //       }, [handleNext]);
    //     }",
    //     r"function Counter() {
    //       let [count, setCount] = useState(0);

    //       useEffect(() => {
    //         let id = setInterval(() => {
    //           setCount(count + 1);
    //         }, 1000);
    //         return () => clearInterval(id);
    //       }, []);

    //       return <h1>{count}</h1>;
    //     }",
    //     r"function Counter() {
    //       let [count, setCount] = useState(0);
    //       let [increment, setIncrement] = useState(0);

    //       useEffect(() => {
    //         let id = setInterval(() => {
    //           setCount(count + increment);
    //         }, 1000);
    //         return () => clearInterval(id);
    //       }, []);

    //       return <h1>{count}</h1>;
    //     }",
    //     r"function Counter() {
    //       let [count, setCount] = useState(0);
    //       let [increment, setIncrement] = useState(0);

    //       useEffect(() => {
    //         let id = setInterval(() => {
    //           setCount(count => count + increment);
    //         }, 1000);
    //         return () => clearInterval(id);
    //       }, []);

    //       return <h1>{count}</h1>;
    //     }",
    //     r"function Counter() {
    //       let [count, setCount] = useState(0);
    //       let increment = useCustomHook();

    //       useEffect(() => {
    //         let id = setInterval(() => {
    //           setCount(count => count + increment);
    //         }, 1000);
    //         return () => clearInterval(id);
    //       }, []);

    //       return <h1>{count}</h1>;
    //     }",
    //     r"function Counter({ step }) {
    //       let [count, setCount] = useState(0);

    //       function increment(x) {
    //         return x + step;
    //       }

    //       useEffect(() => {
    //         let id = setInterval(() => {
    //           setCount(count => increment(count));
    //         }, 1000);
    //         return () => clearInterval(id);
    //       }, []);

    //       return <h1>{count}</h1>;
    //     }",
    //     r"function Counter({ step }) {
    //       let [count, setCount] = useState(0);

    //       function increment(x) {
    //         return x + step;
    //       }

    //       useEffect(() => {
    //         let id = setInterval(() => {
    //           setCount(count => increment(count));
    //         }, 1000);
    //         return () => clearInterval(id);
    //       }, [increment]);

    //       return <h1>{count}</h1>;
    //     }",
    //     r"function Counter({ increment }) {
    //       let [count, setCount] = useState(0);

    //       useEffect(() => {
    //         let id = setInterval(() => {
    //           setCount(count => count + increment);
    //         }, 1000);
    //         return () => clearInterval(id);
    //       }, []);

    //       return <h1>{count}</h1>;
    //     }",
    //     r"function Counter() {
    //       const [count, setCount] = useState(0);

    //       function tick() {
    //         setCount(count + 1);
    //       }

    //       useEffect(() => {
    //         let id = setInterval(() => {
    //           tick();
    //         }, 1000);
    //         return () => clearInterval(id);
    //       }, []);

    //       return <h1>{count}</h1>;
    //     }",
    //     r"function Podcasts() {
    //       useEffect(() => {
    //         alert(podcasts);
    //       }, []);
    //       let [podcasts, setPodcasts] = useState(null);
    //     }",
    //     r"function Podcasts({ fetchPodcasts, id }) {
    //       let [podcasts, setPodcasts] = useState(null);
    //       useEffect(() => {
    //         fetchPodcasts(id).then(setPodcasts);
    //       }, [id]);
    //     }",
    //     r"function Podcasts({ api: { fetchPodcasts }, id }) {
    //       let [podcasts, setPodcasts] = useState(null);
    //       useEffect(() => {
    //         fetchPodcasts(id).then(setPodcasts);
    //       }, [id]);
    //     }",
    //     r"function Podcasts({ fetchPodcasts, fetchPodcasts2, id }) {
    //       let [podcasts, setPodcasts] = useState(null);
    //       useEffect(() => {
    //         setTimeout(() => {
    //           console.log(id);
    //           fetchPodcasts(id).then(setPodcasts);
    //           fetchPodcasts2(id).then(setPodcasts);
    //         });
    //       }, [id]);
    //     }",
    //     r"function Podcasts({ fetchPodcasts, id }) {
    //       let [podcasts, setPodcasts] = useState(null);
    //       useEffect(() => {
    //         console.log(fetchPodcasts);
    //         fetchPodcasts(id).then(setPodcasts);
    //       }, [id]);
    //     }",
    //     r"function Podcasts({ fetchPodcasts, id }) {
    //       let [podcasts, setPodcasts] = useState(null);
    //       useEffect(() => {
    //         console.log(fetchPodcasts);
    //         fetchPodcasts?.(id).then(setPodcasts);
    //       }, [id]);
    //     }",
    //     r"function Thing() {
    //       useEffect(() => {
    //         const fetchData = async () => {};
    //         fetchData();
    //       }, [fetchData]);
    //     }",
    //     r"function Hello() {
    //       const [state, setState] = useState(0);
    //       useEffect(() => {
    //         setState({});
    //       });
    //     }",
    //     r"function Hello() {
    //       const [data, setData] = useState(0);
    //       useEffect(() => {
    //         fetchData.then(setData);
    //       });
    //     }",
    //     r"function Hello({ country }) {
    //       const [data, setData] = useState(0);
    //       useEffect(() => {
    //         fetchData(country).then(setData);
    //       });
    //     }",
    //     r"function Hello({ prop1, prop2 }) {
    //       const [state, setState] = useState(0);
    //       useEffect(() => {
    //         if (prop1) {
    //           setState(prop2);
    //         }
    //       });
    //     }",
    //     r"function Thing() {
    //       useEffect(async () => {}, []);
    //     }",
    //     r"function Thing() {
    //       useEffect(async () => {});
    //     }",
    //     r"function Example() {
    //       const foo = useCallback(() => {
    //         foo();
    //       }, [foo]);
    //     }",
    //     r"function Example({ prop }) {
    //       const foo = useCallback(() => {
    //         prop.hello(foo);
    //       }, [foo]);
    //       const bar = useCallback(() => {
    //         foo();
    //       }, [foo]);
    //     }",
    //     r"function MyComponent() {
    //       const local = {};
    //       function myEffect() {
    //         console.log(local);
    //       }
    //       useEffect(myEffect, []);
    //     }",
    //     r"function MyComponent() {
    //       const local = {};
    //       const myEffect = () => {
    //         console.log(local);
    //       };
    //       useEffect(myEffect, []);
    //     }",
    //     r"function MyComponent() {
    //       const local = {};
    //       const myEffect = function() {
    //         console.log(local);
    //       };
    //       useEffect(myEffect, []);
    //     }",
    //     r"function MyComponent() {
    //       const local = {};
    //       const myEffect = () => {
    //         otherThing();
    //       };
    //       const otherThing = () => {
    //         console.log(local);
    //       };
    //       useEffect(myEffect, []);
    //     }",
    //     r"function MyComponent() {
    //       const local = {};
    //       const myEffect = debounce(() => {
    //         console.log(local);
    //       }, delay);
    //       useEffect(myEffect, []);
    //     }",
    //     r"function MyComponent() {
    //       const local = {};
    //       const myEffect = debounce(() => {
    //         console.log(local);
    //       }, delay);
    //       useEffect(myEffect, [local]);
    //     }",
    //     r"function MyComponent({myEffect}) {
    //       useEffect(myEffect, []);
    //     }",
    //     r"function MyComponent() {
    //       const local = {};
    //       useEffect(debounce(() => {
    //         console.log(local);
    //       }, delay), []);
    //     }",
    //     r"function MyComponent() {
    //       const local = {};
    //       useEffect(() => {
    //         console.log(local);
    //       }, []);
    //     }",
    //     r"function MyComponent(props) {
    //       let foo = {}
    //       useEffect(() => {
    //         foo.bar.baz = 43;
    //         props.foo.bar.baz = 1;
    //       }, []);
    //     }",
    //     r"function Component() {
    //       const foo = {};
    //       useMemo(() => foo, [foo]);
    //     }",
    //     r"function Component() {
    //       const foo = [];
    //       useMemo(() => foo, [foo]);
    //     }",
    //     r"function Component() {
    //       const foo = () => {};
    //       useMemo(() => foo, [foo]);
    //     }",
    //     r"function Component() {
    //       const foo = function bar(){};
    //       useMemo(() => foo, [foo]);
    //     }",
    //     r"function Component() {
    //       const foo = class {};
    //       useMemo(() => foo, [foo]);
    //     }",
    //     r"function Component() {
    //       const foo = true ? {} : 'fine';
    //       useMemo(() => foo, [foo]);
    //     }",
    //     r"function Component() {
    //       const foo = bar || {};
    //       useMemo(() => foo, [foo]);
    //     }",
    //     r"function Component() {
    //       const foo = bar ?? {};
    //       useMemo(() => foo, [foo]);
    //     }",
    //     r"function Component() {
    //       const foo = bar && {};
    //       useMemo(() => foo, [foo]);
    //     }",
    //     r"function Component() {
    //       const foo = bar ? baz ? {} : null : null;
    //       useMemo(() => foo, [foo]);
    //     }",
    //     r"function Component() {
    //       let foo = {};
    //       useMemo(() => foo, [foo]);
    //     }",
    //     r"function Component() {
    //       var foo = {};
    //       useMemo(() => foo, [foo]);
    //     }",
    //     r"function Component() {
    //       const foo = {};
    //       useCallback(() => {
    //         console.log(foo);
    //       }, [foo]);
    //     }",
    //     r"function Component() {
    //       const foo = {};
    //       useEffect(() => {
    //         console.log(foo);
    //       }, [foo]);
    //     }",
    //     r"function Component() {
    //       const foo = {};
    //       useLayoutEffect(() => {
    //         console.log(foo);
    //       }, [foo]);
    //     }",
    //     r"function Component() {
    //       const foo = {};
    //       useImperativeHandle(
    //         ref,
    //         () => {
    //            console.log(foo);
    //         },
    //         [foo]
    //       );
    //     }",
    //     r"function Foo(section) {
    //       const foo = section.section_components?.edges ?? [];
    //       useEffect(() => {
    //         console.log(foo);
    //       }, [foo]);
    //     }",
    //     r"function Foo(section) {
    //       const foo = {};
    //       console.log(foo);
    //       useMemo(() => {
    //         console.log(foo);
    //       }, [foo]);
    //     }",
    //     r"function Foo() {
    //       const foo = <>Hi!</>;
    //       useMemo(() => {
    //         console.log(foo);
    //       }, [foo]);
    //     }",
    //     r"function Foo() {
    //       const foo = <div>Hi!</div>;
    //       useMemo(() => {
    //         console.log(foo);
    //       }, [foo]);
    //     }",
    //     r"function Foo() {
    //       const foo = bar = {};
    //       useMemo(() => {
    //         console.log(foo);
    //       }, [foo]);
    //     }",
    //     r"function Foo() {
    //       const foo = new String('foo'); // Note 'foo' will be boxed, and thus an object and thus compared by reference.
    //       useMemo(() => {
    //         console.log(foo);
    //       }, [foo]);
    //     }",
    //     r"function Foo() {
    //       const foo = new Map([]);
    //       useMemo(() => {
    //         console.log(foo);
    //       }, [foo]);
    //     }",
    //     r"function Foo() {
    //       const foo = /reg/;
    //       useMemo(() => {
    //         console.log(foo);
    //       }, [foo]);
    //     }",
    //     r"function Foo() {
    //       class Bar {};
    //       useMemo(() => {
    //         console.log(new Bar());
    //       }, [Bar]);
    //     }",
    //     r"function Foo() {
    //       const foo = {};
    //       useLayoutEffect(() => {
    //         console.log(foo);
    //       }, [foo]);
    //       useEffect(() => {
    //         console.log(foo);
    //       }, [foo]);
    //     }",
    // ];

    Tester::new(ExhaustiveDeps::NAME, pass, fail).test_and_snapshot();
}
