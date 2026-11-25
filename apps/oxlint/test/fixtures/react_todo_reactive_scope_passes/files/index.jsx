export function Foo() {
  function foo() {
    return bar();
  }
  function bar() {
    return 42;
  }
  return <div fn1={foo} fn2={bar} />;
}
