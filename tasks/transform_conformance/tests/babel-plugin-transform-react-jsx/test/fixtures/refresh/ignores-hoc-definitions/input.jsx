// TODO: we might want to handle HOCs at usage site, however.
// TODO: it would be nice if we could always avoid registering
// a function that is known to return a function or other non-node.

let connect = () => {
  function Comp() {
    const handleClick = () => {};
    return <h1 onClick={handleClick}>Hi</h1>;
  }
  return Comp;
};
function withRouter() {
  return function Child() {
    const handleClick = () => {};
    return <h1 onClick={handleClick}>Hi</h1>;
  }
};
