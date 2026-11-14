// @enableNewMutationAliasingModel
function useHook({el1, el2}) {
  const s = new Set();
  const arr = makeArray(el1);
  s.add(arr);
  // Mutate after store
  arr.push(el2);

  s.add(makeArray(el2));
  return s.size;
}

import {makeArray} from 'shared-runtime';
const sproutInputs = [
  [{el1:1,el2:2}],
  [{el1:3,el2:4}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return useHook(args[0]);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
