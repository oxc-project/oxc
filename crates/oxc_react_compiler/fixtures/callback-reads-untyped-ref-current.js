import {useEffect, useRef} from 'react';

// After `??` the compiler no longer knows that `target` is a ref. The effect
// reads `target.current` after the ref is attached, so the cache guard must not
// read it during render, while it is still null.
function Component({forwardedRef}) {
  const ownRef = useRef(null);
  const target = forwardedRef ?? ownRef;
  const report = () => {
    console.log(target.current.tagName);
  };
  useEffect(report);
  return <div ref={target} />;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{forwardedRef: undefined}],
  sequentialRenders: [{forwardedRef: undefined}, {forwardedRef: undefined}],
};
