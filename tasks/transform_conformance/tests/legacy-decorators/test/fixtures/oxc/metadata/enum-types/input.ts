enum StringEnum {
  foo = 'string',
  bar = 'another'
}

enum TemplateStringEnum {
  template = `template literal`,
  mixed = `prefix_${'suffix'}`
}

enum NumberEnum {
  a = 1,
  b = 2
}

enum UnaryEnum {
  negative = -1,
  positive = +2,
  bitwise = ~3
}

function getString() { return 'string'; }

enum UnaryOtherEnum {
  negative = -getString(),
  positive = +getString(),
  bitwise = ~getString(),
}

enum AutoIncrementEnum {
  first,  // 0
  second, // 1
  third   // 2
}

enum MixedEnum {
  str = 'string',
  num = 1
}

enum ComputedEnum {
  computed = Math.PI,
  expression = 1 + 2
}

function decorate(target: any, property: string) {}

export class Foo {
  @decorate
  stringProp: StringEnum;

  @decorate
  templateProp: TemplateStringEnum;

  @decorate
  numberProp: NumberEnum;

  @decorate
  unaryProp: UnaryEnum;

  @decorate
  unaryOtherProp: UnaryOtherEnum;

  @decorate
  autoProp: AutoIncrementEnum;

  @decorate
  mixedProp: MixedEnum;

  @decorate
  computedProp: ComputedEnum;

  @decorate
  method(param: StringEnum): NumberEnum { return NumberEnum.a; }
}
