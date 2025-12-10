const { a, b, c } = {};

let x = 1;
var y = "hello";
z = "world";

function topLevelFunction(param: number) {
  const localVar = param + x;
  {
    const deepestVar = y + localVar;
    return deepestVar;
  }
  return localVar;
}

export module TopLevelModule {
  interface ConcreteInterface {
    concreteVar: number;
  }
  export interface GenericInterface<T> extends ConcreteInterface {
    genericVar: T;
  }
  export const x: GenericInterface<string> = {
    concreteVar: 42,
    genericVar: "string",
  };
}

const concreteValue: TopLevelModule.GenericInterface<string> = {
  concreteVar: TopLevelModule.x.concreteVar,
  genericVar: "string",
};

class TestClass {
  instanceVar: string;
  #privateVar: string;
  static {
    const privateVar = "private";
    this.prototype.#privateVar = arrowFunc(privateVar);

    const arrowFunc = (param: string) => {
      const arrowVar = param;
      return arrowVar + y;
    };
  }

  constructor(x: string) {
    if (x) {
      this.instanceVar = x;
    }
  }
}

label: {
  const blockVar = "block";
  console.log(blockVar);
}

const unusedVar = "should be detected";
