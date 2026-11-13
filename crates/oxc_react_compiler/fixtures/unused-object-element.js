function Foo(props) {
  const {x, y, ...z} = props.a;
  return x;
}

const sproutInputs = [
  [{a:{x:1,y:2,z:3}}],
  [{a:{x:4,y:5}}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Foo(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
