function foo(a) {
  const x = [a.b];
  return x;
}

const sproutInputs = [
  [{b:1}],
  [{b:2}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return foo(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
