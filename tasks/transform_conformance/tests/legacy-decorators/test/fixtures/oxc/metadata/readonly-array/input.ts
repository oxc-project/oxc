declare function dec(target: any, key: string): void;

class Source {
  @dec items!: ReadonlyArray<string>;
  @dec nested!: ReadonlyArray<ReadonlyArray<number>>;
}
