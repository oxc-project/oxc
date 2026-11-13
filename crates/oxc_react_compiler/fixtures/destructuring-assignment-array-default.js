function Component(props) {
  let x;
  if (props.cond) {
    [[x] = ['default']] = props.y;
  } else {
    x = props.fallback;
  }
  return x;
}

const sproutInputs = [
  [{cond:true,y:[]}],
  [{cond:true,y:[[42]]}],
  [{cond:false,y:[],fallback:7}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[2], sproutInputs[2], sproutInputs[0]],
};
