function Component(props) {
  const a = [props.a];
  const b = [props.b];
  const c = [props.c];
  // We don't do constant folding for non-primitive values (yet) so we consider
  // that any of a, b, or c could return here
  return (a && b) || c;
}

const sproutInputs = [
  [{"a":1,"b":2,"c":3}],
  [{"a":11,"b":12,"c":13}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
