type Props = { obj?: { get?: () => { value: number } } };

export function Value({ obj }: Props) {
  const a = obj?.get!().value;
  const b = obj?.get?.()!.value;
  return (
    <span>
      {a}
      {b}
    </span>
  );
}


const sproutInputs = [[{
  obj: undefined
}], [{
  obj: {
    get: () => {
      "use no memo";

      return {
        value: 42
      };
    }
  }
}]];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Value(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
