function foo(a, b, c, d) {
  let y = [];
  label: if (a) {
    if (b) {
      y.push(c);
      break label;
    }
    y.push(d);
  }
  return y;
}

const sproutInputs = [
  [true,true,1,2],
  [true,false,3,4],
  [false,true,5,6],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return foo(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[2], sproutInputs[2], sproutInputs[0]],
};
