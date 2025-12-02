// Functions and classes
export { definePlugin, defineRule } from "./package/define.ts";
export { RuleTester } from "./package/rule_tester.ts";

// ESTree types
export type * as ESTree from "./generated/types.d.ts";

// Plugin types
export type { Context, LanguageOptions } from "./plugins/context.ts";
export type { Fix, Fixer, FixFn } from "./plugins/fix.ts";
export type { CreateOnceRule, CreateRule, Plugin, Rule } from "./plugins/load.ts";
export type { Options } from "./plugins/options.ts";
export type { Diagnostic, DiagnosticData, Suggestion } from "./plugins/report.ts";
export type {
  Definition,
  DefinitionType,
  Reference,
  Scope,
  ScopeManager,
  ScopeType,
  Variable,
} from "./plugins/scope.ts";
export type { Settings } from "./plugins/settings.ts";
export type { SourceCode } from "./plugins/source_code.ts";
export type {
  CountOptions,
  FilterFn,
  RangeOptions,
  SkipOptions,
  Token,
  BooleanToken,
  CommentToken,
  BlockCommentToken,
  LineCommentToken,
  IdentifierToken,
  JSXIdentifierToken,
  JSXTextToken,
  KeywordToken,
  NullToken,
  NumericToken,
  PrivateIdentifierToken,
  PunctuatorToken,
  RegularExpressionToken,
  StringToken,
  TemplateToken,
} from "./plugins/tokens.ts";
export type {
  RuleMeta,
  RuleDocs,
  RuleOptionsSchema,
  RuleDeprecatedInfo,
  RuleReplacedByInfo,
  RuleReplacedByExternalSpecifier,
} from "./plugins/rule_meta.ts";
export type { LineColumn, Location, Range, Ranged, Span } from "./plugins/location.ts";
export type {
  AfterHook,
  BeforeHook,
  Comment,
  Node,
  NodeOrToken,
  Visitor,
  VisitorWithHooks,
} from "./plugins/types.ts";

// Rule tester types.
// Export as namespace to avoid lengthy type names.
import type {
  Config as _Config,
  DescribeFn as _DescribeFn,
  ItFn as _ItFn,
  ValidTestCase as _ValidTestCase,
  InvalidTestCase as _InvalidTestCase,
  TestCases as _TestCases,
  Error as _Error,
} from "./package/rule_tester.ts";

export namespace RuleTester {
  export type Config = _Config;
  export type DescribeFn = _DescribeFn;
  export type ItFn = _ItFn;
  export type ValidTestCase = _ValidTestCase;
  export type InvalidTestCase = _InvalidTestCase;
  export type TestCases = _TestCases;
  export type Error = _Error;
}
