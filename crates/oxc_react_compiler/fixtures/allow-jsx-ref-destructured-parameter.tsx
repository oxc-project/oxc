// @validateRefAccessDuringRender @compilationMode:"infer"

export function Button({ref}) {
  return <button ref={ref} />;
}


const sproutInputs = [
  [{ref:null}],
  [{ref:{current:null}}],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return Button(...args);
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
