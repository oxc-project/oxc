import { AExtend, BExtend, Type, CImplements, CType, ThisType1, ThisType2, Unused } from 'mod';

export interface A extends AExtend<Type> {}
export class B extends BExtend<Type> {}
export class C implements CImplements<CType> {}
export function foo(this: ThisType1): void {}
export const bar: (this: ThisType2) => void = function() {}