// @flow
// requires babel-eslint parser or flow plugin
// https://flowtype.org/blog/2015/02/18/Import-Types.html
export type MyType = {
  id: number,
  firstName: string,
  lastName: string
};

export interface MyInterface {}

export class MyClass {}

export opaque type MyOpaqueType: string = string;

