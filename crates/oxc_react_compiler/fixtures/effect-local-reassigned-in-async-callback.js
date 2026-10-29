import {useEffect} from 'react';

// `timeoutId` is declared inside the effect, so reassigning it after `await`
// in the async callback never crosses a render.
function useAfterRun(run) {
  useEffect(() => {
    let timeoutId;
    const go = async () => {
      await run();
      timeoutId = setTimeout(() => {}, 10);
    };
    go();
    return () => clearTimeout(timeoutId);
  }, [run]);
}
