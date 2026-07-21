// A resource declaration is preserved while the component's memoizable work is compiled.
function WithUsing(props) {
  using resource = props.acquire();
  return <div onClick={() => props.onClick()}>{props.text}</div>;
}

function Clean(props) {
  return <div onClick={() => props.onClick()}>{props.text}</div>;
}
