// `const C = class {}` (Variable symbol, not Class declaration). The guard
// returns `C` at runtime — same code path as named class declarations.

const C = class {
  value = 1;
};

declare function dec(target: any, key: string): void;

class Source {
  @dec x!: C;
}
