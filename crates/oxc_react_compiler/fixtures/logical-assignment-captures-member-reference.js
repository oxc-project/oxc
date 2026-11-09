import {Stringify} from 'shared-runtime';

// A getter may reassign the receiver or the computed key after the reference
// for `??=` / `||=` / `&&=` is evaluated. The store must use that original
// reference, not a second read of the binding.
function Component({value}) {
  let obj = {};
  const other = {x: 'other'};
  const stored = {};
  Object.defineProperty(obj, 'x', {
    get() {
      obj = other;
      return null;
    },
    set(v) {
      stored.x = v;
    },
    configurable: true,
  });
  const fromGetter = obj.x ??= value;

  let key = 'a';
  const keyed = {};
  const keyedStore = {};
  Object.defineProperty(keyed, 'a', {
    get() {
      key = 'b';
      return null;
    },
    set(v) {
      keyedStore.a = v;
    },
    configurable: true,
  });
  Object.defineProperty(keyed, 'b', {
    get() {
      return 'b-get';
    },
    set(v) {
      keyedStore.b = v;
    },
    configurable: true,
  });
  const fromKey = keyed[key] ??= value;

  return (
    <Stringify
      fromGetter={fromGetter}
      storedX={stored.x}
      objIsOther={obj === other}
      otherX={other.x}
      fromKey={fromKey}
      keyedStore={keyedStore}
      key={key}
    />
  );
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{value: 42}],
};
