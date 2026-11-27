// When multiple functions in the same file generate UIDs, they should not
// collide. Babel's generateUid accumulates names at the program scope, so
// the second function's UIDs start where the first function left off.

function Component1({items}) {
  const mapped = items.map(x => x.id);
  return <div>{mapped}</div>;
}

function Component2({items}) {
  const mapped = items.map(x => x.name);
  return <span>{mapped}</span>;
}

export {Component1, Component2};


const sproutInputs = [
  [{items:[{id:1,name:"Ada"}]}],
  [{items:[{id:2,name:"Grace"}]}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return [Component1(...args),Component2(...args)];
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
