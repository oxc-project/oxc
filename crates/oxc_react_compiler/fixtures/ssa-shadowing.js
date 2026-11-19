function log() {}

function Foo(cond) {
  let str = '';
  if (cond) {
    let str = 'other test';
    log(str);
  } else {
    str = 'fallthrough test';
  }
  log(str);
}


const sproutInputs = [
  [true],
  [false],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Foo(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
