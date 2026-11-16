function ternary(props) {
  const a = props.a && props.b ? props.c || props.d : (props.e ?? props.f);
  const b = props.a ? (props.b && props.c ? props.d : props.e) : props.f;
  return a ? b : null;
}

const sproutInputs = [
  [{"a":1,"b":2,"c":3,"d":4,"e":5,"f":6}],
  [{"a":11,"b":12,"c":13,"d":14,"e":15,"f":16}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return ternary(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
