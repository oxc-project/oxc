// Unlike __register__, we want to sign all functions -- not just top level.
// This lets us support editing HOCs better.
// For function declarations, __signature__ is called on next line.
// For function expressions, it wraps the expression.
// In order for this to work, __signature__ returns its first argument.

export const A = React.memo(React.forwardRef((props, ref) => {
  const [foo, setFoo] = useState(0);
  React.useEffect(() => {});
  return <h1 ref={ref}>{foo}</h1>;
}));

export const B = React.memo(React.forwardRef(function(props, ref) {
  const [foo, setFoo] = useState(0);
  React.useEffect(() => {});
  return <h1 ref={ref}>{foo}</h1>;
}));

function hoc() {
  return function Inner() {
    const [foo, setFoo] = useState(0);
    React.useEffect(() => {});
    return <h1 ref={ref}>{foo}</h1>;
  };
}

export let C = hoc();
