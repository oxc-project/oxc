function Component(props) {
  // b is an object, must be memoized even though the input is not memoized
  const {a, ...b} = props.a;
  // d is an array, mut be memoized even though the input is not memoized
  const [c, ...d] = props.c;
  return <div b={b} d={d}></div>;
}

const sproutInputs = [
  [{a:{a:1,b:2},c:[3,4,5]}],
  [{a:{a:6,b:7},c:[8,9]}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
