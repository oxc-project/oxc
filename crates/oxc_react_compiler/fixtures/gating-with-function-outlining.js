// @gating
import { useEffect } from 'react';

// The outlined effect callback is inserted immediately after the gated
// declaration (upstream inserts it before gating rewrites the statement),
// not appended at the end of the program.
function Component() {
  useEffect(() => {
    console.log('mounted');
  }, []);
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
  sequentialRenders: sproutInputs,
};
