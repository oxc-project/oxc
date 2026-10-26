let someGlobal = false;

// An anonymous component wrapped in `React.memo` is linted.
export const Component = React.memo(function (props) {
  someGlobal = true;
  return <div>{props.text}</div>;
});
