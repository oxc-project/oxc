// @enableEmitInstrumentForget @compilationMode:"annotation"

import {identity} from 'shared-runtime';

function Bar(props) {
  'use forget';
  const shouldInstrument = identity(null);
  const _shouldInstrument = identity(null);
  const _x2 = () => {
    const _shouldInstrument2 = 'hello world';
    return identity({_shouldInstrument2});
  };
  return (
    <div style={shouldInstrument} other={_shouldInstrument}>
      {props.bar}
    </div>
  );
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
