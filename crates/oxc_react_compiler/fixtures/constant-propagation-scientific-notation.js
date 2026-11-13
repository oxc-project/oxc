// Template literal constant folding uses format_js_number to convert
// numbers to strings. Without the shared implementation, large numbers
// may format incorrectly (e.g. 1e21 as "1e21" instead of "1e+21").

function Component() {
  const x = `value is ${1e21}`;
  return <div>{x}</div>;
}


const sproutInputs = [
  [],
  [],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
