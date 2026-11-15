function check(createFunction, initialValue, expectedValue) {
  let active = false;
  let firstValue = initialValue;
  const second = { value: initialValue };
  const first = {
    get value() {
      if (!active) {
        active = true;
        f();
      }
      return initialValue;
    },
    set value(value) {
      firstValue = value;
    },
  };
  function getObject() {
    return active ? second : first;
  }
  const f = createFunction(getObject);
  f();
  expect([firstValue, second.value]).toEqual([expectedValue, expectedValue]);
}

check(getObject => function (value = (getObject().value ||= 1)) {}, 0, 1);
check(getObject => function (value = (getObject().value &&= 2)) {}, 1, 2);
check(getObject => function (value = (getObject().value ??= 1)) {}, null, 1);
check(getObject => (value = [getObject().value ||= 1]) => {}, 0, 1);
check(getObject => function ({ value = (getObject().value ||= 1) } = {}) {}, 0, 1);
check(getObject => function ([value = (getObject().value ||= 1)] = []) {}, 0, 1);
check(getObject => function ({ [getObject().value ||= 1]: value } = {}) {}, 0, 1);
check(getObject => function (...[value = (getObject().value ||= 1)]) {}, 0, 1);

let active = false;
let firstValue = 0;
const values = {
  get first() {
    if (!active) {
      active = true;
      f();
    }
    return 0;
  },
  set first(value) {
    firstValue = value;
  },
  second: 0,
};
function getKey() {
  return active ? "second" : "first";
}
function f(value = (values[getKey()] ||= 1)) {}
f();
expect([firstValue, values.second]).toEqual([1, 1]);
