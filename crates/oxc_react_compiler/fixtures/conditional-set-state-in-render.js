import {useState} from 'react';
function Component(props) {
  const [x, setX] = useState(0);

  const foo = () => {
    setX(1);
  };

  if (props.cond) {
    setX(2);
    foo();
  }

  return x;
}

const sproutInputs = [
  [{cond:false}],
  [{cond:false,value:1}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
