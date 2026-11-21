function component(a) {
  let z = {a: {a}};
  let x = function () {
    (function () {
      console.log(z.a.a);
    })();
  };
  return x;
}

const sproutInputs = [
  [1],
  [11],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return component(...args)();
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
