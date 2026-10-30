// @outputMode:"lint" @panicThreshold:"none"
import {Stringify} from 'shared-runtime';

// Lint reports the Todo that a pass after validation records for the hoisted
// function reference.
function Foo() {
  function foo() {
    return bar();
  }
  function bar() {
    return 42;
  }
  return <Stringify fn1={foo} fn2={bar} />;
}
