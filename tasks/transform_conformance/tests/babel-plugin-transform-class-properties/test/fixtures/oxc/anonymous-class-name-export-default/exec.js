// `export default class {}` is named "default". Static initializers run after naming.
export default class {
  static n = (globalThis.anonymousDefaultExportClassName = this.name);
}

expect(globalThis.anonymousDefaultExportClassName).toBe("default");
delete globalThis.anonymousDefaultExportClassName;
