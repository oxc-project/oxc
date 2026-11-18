// Pattern 1a: shapeId null→generated + return Type→Primitive
// Arrow function with forEach callback
// Divergence: TS has shapeId:null, return:Type(22); Rust has shapeId:"<generated_0>", return:Primitive

const getItemKeyMap = (): Map<string, string> => {
  const itemKeys = [];
  itemKeys.forEach((itemKey: string) => {});
};


const sproutInputs = [
  [],
  [],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return getItemKeyMap(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
