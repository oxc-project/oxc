// @panicThreshold:"none"
import {useHook} from 'shared-runtime';

function InvalidComponent(props) {
  if (props.cond) {
    useHook();
  }
  return <div>Hello World!</div>;
}

function ValidComponent(props) {
  return <div>{props.greeting}</div>;
}


const sproutInputs = [
  [{greeting:"first"}],
  [{greeting:"second"}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return ValidComponent(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
