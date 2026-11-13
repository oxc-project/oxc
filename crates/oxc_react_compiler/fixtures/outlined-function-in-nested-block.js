// @compilationMode:"infer"
import { useEffect } from 'react';

// Like outlined-function-in-nested-declaration, but the compiled component sits
// in a block statement: the outlined callback is inserted into that block,
// mirroring Babel `insertAfter` on the enclosing statement list.
export function makeProbe(flag) {
  let mounts = 0;
  if (flag) {
    function MountCounter() {
      useEffect(() => {
        mounts += 1;
      }, []);
      return null;
    }
    return { MountCounter, read: () => mounts };
  }
  return null;
}


const sproutInputs = [
  [true],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return (() => { const probe=makeProbe(...args); return <probe.MountCounter/>; })();
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
