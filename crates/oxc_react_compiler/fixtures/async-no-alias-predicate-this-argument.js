function Component() {
  let count = 0;
  const bad = () => count++;
  const receiver = async () => {
    await 0;
    bad();
  };
  const values = [0];
  values.filter(() => true, receiver);
  values.every(() => true, receiver);
  values.some(() => true, receiver);
  values.find(() => true, receiver);
  values.findIndex(() => true, receiver);
  new Set(values).forEach(() => null, receiver);
  new Map().forEach(() => null, receiver);
  return null;
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
