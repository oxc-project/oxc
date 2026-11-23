function run() {
  const object = { value: 3 };
  function getObject() { return object; }
  function check(value = getObject().value **= eval("var fromEval = 1; 2"), later = fromEval) {
    return [value, later, fromEval];
  }
  return check();
}
const sloppyRun = Function(`return (${run.toString()})`)();
expect(sloppyRun()).toEqual([9, 1, 1]);
