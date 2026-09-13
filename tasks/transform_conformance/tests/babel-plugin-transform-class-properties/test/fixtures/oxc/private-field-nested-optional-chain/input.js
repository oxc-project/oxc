class Cache {
  #items = new Map();
  runNext(key) {
    return (this.#items.get(key)?.find(x => x.paused))?.continue()
      ?? Promise.resolve();
  }
  callNext(key) {
    return (this.#items.get(key)?.find(x => x.paused))?.();
  }
}
