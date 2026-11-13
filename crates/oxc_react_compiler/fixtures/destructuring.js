function foo(a, b, c) {
  const [
    d,
    [
      {
        e: {f},
        ...g
      },
    ],
    ...h
  ] = a;
  const {
    l: {
      m: [[n], ...o],
    },
    p,
  } = b;
  return [d, f, g, h, n, o, p];
}

const sproutInputs = [
  [[1,[{e:{f:2},extra:3}],4],{l:{m:[[5],[6]]},p:7},8],
  [[8,[{e:{f:9},extra:10}],11],{l:{m:[[12],[13]]},p:14},15],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return foo(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
