// @compilationMode:"infer" @panicThreshold:"none"
// `suffix` is left out of the dependency list, so the callback keeps the `suffix` it was
// created with until `prefix` changes. The useCallback is kept as written.
import { useCallback } from 'react';

function Label({ prefix, suffix }) {
  const format = useCallback(() => prefix + suffix, [prefix]);
  return <Text format={format} suffix={suffix} />;
}

import {Text} from 'shared-runtime';
const sproutInputs = [
  [{"prefix":1,"suffix":1}],
  [{"prefix":2,"suffix":2}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Label(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
