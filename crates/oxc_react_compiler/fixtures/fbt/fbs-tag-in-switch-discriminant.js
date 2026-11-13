import {fbs} from 'fbt';

function Component(value) {
  switch (<fbs desc="Greeting">Hello</fbs>) {
    case 0:
      let fbs = value;
      return fbs;
    default:
      return null;
  }
}


const sproutInputs = [
  [1],
  [2],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
