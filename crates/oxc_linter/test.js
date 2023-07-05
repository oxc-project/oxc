const cases = {
  valid: [
    "const number: number = 1;",
    "function greet(): string {}",
    "function greet(): Array<string> {}",
    "function greet(): string[] {}",
    "function greet(): Array<Array<string>> {}",
    "function greet(): Array<string[]> {}",
    "function greet(param: Array<string>): Array<string> {}",
    `
class Greeter {
  message: string;
}
    `,
    `
class Greeter {
  message: Array<string>;
}
    `,
    `
class Greeter {
  message: string[];
}
    `,
    `
class Greeter {
  message: Array<Array<string>>;
}
    `,
    `
class Greeter {
  message: Array<string[]>;
}
    `,
    `
interface Greeter {
  message: string;
}
    `,
    `
interface Greeter {
  message: Array<string>;
}
    `,
    `
interface Greeter {
  message: string[];
}
    `,
    `
interface Greeter {
  message: Array<Array<string>>;
}
    `,
    `
interface Greeter {
  message: Array<string[]>;
}
    `,
    `
type obj = {
  message: string;
};
    `,
    `
type obj = {
  message: Array<string>;
};
    `,
    `
type obj = {
  message: string[];
};
    `,
    `
type obj = {
  message: Array<Array<string>>;
};
    `,
    `
type obj = {
  message: Array<string[]>;
};
    `,
    `
type obj = {
  message: string | number;
};
    `,
    `
type obj = {
  message: string | Array<string>;
};
    `,
    `
type obj = {
  message: string | string[];
};
    `,
    `
type obj = {
  message: string | Array<Array<string>>;
};
    `,
    `
type obj = {
  message: string & number;
};
    `,
    `
type obj = {
  message: string & Array<string>;
};
    `,
    `
type obj = {
  message: string & string[];
};
    `,
    `
type obj = {
  message: string & Array<Array<string>>;
};
    `,
    // https://github.com/eslint/typescript-eslint-parser/issues/397
    {
      code: `
        function foo(a: number, ...rest: any[]): void {
          return;
        }
      `,
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "function foo1(...args: any[]) {}",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "const bar1 = function (...args: any[]) {};",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "const baz1 = (...args: any[]) => {};",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "function foo2(...args: readonly any[]) {}",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "const bar2 = function (...args: readonly any[]) {};",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "const baz2 = (...args: readonly any[]) => {};",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "function foo3(...args: Array<any>) {}",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "const bar3 = function (...args: Array<any>) {};",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "const baz3 = (...args: Array<any>) => {};",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "function foo4(...args: ReadonlyArray<any>) {}",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "const bar4 = function (...args: ReadonlyArray<any>) {};",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "const baz4 = (...args: ReadonlyArray<any>) => {};",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: `
interface Qux1 {
  (...args: any[]): void;
}
      `,
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: `
interface Qux2 {
  (...args: readonly any[]): void;
}
      `,
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: `
interface Qux3 {
  (...args: Array<any>): void;
}
      `,
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: `
interface Qux4 {
  (...args: ReadonlyArray<any>): void;
}
      `,
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "function quux1(fn: (...args: any[]) => void): void {}",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "function quux2(fn: (...args: readonly any[]) => void): void {}",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "function quux3(fn: (...args: Array<any>) => void): void {}",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "function quux4(fn: (...args: ReadonlyArray<any>) => void): void {}",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "function quuz1(): (...args: any[]) => void {}",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "function quuz2(): (...args: readonly any[]) => void {}",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "function quuz3(): (...args: Array<any>) => void {}",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "function quuz4(): (...args: ReadonlyArray<any>) => void {}",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "type Fred1 = (...args: any[]) => void;",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "type Fred2 = (...args: readonly any[]) => void;",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "type Fred3 = (...args: Array<any>) => void;",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "type Fred4 = (...args: ReadonlyArray<any>) => void;",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "type Corge1 = new (...args: any[]) => void;",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "type Corge2 = new (...args: readonly any[]) => void;",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "type Corge3 = new (...args: Array<any>) => void;",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "type Corge4 = new (...args: ReadonlyArray<any>) => void;",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: `
interface Grault1 {
  new (...args: any[]): void;
}
      `,
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: `
interface Grault2 {
  new (...args: readonly any[]): void;
}
      `,
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: `
interface Grault3 {
  new (...args: Array<any>): void;
}
      `,
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: `
interface Grault4 {
  new (...args: ReadonlyArray<any>): void;
}
      `,
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: `
interface Garply1 {
  f(...args: any[]): void;
}
      `,
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: `
interface Garply2 {
  f(...args: readonly any[]): void;
}
      `,
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: `
interface Garply3 {
  f(...args: Array<any>): void;
}
      `,
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: `
interface Garply4 {
  f(...args: ReadonlyArray<any>): void;
}
      `,
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "declare function waldo1(...args: any[]): void;",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "declare function waldo2(...args: readonly any[]): void;",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "declare function waldo3(...args: Array<any>): void;",
      options: [{ ignoreRestArgs: true }],
    },
    {
      code: "declare function waldo4(...args: ReadonlyArray<any>): void;",
      options: [{ ignoreRestArgs: true }],
    },
  ],
  invalid: [
    {
      code: "const number: any = 1",
      errors: [
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 15,
        },
      ],
    },
    {
      code: "function generic(): any {}",
      errors: [
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 21,
        },
      ],
    },
    {
      code: "function generic(): Array<any> {}",
      errors: [
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 27,
        },
      ],
    },
    {
      code: "function generic(): any[] {}",
      errors: [
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 21,
        },
      ],
    },
    {
      code: "function generic(param: Array<any>): number {}",
      errors: [
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 31,
        },
      ],
    },
    {
      code: "function generic(param: any[]): number {}",
      errors: [
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 25,
        },
      ],
    },
    {
      code: "function generic(param: Array<any>): Array<any> {}",
      errors: [
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 31,
          suggestions: [
            {
              messageId: "suggestUnknown",
              output: "function generic(param: Array<unknown>): Array<any> {}",
            },
            {
              messageId: "suggestNever",
              output: "function generic(param: Array<never>): Array<any> {}",
            },
          ],
        },
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 44,
          suggestions: [
            {
              messageId: "suggestUnknown",
              output: "function generic(param: Array<any>): Array<unknown> {}",
            },
            {
              messageId: "suggestNever",
              output: "function generic(param: Array<any>): Array<never> {}",
            },
          ],
        },
      ],
    },
    {
      code: "function generic(): Array<Array<any>> {}",
      errors: [
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 33,
        },
      ],
    },
    {
      code: "function generic(): Array<any[]> {}",
      errors: [
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 27,
        },
      ],
    },
    {
      code: `
class Greeter {
    constructor(param: Array<any>) {}
}
            `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 30,
        },
      ],
    },
    {
      code: `
class Greeter {
    message: any;
}
            `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 14,
        },
      ],
    },
    {
      code: `
class Greeter {
    message: Array<any>;
}
            `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 20,
        },
      ],
    },
    {
      code: `
class Greeter {
    message: any[];
}
            `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 14,
        },
      ],
    },
    {
      code: `
class Greeter {
    message: Array<Array<any>>;
}
            `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 26,
        },
      ],
    },
    {
      code: `
class Greeter {
    message: Array<any[]>;
}
            `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 20,
        },
      ],
    },
    {
      code: `
interface Greeter {
    message: any;
}
            `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 14,
        },
      ],
    },
    {
      code: `
interface Greeter {
    message: Array<any>;
}
            `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 20,
        },
      ],
    },
    {
      code: `
interface Greeter {
    message: any[];
}
            `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 14,
        },
      ],
    },
    {
      code: `
interface Greeter {
    message: Array<Array<any>>;
}
            `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 26,
        },
      ],
    },
    {
      code: `
interface Greeter {
    message: Array<any[]>;
}
            `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 20,
        },
      ],
    },
    {
      code: `
type obj = {
    message: any;
}
            `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 14,
        },
      ],
    },
    {
      code: `
type obj = {
    message: Array<any>;
}
            `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 20,
        },
      ],
    },
    {
      code: `
type obj = {
    message: any[];
}
            `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 14,
        },
      ],
    },
    {
      code: `
type obj = {
    message: Array<Array<any>>;
}
            `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 26,
        },
      ],
    },
    {
      code: `
type obj = {
    message: Array<any[]>;
}
            `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 20,
        },
      ],
    },
    {
      code: `
type obj = {
    message: string | any;
}
            `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 23,
        },
      ],
    },
    {
      code: `
type obj = {
    message: string | Array<any>;
}
            `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 29,
        },
      ],
    },
    {
      code: `
type obj = {
    message: string | any[];
}
            `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 23,
        },
      ],
    },
    {
      code: `
type obj = {
    message: string | Array<Array<any>>;
}
            `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 35,
        },
      ],
    },
    {
      code: `
type obj = {
    message: string | Array<any[]>;
}
            `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 29,
        },
      ],
    },
    {
      code: `
type obj = {
    message: string & any;
}
            `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 23,
        },
      ],
    },
    {
      code: `
type obj = {
    message: string & Array<any>;
}
            `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 29,
        },
      ],
    },
    {
      code: `
type obj = {
    message: string & any[];
}
            `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 23,
        },
      ],
    },
    {
      code: `
type obj = {
    message: string & Array<Array<any>>;
}
            `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 35,
        },
      ],
    },
    {
      code: `
type obj = {
    message: string & Array<any[]>;
}
            `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 29,
        },
      ],
    },
    {
      code: "class Foo<t = any> extends Bar<any> {}",
      errors: [
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 15,
          suggestions: [
            {
              messageId: "suggestUnknown",
              output: "class Foo<t = unknown> extends Bar<any> {}",
            },
            {
              messageId: "suggestNever",
              output: "class Foo<t = never> extends Bar<any> {}",
            },
          ],
        },
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 32,
          suggestions: [
            {
              messageId: "suggestUnknown",
              output: "class Foo<t = any> extends Bar<unknown> {}",
            },
            {
              messageId: "suggestNever",
              output: "class Foo<t = any> extends Bar<never> {}",
            },
          ],
        },
      ],
    },
    {
      code: "abstract class Foo<t = any> extends Bar<any> {}",
      errors: [
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 24,
          suggestions: [
            {
              messageId: "suggestUnknown",
              output: "abstract class Foo<t = unknown> extends Bar<any> {}",
            },
            {
              messageId: "suggestNever",
              output: "abstract class Foo<t = never> extends Bar<any> {}",
            },
          ],
        },
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 41,
          suggestions: [
            {
              messageId: "suggestUnknown",
              output: "abstract class Foo<t = any> extends Bar<unknown> {}",
            },
            {
              messageId: "suggestNever",
              output: "abstract class Foo<t = any> extends Bar<never> {}",
            },
          ],
        },
      ],
    },
    {
      code: "abstract class Foo<t = any> implements Bar<any>, Baz<any> {}",
      errors: [
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 24,
          suggestions: [
            {
              messageId: "suggestUnknown",
              output:
                "abstract class Foo<t = unknown> implements Bar<any>, Baz<any> {}",
            },
            {
              messageId: "suggestNever",
              output:
                "abstract class Foo<t = never> implements Bar<any>, Baz<any> {}",
            },
          ],
        },
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 44,
          suggestions: [
            {
              messageId: "suggestUnknown",
              output:
                "abstract class Foo<t = any> implements Bar<unknown>, Baz<any> {}",
            },
            {
              messageId: "suggestNever",
              output:
                "abstract class Foo<t = any> implements Bar<never>, Baz<any> {}",
            },
          ],
        },
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 54,
          suggestions: [
            {
              messageId: "suggestUnknown",
              output:
                "abstract class Foo<t = any> implements Bar<any>, Baz<unknown> {}",
            },
            {
              messageId: "suggestNever",
              output:
                "abstract class Foo<t = any> implements Bar<any>, Baz<never> {}",
            },
          ],
        },
      ],
    },
    {
      code: "new Foo<any>()",
      errors: [
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 9,
        },
      ],
    },
    {
      code: "Foo<any>()",
      errors: [
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 5,
        },
      ],
    },
    {
      // https://github.com/typescript-eslint/typescript-eslint/issues/64
      code: `
function test<T extends Partial<any>>() {}
const test = <T extends Partial<any>>() => {};
      `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 2,
          column: 33,
          suggestions: [
            {
              messageId: "suggestUnknown",
              output: `
function test<T extends Partial<unknown>>() {}
const test = <T extends Partial<any>>() => {};
      `,
            },
            {
              messageId: "suggestNever",
              output: `
function test<T extends Partial<never>>() {}
const test = <T extends Partial<any>>() => {};
      `,
            },
          ],
        },
        {
          messageId: "unexpectedAny",
          line: 3,
          column: 33,
          suggestions: [
            {
              messageId: "suggestUnknown",
              output: `
function test<T extends Partial<any>>() {}
const test = <T extends Partial<unknown>>() => {};
      `,
            },
            {
              messageId: "suggestNever",
              output: `
function test<T extends Partial<any>>() {}
const test = <T extends Partial<never>>() => {};
      `,
            },
          ],
        },
      ],
    },
    {
      // https://github.com/eslint/typescript-eslint-parser/issues/397
      code: `
        function foo(a: number, ...rest: any[]): void {
          return;
        }
      `,
      errors: [
        {
          messageId: "unexpectedAny",
          line: 2,
          column: 42,
        },
      ],
    },
    {
      code: "type Any = any;",
      options: [{ ignoreRestArgs: true }],
      errors: [
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 12,
        },
      ],
    },
    {
      code: "function foo5(...args: any) {}",
      options: [{ ignoreRestArgs: true }],
      errors: [
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 24,
        },
      ],
    },
    {
      code: "const bar5 = function (...args: any) {}",
      options: [{ ignoreRestArgs: true }],
      errors: [
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 33,
        },
      ],
    },
    {
      code: "const baz5 = (...args: any) => {}",
      options: [{ ignoreRestArgs: true }],
      errors: [
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 24,
        },
      ],
    },
    {
      code: "interface Qux5 { (...args: any): void; }",
      options: [{ ignoreRestArgs: true }],
      errors: [
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 28,
        },
      ],
    },
    {
      code: "function quux5(fn: (...args: any) => void): void {}",
      options: [{ ignoreRestArgs: true }],
      errors: [
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 30,
        },
      ],
    },
    {
      code: "function quuz5(): ((...args: any) => void) {}",
      options: [{ ignoreRestArgs: true }],
      errors: [
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 30,
        },
      ],
    },
    {
      code: "type Fred5 = (...args: any) => void;",
      options: [{ ignoreRestArgs: true }],
      errors: [
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 24,
        },
      ],
    },
    {
      code: "type Corge5 = new (...args: any) => void;",
      options: [{ ignoreRestArgs: true }],
      errors: [
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 29,
        },
      ],
    },
    {
      code: "interface Grault5 { new (...args: any): void; }",
      options: [{ ignoreRestArgs: true }],
      errors: [
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 35,
        },
      ],
    },
    {
      code: "interface Garply5 { f(...args: any): void; }",
      options: [{ ignoreRestArgs: true }],
      errors: [
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 32,
        },
      ],
    },
    {
      code: "declare function waldo5(...args: any): void;",
      options: [{ ignoreRestArgs: true }],
      errors: [
        {
          messageId: "unexpectedAny",
          line: 1,
          column: 34,
        },
      ],
    },
  ],
};
