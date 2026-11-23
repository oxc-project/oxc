// Enums declared after the decorated class are classified the same as enums declared
// before it, so the metadata never reads the enum binding before it is initialized.

function dec(target: any, key: string, descriptor?: PropertyDescriptor) {}

enum DeclaredBefore {
  A = 'a',
}

export enum ExportedBefore {
  A = 1,
}

export class Foo {
  @dec
  declaredBefore: DeclaredBefore;

  @dec
  exportedBefore: ExportedBefore;

  @dec
  stringProp: StringEnum;

  @dec
  numberProp?: NumberEnum;

  @dec
  autoProp!: AutoIncrementEnum;

  @dec
  mixedProp: MixedEnum;

  @dec
  constProp: ConstEnum;

  @dec
  method(param: StringEnum): NumberEnum {
    return 1;
  }
}

export enum StringEnum {
  A = 'a',
  B = 'b',
}

enum NumberEnum {
  A = 1,
  B = 2,
}

export enum AutoIncrementEnum {
  A,
  B,
}

enum MixedEnum {
  A = 'a',
  B = 1,
}

const enum ConstEnum {
  A = 'a',
}
