// Printing options, and the source map types the `sourceMap` option brings with it.

/**
 * Source map generator, compatible with `SourceMapGenerator` from the `source-map` package.
 */
export interface SourceMapGenerator {
  addMapping(mapping: Mapping): void;
  file?: string;
  _file?: string;
}

/**
 * Source map mapping emitted through `Options["sourceMap"]`.
 *
 * Note: the `Mapping` objects passed to `addMapping` are reused across calls;
 * implementations must copy any values they need to retain.
 */
export interface Mapping {
  original: Position;
  generated: Position;
  name: string | undefined;
  source: string;
}

/**
 * A `Mapping` before the per-mapping fields are filled in.
 */
export type MutableMapping = Omit<Mapping, "original" | "source"> & {
  original: Position | null;
  source: string | undefined;
};

/**
 * Position in the original source, as found on a node's `loc.start`.
 */
export interface Position {
  line: number;
  column: number;
}

/**
 * Code generator options.
 */
export interface Options {
  /**
   * String to use for indentation, defaults to `"\t"`.
   * Must be a non-empty string consisting only of spaces and/or tabs.
   * Throws a `TypeError` otherwise.
   */
  indent?: string;

  /**
   * Non-negative integer indent level to start from, from `0` to `1000`. Defaults to `0`.
   */
  startingIndentLevel?: number;

  /**
   * `.tsx` mode if `true`, in which a lone type parameter prints as `<T,>`.
   * Defaults to `false`.
   */
  jsx?: boolean;

  /**
   * `true` if the AST may contain TypeScript syntax (TS-ESLint dialect).
   * Defaults to `false`.
   */
  ts?: boolean;

  /**
   * If present, source mappings are emitted through it.
   */
  sourceMap?: SourceMapGenerator;
}
