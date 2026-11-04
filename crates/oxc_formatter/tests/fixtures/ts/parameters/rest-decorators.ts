// Decorators on a rest parameter are printed like those on other parameters
class C { m(@d ...args: unknown[]) {} }
class C2 { constructor(@Inject(TOKEN) ...deps: any[]) {} }
class C3 { m(@d x, @e ...rest) {} }
class C4 { async rest(@(await p) ...args: unknown[]) {} }
class C5 { m(@d @e ...args) {} }
class C6 {
  constructor(
    @Inject(VERY_LONG_TOKEN_NAME_FOR_DEPENDENCY_INJECTION) private readonly a: A,
    @Inject(ANOTHER_VERY_LONG_TOKEN_NAME) ...rest: unknown[]
  ) {}
}
class C7 { m(@d
  ...args) {} }
// Comments around the decorator keep their place
class C8 { m(@d /* c */ ...args) {} }
class C9 { m(@d // c
  ...args) {} }
class C10 { m(@d ...args // c
) {} }
