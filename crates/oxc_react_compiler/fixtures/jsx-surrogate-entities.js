function Component() {
  return (
    <div title="&#xD800;" pair="&#xD83D;&#xDE00;" mixed="x&#xDC00;y">
      &#xD800;{"&#xD800;"}&#xD83D;&#xDE00;
      &#xD83D;
      &#xDE00;
    </div>
  );
}


const sproutInputs = [
  [],
  [],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
