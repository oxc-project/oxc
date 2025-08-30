declare function methodDecorator(target: any);
declare function paramDecorator(target: any);

export class Foo {
  @methodDecorator(1)
  @methodDecorator(2)
  method1(@paramDecorator param: string): boolean {
    return !!param
  }

  @methodDecorator(1)
  @methodDecorator(2)
  method2(param: string): boolean {
    return !!param
  }

  constructor(@paramDecorator param: number) {

  }

  method3(@paramDecorator param: string): boolean {
    return !!param
  }

  method4(@paramDecorator param: string, @paramDecorator param2: string): boolean {
    return !!param
  }
}
