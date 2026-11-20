class A {
  @dec(x as any) a: number;
  @dec<string>((v: string) => v!) b = 1;
}
