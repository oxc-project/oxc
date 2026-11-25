function foo(a, b, c) {
  const x = [];
  if (a) {
    const y = [];
    if (b) {
      y.push(c);
    }
    x.push(<div>{y}</div>);
  }
  return x;
}

const sproutInputs = [
  [true,true,1],
  [true,false,2],
  [false,true,3],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return foo(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[2], sproutInputs[2], sproutInputs[0]],
};
