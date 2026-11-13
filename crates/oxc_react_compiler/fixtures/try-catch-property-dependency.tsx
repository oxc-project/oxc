function Component({name}: {name: string}) {
  try {
    const url = new URL(name);
    return <div>{url.search}{url.hash}</div>;
  } catch {
    return null;
  }
}


const sproutInputs = [
  [{name:"https://example.com/?a=1#one"}],
  [{name:"invalid"}],
  [{name:"https://example.com/?a=2#two"}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Component(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
