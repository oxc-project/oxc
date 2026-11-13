function useFoo(props: {value: {x: string; y: string} | null}) {
  const value = props.value;
  return createArray(value?.x, value?.y)?.join(', ');
}

function createArray<T>(...args: Array<T>): Array<T> {
  return args;
}




const sproutInputs = [
  [{value:null}],
  [{value:{x:"a",y:"b"}}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return useFoo(args[0]);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
