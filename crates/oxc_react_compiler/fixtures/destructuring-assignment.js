function foo(a, b, c) {
  let d, g, n, o;
  [
    d,
    [
      {
        e: {f: g},
      },
    ],
  ] = a;
  ({
    l: {
      m: [[n]],
    },
    o,
  } = b);
  return {d, g, n, o};
}

const sproutInputs = [
  [[1,[{e:{f:2}}]],{l:{m:[[3]]},o:4},5],
  [[6,[{e:{f:7}}]],{l:{m:[[8]]},o:9},10],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return foo(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
