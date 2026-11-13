import {useState} from 'react';
function Component(props) {
  const [x, setX] = useState(null);

  const onChange = e => {
    let x = null; // intentionally shadow the original x
    setX(currentX => currentX + x); // intentionally refer to shadowed x
  };

  return <input value={x} onChange={onChange} />;
}

const sproutInputs = [
  [{}],
  [{value:1}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
