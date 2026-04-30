function named(a: string, b: number): void {}

function anonymous() {}

class MyClass {
  fetchChanged(since: Date): void {}

  async asyncMethod(x: number): Promise<void> {}

  get value(): string { return ""; }

  set value(v: string) {}
}

interface Fetcher {
  fetchChanged(since: Date): void;
}

type FetchFn = (since: Date) => void;
