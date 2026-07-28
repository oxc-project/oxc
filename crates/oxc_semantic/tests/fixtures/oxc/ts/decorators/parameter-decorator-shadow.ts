const foo = { KEY: "token" };

declare function Inject(token: unknown): ParameterDecorator;

export class C {
  constructor(@Inject(foo.KEY) private readonly foo: number) {}
}

export class D {
  constructor(foo: string, @Inject(foo.KEY) bar: number) {}
}
