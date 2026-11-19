// @expectNothingCompiled @compilationMode:"infer"
export function Decorate() {
  return function (
    _target: object,
    _key: string,
    descriptor: PropertyDescriptor,
  ) {
    const original = descriptor.value;
    descriptor.value = function (this: unknown) {
      return original.apply(this, []);
    };
  };
}


const sproutInputs = [
  [1],
  [2],
];
export const FIXTURE_ENTRYPOINT = {
  fn: args => {
    "use no memo";
    return (() => { const descriptor={value:function(){return this.value;}}; Decorate()({},"value",descriptor); return descriptor.value.call({value:args[0]}); })();
  },
  params: [sproutInputs[0]],
  sequentialRenders: [sproutInputs[0], sproutInputs[0], ...sproutInputs.slice(1), sproutInputs.at(-1), sproutInputs[0]],
};
