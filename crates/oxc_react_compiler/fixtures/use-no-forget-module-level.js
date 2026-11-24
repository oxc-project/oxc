'use no forget';

export default function foo(x, y) {
  if (x) {
    return foo(false, y);
  }
  return [y * 10];
}


const sproutInputs = [
  [true,2],
  [false,3],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return foo(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
