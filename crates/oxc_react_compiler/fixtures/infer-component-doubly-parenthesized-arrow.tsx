type Props = { original: string };

export const DiffEditor = (({ original }: Props) => {
  return <div>{original}</div>;
});


const sproutInputs = [
  [{original:"first"}],
  [{original:"second"}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return DiffEditor(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
