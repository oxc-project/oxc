// Valid although unconditional return doesn't make sense and would fail other rules.
// We could make it invalid but it doesn't matter.
function useUnreachable() {
  return;
  useHook();
}

import {useHook} from 'shared-runtime';
const sproutInputs = [
  [],
  [],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return useUnreachable();
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
