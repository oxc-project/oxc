import {Stringify, mutate} from 'shared-runtime';

function foo() {
  const nt = new.target;
  return <Stringify value={nt} />;
}

function Component() {
  return (function () {
    return new.target;
  })();
}

function AliasingComponent() {
  const unused = new.target;
  const Constructor = new.target;
  mutate(Constructor);
  return <Stringify value={new.target.x} />;
}

function ArrowDependencyComponent() {
  const read = () => new.target;
  return <Stringify value={read().x} />;
}
