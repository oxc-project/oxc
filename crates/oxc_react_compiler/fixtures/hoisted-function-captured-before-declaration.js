import {Stringify, useIdentity} from 'shared-runtime';

// A function declaration is initialized on scope entry, so capturing it in a
// callback before its declaration does not access it before initialization.
function Component({value}) {
  const callback = useIdentity(() => format());
  function format() {
    return `value: ${value}`;
  }
  return <Stringify callback={callback} format={format} shouldInvokeFns={true} />;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{value: 1}],
  sequentialRenders: [{value: 1}, {value: 1}, {value: 2}],
};
