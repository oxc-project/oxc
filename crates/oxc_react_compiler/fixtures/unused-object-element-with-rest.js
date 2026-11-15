function Foo(props) {
  // can't remove `unused` since it affects which properties are copied into `rest`
  const {unused, ...rest} = props.a;
  return rest;
}

const sproutInputs = [
  [{a:{unused:1,x:2,y:3}}],
  [{a:{unused:4,x:5}}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Foo(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
