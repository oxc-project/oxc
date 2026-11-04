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
