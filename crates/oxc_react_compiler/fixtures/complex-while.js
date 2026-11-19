function foo(a, b, c) {
  label: if (a) {
    while (b) {
      if (c) {
        break label;
      }
    }
  }
  return c;
}

const sproutInputs = [
  [true,true,true],
  [true,false,false],
  [false,true,false],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return foo(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[2], sproutInputs[2], sproutInputs[0]],
};
