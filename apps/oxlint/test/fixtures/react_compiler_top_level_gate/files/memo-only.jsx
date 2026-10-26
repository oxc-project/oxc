let someGlobal = false;

// Only a `React.memo` call at the top level, so the file is skipped like in
// `eslint-plugin-react-hooks`.
export const Component = React.memo(function (props) {
  someGlobal = true;
  return <div>{props.text}</div>;
});
