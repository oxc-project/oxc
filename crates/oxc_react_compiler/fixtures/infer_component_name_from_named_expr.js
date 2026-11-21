const MyComponent = function helper({x}) {
  return <div>{x * 2}</div>;
};


const sproutInputs = [
  [{"x":1}],
  [{"x":2}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return MyComponent(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
