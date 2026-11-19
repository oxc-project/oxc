// @compilationMode:"infer" @panicThreshold:"none"
import {useMemo} from 'react';

export function Component({status, tick}) {
  const value = useMemo(() => {
    void tick;
    return status.expiresAt;
  }, [tick, status.expiresAt]);
  return <div>{value}</div>;
}


const sproutInputs = [
  [{status:{expiresAt:1},tick:0}],
  [{status:{expiresAt:2},tick:1}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
