export class SymbolKeys {
  [Symbol.iterator](): Iterator<unknown> {
    return [][Symbol.iterator]();
  }

  [globalThis.Symbol.iterator](): Iterator<unknown> {
    return [][Symbol.iterator]();
  }

  [window.Symbol.iterator](): Iterator<unknown> {
    return [][Symbol.iterator]();
  }
}
