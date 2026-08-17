import {Stringify} from 'shared-runtime';

function Component() {
  function Factory() {
    return () => () => new.target;
  }

  const readNewTarget = new Factory()();
  return <Stringify value={readNewTarget()} />;
}

function ConditionalComponent(flag) {
  if (flag) {
    new.target;
  }
  const readNewTarget = () => new.target;
  return <Stringify value={readNewTarget()} />;
}
