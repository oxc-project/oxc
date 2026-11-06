import {applyCallbackAndReturnPrimitive} from 'ReactCompilerNoAliasTest';

function Component() {
  let count = 0;
  const result = applyCallbackAndReturnPrimitive(() => () => count++);
  return result;
}
