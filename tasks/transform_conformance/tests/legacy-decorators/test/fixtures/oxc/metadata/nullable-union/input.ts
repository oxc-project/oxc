declare function dec(target: any, key: string): void;

class Source {
  @dec a!: string | null;
  @dec b!: number | undefined;
  @dec c!: boolean | null | undefined;
}
