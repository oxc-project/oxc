const Foo = 1;
type FooType = string;

// Declaration-bearing exports
export /* before declaration */ const declarationValue={foo:1};
export type /* before type name */ DeclarationType={foo:string};

// prettier-ignore
export const ignoredDeclaration={foo:1,bar:2};

// Local named exports
export /* before local specifiers */ {Foo};
export type /* before local type specifiers */ {FooType};

// prettier-ignore
export   {Foo as LocalFoo,FooType};

// Source re-exports
export {Foo} from /* before source */ "./mod";
export type /* before source type specifiers */ {FooType} from "./mod";
export {Foo as JsonFoo} from "./data.json" /* before attributes */ with {type:"json"};

// prettier-ignore
export   {Foo as RemoteFoo,FooType}   from   "./mod";
