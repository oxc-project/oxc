// Hello, Bar, and Baz should be registered; handleClick and sum shouldn't.
let Hello = () => {
  const handleClick = () => {};
  return <h1 onClick={handleClick}>Hi</h1>;
}
const Bar = () => {
  return <Hello />;
};
var Baz = () => <div />;
var sum = () => {};
