function foo(a, b, c) {
  const x = {...a};
  x[b] = c[b];
  x[1 + 2] = c[b * 4];
}

const sproutInputs = [
  [{base:1},1,{1:2,4:3}],
  [{base:4},2,{2:5,8:6}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return foo(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
