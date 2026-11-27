function Component(props) {
  // a does not need to be memoized ever, even though it's a
  // dependency of c, which exists in a scope that has a memoized
  // output. it doesn't need to be memoized bc the value is a primitive type.
  const a = props.a + props.b;

  // b and c are interleaved and grouped into a single scope,
  // but they are independent values. c does not escape, but
  // we need to ensure that a is memoized or else b will invalidate
  // on every render since a is a dependency.
  const b = [];
  const c = {};
  c.a = a;
  b.push(props.c);

  return b;
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
