const p = Promise.resolve(() => {});
class C {
  async rest(@(await p) ...args: unknown[]) {}
}
export {};
