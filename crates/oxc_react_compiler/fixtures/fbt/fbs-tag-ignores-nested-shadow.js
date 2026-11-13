import {fbs} from 'fbt';

function Component() {
  function nested() {
    const fbs = () => null;
    return fbs;
  }
  return <fbs desc="Greeting">Hello</fbs>;
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
