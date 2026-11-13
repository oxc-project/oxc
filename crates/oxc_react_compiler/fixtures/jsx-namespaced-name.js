function Component(props) {
  return <xml:http protocol:version={props.version} />;
}

const sproutInputs = [
  [{version:1}],
  [{version:2}],
];

export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], sproutInputs[1], sproutInputs[1], sproutInputs[0]],
};
