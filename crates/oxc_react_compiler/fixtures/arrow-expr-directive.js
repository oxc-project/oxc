function Component() {
  'use strict';
  let [count, setCount] = React.useState(0);
  const update = () => {
    'worklet';
    setCount(count => count + 1);
  };
  return <button onClick={update}>{count}</button>;
}


const sproutInputs = [
  [],
  [],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
