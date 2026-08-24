// Printing options and returned output types.

/** Standard Source Map v3 output, compatible with Rollup and other JavaScript tooling. */
export interface SourceMap {
  version: 3;
  mappings: string;
  names: string[];
  sources: string[];
  sourcesContent?: string[];
}

/** Result returned by `printSync`. */
export interface CodegenResult {
  code: string;
  map: SourceMap | null;
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
   * Generate and return a source map in `CodegenResult.map`.
   */
  sourcemap?: boolean;

  /**
   * Original source text. Required when `sourcemap` is `true`.
   */
  sourceText?: string;

  /**
   * Original source filename recorded in the returned source map.
   */
  sourceFilename?: string;
}
