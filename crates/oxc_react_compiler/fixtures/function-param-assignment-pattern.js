function Component(x = 'default', y = [{}]) {
  return [x, y];
}

const sproutInputs = [
  [],
  ["explicit",[1,2]],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
