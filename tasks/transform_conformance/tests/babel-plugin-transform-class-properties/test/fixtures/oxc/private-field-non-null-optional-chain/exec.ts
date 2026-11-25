class C {
  #fn = function () { return this; };
  #items = new Map();
  run() { return (this?.#fn!)(); }
  runOptional() { return (this?.#fn!)?.(); }
  runMultiple() { return (this?.#fn!!)(); }
  runOptionalMultiple() { return (this?.#fn!!!)?.(); }
  runMissing() { return (this.#items.get("missing")?.find(x => x.paused)!)?.(); }
  runMissingMultiple() { return (this.#items.get("missing")?.find(x => x.paused)!!)?.(); }
}

const c = new C();
expect(c.run()).toBe(c);
expect(c.runOptional()).toBe(c);
expect(c.runMultiple()).toBe(c);
expect(c.runOptionalMultiple()).toBe(c);
expect(c.runMissing()).toBeUndefined();
expect(c.runMissingMultiple()).toBeUndefined();
