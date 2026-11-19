import "reflect-metadata";

function check(expected: unknown) {
  return (target: object, key: string) => {
    expect(Reflect.getMetadata("design:type", target, key), key).toBe(expected);
  };
}

{
  // Empty-first, empty-last, and split declarations.
  enum EmptyFirstNum {}
  enum EmptyFirstNum { A = 1 }
  enum EmptyFirstStr {}
  enum EmptyFirstStr { A = "a" }
  enum EmptyLastNum { A = 1 }
  enum EmptyLastNum {}
  enum EmptyLastStr { A = "a" }
  enum EmptyLastStr {}
  enum SplitNum { A = 1 }
  enum SplitStr { A = "a" }

  // Empty declarations are neutral; mixed and unknown values still yield Object.
  enum Empty {}
  enum Empty {}
  enum Mixed { A = "a", B = 1 }
  enum Mixed {}
  enum Mixed { C = 2 }
  enum Unknown { A = Math.random() }
  enum Unknown {}
  enum Unknown { B = 1 }

  class C {
    @check(Number) numEmptyFirst!: EmptyFirstNum;
    @check(String) strEmptyFirst!: EmptyFirstStr;
    @check(Number) numEmptyLast!: EmptyLastNum;
    @check(String) strEmptyLast!: EmptyLastStr;
    @check(Number) numSplit!: SplitNum;
    @check(String) strSplit!: SplitStr;
    @check(Number) numForwardEmptyFirst!: ForwardEmptyFirstNum;
    @check(String) strForwardEmptyFirst!: ForwardEmptyFirstStr;
    @check(Number) numForwardEmptyLast!: ForwardEmptyLastNum;
    @check(String) strForwardEmptyLast!: ForwardEmptyLastStr;
    @check(Object) empty!: Empty;
    @check(Object) mixed!: Mixed;
    @check(Object) unknown!: Unknown;
  }

  // Empty declarations after the class, including forward references lowered to let.
  enum SplitNum {}
  enum SplitStr {}
  enum ForwardEmptyFirstNum {}
  enum ForwardEmptyFirstNum { A = 1 }
  enum ForwardEmptyFirstStr {}
  enum ForwardEmptyFirstStr { A = "a" }
  enum ForwardEmptyLastNum { A = 1 }
  enum ForwardEmptyLastNum {}
  enum ForwardEmptyLastStr { A = "a" }
  enum ForwardEmptyLastStr {}
}
