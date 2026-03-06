// Global constants defined at build time by TSDown

// `true` if is debug build
declare const DEBUG: boolean;

// `true` if is build for conformance testing
declare const CONFORMANCE: boolean;

// `RegExp.escape` is not yet in TypeScript's lib types (available from ES2025).
// TODO: Remove this once TypeScript ships it.
interface RegExpConstructor {
  escape(str: string): string;
}
