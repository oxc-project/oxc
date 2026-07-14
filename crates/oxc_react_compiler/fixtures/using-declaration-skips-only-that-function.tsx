// `using`/`await using` disposal semantics aren't preserved yet, so `WithUsing` is
// skipped and emitted unchanged, while the structurally identical `Clean` component
// in the same file is still compiled.
function WithUsing(props) {
  using resource = props.acquire();
  return <div onClick={() => props.onClick()}>{props.text}</div>;
}

function Clean(props) {
  return <div onClick={() => props.onClick()}>{props.text}</div>;
}
