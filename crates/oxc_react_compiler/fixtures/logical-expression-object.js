function component(props) {
  // The mutable range for a extens the entire body.
  // commenting out the last line of InferMutableRanges fixes it.
  // my guess of what's going on is that a is aliased into the return value object literal,
  // and that alias makes it look like the range of a needs to be extended to that point.
  // but what's weird is that the end of a's range doesn't quite extend to the object.
  let a = props.a || (props.b && props.c && props.d);
  let b = (props.a && props.b && props.c) || props.d;
  return {a, b};
}

const sproutInputs = [
  [{a:0,b:1,c:2,d:3}],
  [{a:4,b:0,c:5,d:6}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
