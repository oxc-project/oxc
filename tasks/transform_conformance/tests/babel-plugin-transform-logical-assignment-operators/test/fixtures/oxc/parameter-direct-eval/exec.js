function run() {
  const object = {};
  function getObject() {
    return object;
  }
  function check(value = (getObject().value ||= eval("var fromEval = 1; 2")), later = fromEval) {
    return [value, later, fromEval];
  }
  return check();
}
// Vitest modules are strict. Reparse the transformed function text in sloppy mode
// so direct eval can introduce bindings visible to the later parameter and body.
const sloppyRun = Function(`return (${run.toString()})`)();
expect(sloppyRun()).toEqual([2, 1, 1]);
