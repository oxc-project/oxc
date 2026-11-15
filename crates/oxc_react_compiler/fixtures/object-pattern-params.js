function component({a, b}) {
  let y = {a};
  let z = {b};
  return {y, z};
}

const sproutInputs = [
  [{a:1,b:2}],
  [{a:3,b:4}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
