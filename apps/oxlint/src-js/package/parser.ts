export interface ParserLike {
  parse?: (code: string, options?: Record<string, unknown>) => unknown;
  parseForESLint?: (code: string, options?: Record<string, unknown>) => unknown;
  VisitorKeys?: Readonly<Record<string, readonly string[]>>;
  Syntax?: Readonly<Record<string, string>>;
  name?: string;
  version?: string;
  latestEcmaVersion?: number;
  supportedEcmaVersions?: readonly number[];
}
