export function exportedFunction(): void {}

export declare namespace exportedFunction {
  let variable: number;
  function func(): void;
  class Class {}
  enum Enum {}
}

exportedFunction.variable = 1;
exportedFunction.func = () => {};
exportedFunction.Class = class {};
exportedFunction.Enum = {} as typeof exportedFunction.Enum;

function localFunction(): void {}

declare namespace localFunction {
  let property: number;
}

localFunction.property = 1;

export function exportedWithLocalNamespace(): void {}

declare namespace exportedWithLocalNamespace {
  let property: number;
}

exportedWithLocalNamespace.property = 1;

export function exportedWithTypeOnlyProperty(): void {}

export declare namespace exportedWithTypeOnlyProperty {
  interface property {}
}

exportedWithTypeOnlyProperty.property = {};

export default function defaultFunction(): void {}

declare namespace defaultFunction {
  let property: number;
}

defaultFunction.property = 1;
