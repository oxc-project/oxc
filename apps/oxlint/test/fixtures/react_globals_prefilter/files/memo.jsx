let someGlobal = false;

// An anonymous component wrapped in `React.memo` is linted when the file also
// has a top-level component.
export const Component = React.memo(function (props) {
  someGlobal = true;
  return <div>{props.text}</div>;
});

export function App() {
  return <Component text="hello" />;
}
