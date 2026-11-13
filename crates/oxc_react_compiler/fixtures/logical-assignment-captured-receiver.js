function Component({value, initial, operator}) {
  const stored = {x: null};
  const other = {x: 'other'};
  let obj = {};
  Object.defineProperty(obj, 'x', {
    get() {
      obj = other;
      return initial;
    },
    set(value) {
      stored.x = value;
    },
  });
  let result;
  if (operator === '??=') {
    result = obj.x ??= value;
  } else if (operator === '||=') {
    result = obj.x ||= value;
  } else {
    result = obj.x &&= value;
  }
  return [result, stored.x, obj === other, other.x];
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{value: 42, initial: null, operator: '??='}],
};
