function foo(a, b, c) {
  let x = a;
  if (b) {
    if (c) {
      x = c;
    }
    // TODO: move the return to the end of the function
    return x;
  }
}

const sproutInputs = [
  [1,true,2],
  [3,true,false],
  [4,false,5],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return foo(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[2], sproutInputs[2], sproutInputs[0]],
};
