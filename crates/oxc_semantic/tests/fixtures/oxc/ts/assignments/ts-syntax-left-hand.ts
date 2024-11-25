let Foo: any= 0;

Foo! += 1;
(<any>Foo) = 1;
(Foo as any) = 1;
(Foo satisfies any) = 1;

Foo.bar! += 1;
(<any>Foo.bar) = 1;
(Foo.bar as any) = 1;
(Foo.bar satisfies any) = 1;