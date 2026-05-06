declare function property(): (target: any, propertyKey: string, desc: PropertyDescriptor) => void;

const prop = 'foo'
const _prop = 'bar'

export class Foo {
  @property()
  accessor prop!: string;

  @property()
  accessor _prop!: string;

  @property()
  accessor [prop]!: string;

  @property()
  accessor [_prop]!: string;
}
