import {useEffect, useMemo} from 'react';

// The `Map` is part of the `useMemo` result, so a new one is created whenever
// `env` changes instead of once per mount.
function Component({env}) {
  const cache = useMemo(() => ({builtFor: env, entries: new Map()}), [env]);
  useEffect(() => {
    cache.entries.set(cache.builtFor, true);
  }, [cache]);
  return (
    <div>
      {cache.builtFor}:{cache.entries.size}
    </div>
  );
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{env: 'a'}],
  sequentialRenders: [{env: 'a'}, {env: 'a'}, {env: 'b'}, {env: 'b'}],
};
