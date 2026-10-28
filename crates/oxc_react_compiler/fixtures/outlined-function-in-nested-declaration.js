// @compilationMode:"infer"
import { useEffect } from 'react';

// The outlined effect callback references `mounts`, a binding of the function
// enclosing the compiled component. It must be inserted as a sibling of the
// component (where `mounts` is still in scope), not hoisted to module scope.
// https://github.com/oxc-project/oxc/issues/25536
export function makeProbe() {
  let mounts = 0;
  function MountCounter() {
    useEffect(() => {
      mounts += 1;
    }, []);
    return null;
  }
  return { MountCounter, read: () => mounts };
}
