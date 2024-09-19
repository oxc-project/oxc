// Hello and Bar should be registered; handleClick, sum, Baz, and Qux shouldn't.

let Hello = function() {
  function handleClick() {}
  return <h1 onClick={handleClick}>Hi</h1>;
};
const Bar = function Baz() {
  return <Hello />;
};
function sum() {}
let Baz = 10;
var Qux;
