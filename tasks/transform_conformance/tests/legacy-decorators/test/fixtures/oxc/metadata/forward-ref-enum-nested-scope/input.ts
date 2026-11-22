// Enums declared after the decorated class in a nested scope.

function dec(target: any, key: string) {}

export namespace NS {
  export class InNamespace {
    @dec
    prop: Later;
  }

  export enum Later {
    A = 'a',
  }
}

export function inFunction() {
  class InFunction {
    @dec
    prop: Later;
  }

  enum Later {
    A = 1,
  }

  return InFunction;
}

{
  class InBlock {
    @dec
    prop: Later;
  }

  enum Later {
    A = 'a',
  }
}
