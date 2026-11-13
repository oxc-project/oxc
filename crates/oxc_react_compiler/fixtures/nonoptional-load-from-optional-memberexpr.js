// Note that `a?.b.c` is semantically different from `(a?.b).c`
// Here, 'props?.a` is an optional chain, and `.b` is an unconditional load
// (nullthrows if a is nullish)

function Component(props) {
  let x = (props?.a).b;
  return x;
}

const sproutInputs = [
  [{a:{b:1}}],
  [{a:{b:2}}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
