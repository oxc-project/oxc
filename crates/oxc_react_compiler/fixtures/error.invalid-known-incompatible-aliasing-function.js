import {knownIncompatibleAliasing} from 'ReactCompilerKnownIncompatibleTest';

function Component() {
  const data = knownIncompatibleAliasing();
  return <div>{data}</div>;
}
