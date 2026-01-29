// Entry point for definePlugin and defineRule
export { definePlugin, defineRule } from "./package/define.ts";

// ESTree types
export type * as ESTree from "./generated/types.d.ts";

// Plugin types
export type { Context, LanguageOptions } from "./plugins/context.ts";
export type { Fix, Fixer, FixFn } from "./plugins/fix.ts";
export type { Globals, Envs } from "./plugins/globals.ts";
export type { CreateOnceRule, CreateRule, Plugin, Rule } from "./plugins/load.ts";
export type { Options, RuleOptionsSchema } from "./plugins/options.ts";
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
  Visitor,
  VisitorWithHooks,
} from "./plugins/types.ts";
