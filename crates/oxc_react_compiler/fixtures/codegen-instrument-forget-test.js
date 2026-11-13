// @enableEmitInstrumentForget @compilationMode:"annotation"

function Bar(props) {
  'use forget';
  return <div>{props.bar}</div>;
}

function NoForget(props) {
  return <Bar>{props.noForget}</Bar>;
}

function Foo(props) {
  'use forget';
  return <Foo>{props.bar}</Foo>;
}


const sproutInputs = [
  [{bar:"first"}],
  [{bar:"second"}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Bar(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
